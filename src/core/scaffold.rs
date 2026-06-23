//! Project scaffolding for Decapod initialization.
//!
//! This module handles the creation of Decapod project structure, including:
//! - Repository root entrypoints (AGENTS.md, CLAUDE.md, etc.)
//! - Embedded methodology documents

use crate::core::assets;
use crate::core::capsule_policy::{GENERATED_POLICY_REL_PATH, default_policy_json_pretty};
use crate::core::error;
use crate::core::project_specs::{
    LOCAL_PROJECT_SPECS, LOCAL_PROJECT_SPECS_ARCHITECTURE, LOCAL_PROJECT_SPECS_INTENT,
    LOCAL_PROJECT_SPECS_INTERFACES, LOCAL_PROJECT_SPECS_MANIFEST,
    LOCAL_PROJECT_SPECS_MANIFEST_SCHEMA, LOCAL_PROJECT_SPECS_OPERATIONS,
    LOCAL_PROJECT_SPECS_README, LOCAL_PROJECT_SPECS_SECURITY, LOCAL_PROJECT_SPECS_SEMANTICS,
    LOCAL_PROJECT_SPECS_VALIDATION, ProjectSpecManifestEntry, ProjectSpecsManifest, hash_text,
    read_specs_manifest, repo_signal_fingerprint,
};
use crate::plugins::container;
use std::fs;
use std::path::{Path, PathBuf};

/// Scaffolding operation configuration.
///
/// Controls how project initialization templates are written to disk.
pub struct ScaffoldOptions {
    /// Target directory for scaffold output (usually project root)
    pub target_dir: PathBuf,
    /// Force overwrite of existing files
    pub force: bool,
    /// Preview mode - log actions without writing files
    pub dry_run: bool,
    /// Which agent entrypoint files to generate (empty = all)
    pub agent_files: Vec<String>,
    /// Whether .bak files were created during init
    pub created_backups: bool,
    /// Force creation of all 5 entrypoint files regardless of existing state
    pub all: bool,
    /// Preserved content from hijacked agent entrypoints to be blended into OVERRIDE.md.
    pub preserved_agent_content: Vec<(String, String)>,
    /// Generate project-facing specs/ scaffolding.
    pub generate_specs: bool,
    /// Generate GitHub Action workflow for project validation.
    pub generate_ci: bool,
    /// Diagram style for generated architecture document.
    pub diagram_style: DiagramStyle,
    /// Intent/architecture seed captured from inferred or user-confirmed repo context.
    pub specs_seed: Option<SpecsSeed>,
}

pub struct ScaffoldSummary {
    pub entrypoints_created: usize,
    pub entrypoints_unchanged: usize,
    pub entrypoints_preserved: usize,
    pub config_created: usize,
    pub config_unchanged: usize,
    pub config_preserved: usize,
    pub specs_created: usize,
    pub specs_unchanged: usize,
    pub specs_preserved: usize,
    pub ci_created: usize,
    pub ci_unchanged: usize,
    pub ci_preserved: usize,
}

#[derive(Clone, Copy, Debug)]
pub enum DiagramStyle {
    Ascii,
    Mermaid,
}

#[derive(Clone, Debug)]
pub struct SpecsSeed {
    pub product_name: Option<String>,
    pub product_summary: Option<String>,
    pub architecture_direction: Option<String>,
    pub product_type: Option<String>,
    pub primary_languages: Vec<String>,
    pub detected_surfaces: Vec<String>,
    pub done_criteria: Option<String>,
}

pub const PROJECT_SPEC_TEMPLATE_VERSION: &str = "scaffold-v3";

fn joined_or_fallback(items: &[String], fallback: &str) -> String {
    if items.is_empty() {
        fallback.to_string()
    } else {
        items.join(", ")
    }
}

fn default_test_commands(seed: Option<&SpecsSeed>) -> Vec<String> {
    let mut commands = Vec::new();
    let langs = seed.map(|s| s.primary_languages.as_slice()).unwrap_or(&[]);
    let surfaces = seed.map(|s| s.detected_surfaces.as_slice()).unwrap_or(&[]);

    if langs
        .iter()
        .any(|l| l.to_ascii_lowercase().contains("rust"))
    {
        commands.push("cargo test".to_string());
    }
    if surfaces.iter().any(|s| s.eq_ignore_ascii_case("npm"))
        || langs.iter().any(|l| {
            let l = l.to_ascii_lowercase();
            l.contains("typescript") || l.contains("javascript")
        })
    {
        commands.push("npm test".to_string());
    }
    if langs.iter().any(|l| l.eq_ignore_ascii_case("python")) {
        commands.push("pytest".to_string());
    }
    if langs.iter().any(|l| l.eq_ignore_ascii_case("go")) {
        commands.push("go test ./...".to_string());
    }
    commands
}

fn has_language(seed: Option<&SpecsSeed>, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    seed.map(|s| {
        s.primary_languages
            .iter()
            .any(|l| l.to_ascii_lowercase().contains(&needle))
    })
    .unwrap_or(false)
}

fn primary_language_name(seed: Option<&SpecsSeed>) -> String {
    seed.and_then(|s| s.primary_languages.first().cloned())
        .unwrap_or_else(|| "not detected yet".to_string())
}

fn language_specific_test_criteria(seed: Option<&SpecsSeed>) -> Vec<String> {
    if has_language(seed, "rust") {
        return vec![
            "`cargo test` passes for unit/integration coverage".to_string(),
            "`cargo clippy -- -D warnings` passes with no denied lints".to_string(),
            "`cargo fmt --check` passes on the repo".to_string(),
        ];
    }
    if has_language(seed, "python") {
        return vec![
            "`pytest -q` passes for unit/integration scenarios".to_string(),
            "`ruff check .` passes for lint quality".to_string(),
            "`mypy .` passes for typed modules in production paths".to_string(),
        ];
    }
    if has_language(seed, "go") {
        return vec![
            "`go test ./...` passes for all packages".to_string(),
            "`go vet ./...` passes with no diagnostics".to_string(),
            "`gofmt -l .` returns no files".to_string(),
        ];
    }
    if has_language(seed, "typescript") || has_language(seed, "javascript") {
        return vec![
            "`npm test` (or `pnpm test`) passes for unit/integration suites".to_string(),
            "`npm run lint` passes".to_string(),
            "`npm run typecheck` passes for strict TS projects".to_string(),
        ];
    }
    vec!["Repository test/lint/typecheck commands are defined and wired into CI.".to_string()]
}

fn language_specific_error_example(seed: Option<&SpecsSeed>) -> String {
    if has_language(seed, "rust") {
        return r#"```rust
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("upstream timeout")]
    UpstreamTimeout,
    #[error("conflict: {0}")]
    Conflict(String),
}
```"#
            .to_string();
    }
    if has_language(seed, "python") {
        return r#"```python
class ApiError(Exception):
    def __init__(self, code: str, message: str) -> None:
        self.code = code
        self.message = message
        super().__init__(f"{code}: {message}")
```"#
            .to_string();
    }
    if has_language(seed, "go") {
        return r#"```go
var (
    ErrValidation = errors.New("validation_failed")
    ErrTimeout    = errors.New("upstream_timeout")
    ErrConflict   = errors.New("conflict")
)
```"#
            .to_string();
    }
    r#"```ts
export enum ApiErrorCode {
  Validation = "validation_failed",
  UpstreamTimeout = "upstream_timeout",
  Conflict = "conflict"
}
```"#
        .to_string()
}

fn language_specific_supply_chain_tools(seed: Option<&SpecsSeed>) -> Vec<String> {
    if has_language(seed, "rust") {
        return vec!["cargo audit", "cargo deny", "cargo vet"]
            .into_iter()
            .map(str::to_string)
            .collect();
    }
    if has_language(seed, "python") {
        return vec!["pip-audit", "safety", "bandit"]
            .into_iter()
            .map(str::to_string)
            .collect();
    }
    if has_language(seed, "go") {
        return vec!["govulncheck", "gosec", "nancy"]
            .into_iter()
            .map(str::to_string)
            .collect();
    }
    vec!["npm audit", "osv-scanner", "snyk"]
        .into_iter()
        .map(str::to_string)
        .collect()
}

fn language_specific_logging_hint(seed: Option<&SpecsSeed>) -> String {
    if has_language(seed, "rust") {
        return "Use `tracing` + `tracing-subscriber` with structured JSON output and request correlation ids.".to_string();
    }
    if has_language(seed, "python") {
        return "Use `structlog` (or stdlib logging JSON formatter) with request_id, task_id, and outcome fields.".to_string();
    }
    if has_language(seed, "go") {
        return "Use `zap` or `zerolog` with structured fields and propagated context ids."
            .to_string();
    }
    "Use structured logging (pino/winston) with request_id, actor, latency_ms, and error_code fields.".to_string()
}

fn adaptive_topology_diagram(style: DiagramStyle, seed: Option<&SpecsSeed>) -> String {
    let product_type = seed
        .and_then(|s| s.product_type.as_deref())
        .unwrap_or("service");
    match style {
        DiagramStyle::Ascii => {
            if product_type.contains("cli") {
                r#"```text
User -> CLI Entrypoint -> Command Router -> Core Engine -> Local Store
                                      \-> External API / Filesystem
```"#
                    .to_string()
            } else if product_type.contains("frontend") {
                r#"```text
Browser -> UI Shell -> API Client -> Backend Gateway -> Datastores / Events
```"#
                    .to_string()
            } else if product_type.contains("library") {
                r#"```text
Host Application -> Library API -> Domain Core -> Adapters (Store / Network)
```"#
                    .to_string()
            } else {
                r#"```text
Client -> API Gateway -> Service Core -> Worker Queue -> Datastores
```"#
                    .to_string()
            }
        }
        DiagramStyle::Mermaid => {
            if product_type.contains("cli") {
                r#"```mermaid
flowchart LR
  U[User] --> C[CLI Entrypoint]
  C --> R[Command Router]
  R --> E[Core Engine]
  E --> S[(Local Store)]
  E --> X[External APIs / Filesystem]
```"#
                    .to_string()
            } else if product_type.contains("frontend") {
                r#"```mermaid
flowchart LR
  B[Browser] --> UI[UI Shell]
  UI --> A[API Client]
  A --> G[Backend Gateway]
  G --> DB[(Datastore)]
  G --> Q[(Event Bus)]
```"#
                    .to_string()
            } else if product_type.contains("library") {
                r#"```mermaid
flowchart LR
  H[Host Application] --> L[Library API]
  L --> D[Domain Core]
  D --> AD[Adapter Layer]
  AD --> DB[(Store)]
  AD --> N[Network]
```"#
                    .to_string()
            } else {
                r#"```mermaid
flowchart LR
  C[Client] --> G[API Gateway]
  G --> S[Service Core]
  S --> W[Workers]
  S --> DB[(Primary Datastore)]
  W --> Q[(Queue)]
```"#
                    .to_string()
            }
        }
    }
}

fn adaptive_happy_path_sequence(style: DiagramStyle, seed: Option<&SpecsSeed>) -> String {
    let product_type = seed
        .and_then(|s| s.product_type.as_deref())
        .unwrap_or("service");
    match style {
        DiagramStyle::Ascii => {
            if product_type.contains("cli") {
                r#"```text
User invokes command -> CLI parses args -> Core executes action -> state persists -> result printed
```"#
                    .to_string()
            } else if product_type.contains("frontend") {
                r#"```text
User action -> UI validates input -> API request -> backend persists -> UI renders success
```"#
                    .to_string()
            } else {
                r#"```text
Client request -> API validation -> domain execution -> persistence -> response with trace id
```"#
                    .to_string()
            }
        }
        DiagramStyle::Mermaid => {
            if product_type.contains("cli") {
                r#"```mermaid
sequenceDiagram
  participant U as User
  participant C as CLI
  participant E as Core Engine
  participant S as Store
  U->>C: Run command
  C->>E: Parse + validate
  E->>S: Persist mutation
  S-->>E: Ack
  E-->>C: Result
  C-->>U: Structured output
```"#
                    .to_string()
            } else if product_type.contains("frontend") {
                r#"```mermaid
sequenceDiagram
  participant U as User
  participant UI as Frontend
  participant API as Backend API
  participant DB as Datastore
  U->>UI: Submit action
  UI->>API: Authenticated request
  API->>DB: Write transaction
  DB-->>API: Commit ok
  API-->>UI: 200 + payload
  UI-->>U: Updated view
```"#
                    .to_string()
            } else {
                r#"```mermaid
sequenceDiagram
  participant C as Client
  participant G as API
  participant D as Domain
  participant DB as Datastore
  C->>G: Request
  G->>D: Validate + execute
  D->>DB: Commit transaction
  DB-->>D: Commit ok
  D-->>G: Domain result
  G-->>C: Response + trace_id
```"#
                    .to_string()
            }
        }
    }
}

fn specs_readme_template(seed: Option<&SpecsSeed>) -> String {
    let product = seed
        .and_then(|s| s.product_name.as_deref())
        .unwrap_or("this repository");
    let summary = seed
        .and_then(|s| s.product_summary.as_deref())
        .unwrap_or("Define the intended user-visible outcome.");
    let languages = joined_or_fallback(
        seed.map(|s| s.primary_languages.as_slice()).unwrap_or(&[]),
        "not detected yet",
    );
    let surfaces = joined_or_fallback(
        seed.map(|s| s.detected_surfaces.as_slice()).unwrap_or(&[]),
        "not detected yet",
    );

    format!(
        r#"# Project Specs

Canonical path: `.decapod/generated/specs/`.
These files are the project-local contract for humans and agents.

## Snapshot
- Project: {product}
- Outcome: {summary}
- Detected languages: {languages}
- Detected surfaces: {surfaces}

## How to use this folder
- [INTENT.md](./INTENT.md): what success means and what is explicitly out of scope.
- [ARCHITECTURE.md](./ARCHITECTURE.md): topology, runtime model, data boundaries, and ADR trail.
- [INTERFACES.md](./INTERFACES.md): API/CLI/events/storage contracts and failure behavior.
- [VALIDATION.md](./VALIDATION.md): proof commands, quality gates, and evidence artifacts.
- [SEMANTICS.md](./SEMANTICS.md): state machines, invariants, replay rules, and idempotency.
- [OPERATIONS.md](./OPERATIONS.md): SLOs, monitoring, incident response, and rollout strategy.
- [SECURITY.md](./SECURITY.md): threat model, trust boundaries, auth/authz, and supply-chain posture.

## Canonical `.decapod/` Layout
- `.decapod/data/`: canonical control-plane state (SQLite + ledgers).
- `.decapod/generated/specs/`: **Living project specs** for humans and agents.
- `.decapod/generated/context/`: deterministic context capsules.
- `.decapod/generated/policy/context_capsule_policy.json`: repo-native JIT context policy contract.
- `.decapod/generated/artifacts/provenance/`: promotion manifests and convergence checklist.
- `.decapod/generated/artifacts/custody/`: epistemic custody artifacts (assumptions, contradictions, deferred questions).
- `.decapod/generated/artifacts/inventory/`: deterministic release inventory.
- `.decapod/generated/artifacts/diagnostics/`: opt-in diagnostics artifacts.
- `.decapod/workspaces/`: isolated todo-scoped git worktrees.

## Day-0 Onboarding Checklist
- [ ] Replace all placeholders in all 8 spec files.
- [ ] Confirm primary user outcome and acceptance criteria in [INTENT.md](./INTENT.md).
- [ ] Confirm topology and runtime model in [ARCHITECTURE.md](./ARCHITECTURE.md).
- [ ] Document all inbound/outbound contracts in [INTERFACES.md](./INTERFACES.md).
- [ ] Define validation gates and CI proof surfaces in [VALIDATION.md](./VALIDATION.md).
- [ ] Define state machines and invariants in [SEMANTICS.md](./SEMANTICS.md).
- [ ] Define SLOs, alerting, and incident process in [OPERATIONS.md](./OPERATIONS.md).
- [ ] Define threat model and auth/authz decisions in [SECURITY.md](./SECURITY.md).
- [ ] Ensure architecture diagram, docs, changelog, and tests are mapped to promotion gates.
- [ ] Run all validation/test commands and attach evidence artifacts.
"#
    )
}

fn specs_intent_template(seed: Option<&SpecsSeed>) -> String {
    let product_outcome = seed
        .and_then(|s| s.product_summary.as_deref())
        .unwrap_or("Define the user-visible outcome in one paragraph.");
    let done_criteria = seed
        .and_then(|s| s.done_criteria.as_deref())
        .unwrap_or("Functional behavior is demonstrably correct.");
    let product_name = seed
        .and_then(|s| s.product_name.as_deref())
        .unwrap_or("this repository");
    let product_type = seed
        .and_then(|s| s.product_type.as_deref())
        .unwrap_or("not classified yet");
    let languages = joined_or_fallback(
        seed.map(|s| s.primary_languages.as_slice()).unwrap_or(&[]),
        "not detected yet",
    );
    let surfaces = joined_or_fallback(
        seed.map(|s| s.detected_surfaces.as_slice()).unwrap_or(&[]),
        "not detected yet",
    );
    let language_criteria = language_specific_test_criteria(seed)
        .into_iter()
        .map(|s| format!("- [ ] {s}"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"# Intent

## Product Outcome
- {product_outcome}

## What This Project Is
Decapod is a daemonless, local-first governance kernel for AI coding agents. It is not a passive checklist or a documentation folder. Agents invoke Decapod at governance boundaries to turn human intent into explicit local contracts, refresh generated context, enforce workspace and policy boundaries, coordinate mutable work, and require proof-backed completion.

Key operating facts:
- **Agent control plane**: Agents call Decapod before inference-heavy work, before workspace mutation, before validation, and before claiming completion.
- **Repo-native state**: Canonical mutable state lives under `.decapod/`, including todos, generated specs, context capsules, proof artifacts, policy, and isolated workspaces.
- **Constitution-driven context**: The embedded constitution and project overrides provide queryable doctrine for architecture, interfaces, security, testing, delivery, and agent behavior.
- **Generated specs as live contracts**: `.decapod/generated/specs/*.md` are generated from repo context and refreshed by Decapod execution so agents receive current architecture, interface, validation, operational, and security context.
- **Todo-based coordination**: `decapod todo` provides claim ownership, dependencies, and event journaling for concurrent agents.
- **Validation and proof**: `decapod validate`, proof plans, health claims, and provenance artifacts form the promotion boundary.
- **Workspace isolation**: Todo-scoped git worktrees and optional containers keep agent changes out of the human root checkout.

## Product View
```mermaid
flowchart LR
  U[Primary User] --> P[{product_name}]
  P --> O[User-visible Outcome]
  P --> G[Proof Gates]
  G --> E[Evidence Artifacts]
```

## Inferred Baseline
- Repository: {product_name}
- Product type: {product_type}
- Primary languages: {languages}
- Detected surfaces: {surfaces}

## Scope
| Area | In Scope | Proof Surface |
|---|---|---|
| Core workflow | Define a concrete user-visible workflow | Acceptance criteria + tests |
| Data contracts | Document canonical inputs/outputs | [INTERFACES.md](./INTERFACES.md) and schema checks |
| Delivery quality | Block promotion on broken proof surfaces | [VALIDATION.md](./VALIDATION.md) blocking gates |

## Non-Goals (Falsifiable)
| Non-goal | How to falsify |
|---|---|
| Feature creep beyond the primary outcome | Any PR adds capability not tied to outcome criteria |
| Shipping without evidence | Missing validation artifacts for promoted changes |
| Ambiguous ownership boundaries | Missing owner/system-of-record in interfaces |

## Constraints
- Technical: runtime, dependency, and topology boundaries are explicit.
- Operational: deployment, rollback, and incident ownership are defined.
- Security/compliance: sensitive data handling and authz are mandatory.

## Acceptance Criteria (must be objectively testable)
- [ ] {done_criteria}
- [ ] Non-functional targets are met (latency, reliability, cost, etc.).
- [ ] Validation gates pass and artifacts are attached.
{language_criteria}

## Epistemic Custody Fields

### Active Assumptions
- [ ] List any assumptions made to proceed.
- [ ] Flag assumptions that require future verification.

### Confidence & Risk Level
- **Confidence**: Low/Medium/High (Rationale: )
- **Risk**: Low/Medium/High (Impact of wrong assumptions: )

### Measured vs Inferred Facts
| Fact | Source (Provenance) | Type (Measured/Inferred) |
|---|---|---|
| | | |

### Unresolved Contradictions
- [ ] List any evidence that conflicts with current assumptions or intent.

### Deferred Questions
- [ ] Questions to be answered later.

### Stop Conditions
- [ ] Explicit conditions under which the agent should stop and ask for help.

### Proof Required Before Completion
- [ ] Specific evidence needed to prove the outcome is met.

## Tradeoffs Register
| Decision | Benefit | Cost | Review Trigger |
|---|---|---|---|
| Simplicity vs extensibility | Faster iteration | Potential rework | Feature set expands |
| Strict gates vs dev speed | Higher confidence | More upfront discipline | Lead time regressions |

## First Implementation Slice
- [ ] Define the smallest user-visible workflow to ship first.
- [ ] Define required data/contracts for that workflow.
- [ ] Define what is intentionally postponed until v2.

## Open Questions (with decision deadlines)
| Question | Owner | Deadline | Decision |
|---|---|---|---|
| Which interfaces are versioned at launch? | TBD | YYYY-MM-DD | |
| Which non-functional target is hardest to hit? | TBD | YYYY-MM-DD | |
"#
    )
}

fn specs_architecture_template(style: DiagramStyle, seed: Option<&SpecsSeed>) -> String {
    let summary = seed
        .and_then(|s| s.architecture_direction.as_deref())
        .unwrap_or(
            "Describe architecture in deployment-reality terms: runtime boundaries, operational ownership, and failure containment.",
        );
    let runtime_langs = seed
        .map(|s| s.primary_languages.join(", "))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "to be confirmed".to_string());
    let surfaces = seed
        .map(|s| s.detected_surfaces.join(", "))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "to be confirmed".to_string());
    let product_type = seed
        .and_then(|s| s.product_type.as_deref())
        .unwrap_or("to be confirmed");

    format!(
        r#"# Architecture

## Direction
{summary}

## What This Project Is
Decapod is a daemonless, local-first governance kernel for AI coding agents. Its architecture centers on an explicit control plane: agents call Decapod to refine intent, resolve context, claim exclusive work, generate living specs, execute inside isolated workspaces, and emit proof before promotion.

Architectural principles:
- **Daemonless invocation**: Decapod should run only when invoked and must not require a background service to enforce governance.
- **Local-first authority**: The repository carries the operational truth under `.decapod/`; external systems may reference work but do not replace Decapod's coordination state.
- **Generated architecture custody**: Generated specs are the agent-facing architecture map. They should be refreshed from current repo facts instead of relying on a stale manually maintained design note.
- **Explicit boundaries**: CLI/RPC contracts, store ownership, validation gates, workspace isolation, and proof artifacts are separate surfaces with documented responsibilities.
- **Proof before promotion**: Completion claims are only credible when validation gates and artifacts support them.

## Current Facts
- Runtime/languages: {runtime_langs}
- Detected surfaces/framework hints: {surfaces}
- Product type: {product_type}

## Architecture Map
This project's architecture consists of several interconnected subsystems:
1. **CLI Layer** (`src/cli.rs`): Defines command-line interface and argument parsing for all Decapod commands (init, todo, validate, proof, workspace, etc.).
2. **Core Runtime** (`src/core/`):
   - **Todo System** (`todo.rs`): SQLite-backed task management with claims, ownership, dependencies, and event journaling.
   - **Validation** (`validate.rs`): Intent-driven methodology validation harness with numerous gates (namespace purge, embedded self-contained, repo map, etc.).
   - **Proof** (`proof.rs`): Configurable proof execution from `proofs.toml` with health claim synchronization.
   - **Workspace** (`workspace.rs`): Git worktree and Docker container management for isolated agent workspaces.
   - **Constitution Access** (`assets.rs`): Embedded constitution retrieval and merging with project overrides (OVERRIDE.md).
   - **Specs Generation** (`project_specs.rs`): Scaffolding and manifest management for `.decapod/generated/specs/`.
   - **Storage** (`store.rs`, `db.rs`): Store abstraction and SQLite database access.
   - **Broker** (`broker.rs`): Audit log mutation broker with replay verification.
   - **Workunit** (`workunit.rs`): Work unit manifests linking tasks to specs, state, and proof.
   - **Plan Governance** (`plan_governance.rs`): Governed PLAN artifacts with state transitions.
   - **Context Capsules** (`context_capsule.rs`): Deterministic context resolution from embedded constitution.
   - **Obligation Graph** (`obligation.rs`): Obligation tracking for governance.
   - **Flight Recorder** (`flight_recorder.rs`): Timeline rendering from event logs.
3. **Plugins** (`src/plugins/`): Extensible subsystems (aptitude, federation, health, knowledge, policy, verify, workflow, etc.).
4. **Constitution** (`src/constitution/`): Core constitution nodes (core, interfaces, methodology, specs) re-exported via `src/constitution/core.rs`.
5. **Agent Interface**: Agent-facing documentation (AGENTS.md, CLAUDE.md, etc.) and the Universal Agent Contract (AGENTS.md) that mandates Decapod usage patterns.

## Generated Architecture Contract
Decapod-generated architecture documentation must be detailed enough for a new agent to orient without reading an external architecture note. It should describe:
- What the project is and what it is not.
- The control-plane entrypoints agents call and when those calls are required.
- Core subsystems, storage locations, and event journals.
- Data flows from agent invocation through CLI/RPC, core execution, persistence, validation, and proof emission.
- Current strongest primitives and the extension seams they imply.
- Known limitations, architectural risks, and traps to avoid.
- Candidate follow-up work as design issues, not hidden notes.

## Data Flows
- Agent → Decapod CLI → Core subsystems (todo, validate, proof, workspace, etc.) → `.decapod/` storage
- Constitution access via `assets.rs` → embedded JSON or merged with OVERRIDE.md
- Specs generated via `project_specs.rs` → `.decapod/generated/specs/`
- Workspaces created via `workspace.rs` → `.decapod/workspaces/` (git worktrees)
- Events journaled via todo and proof subsystems → `.decapod/data/todo.events.jsonl` and `proof.events.jsonl`

## Strongest Existing Primitives
1. **Constitution Access System**: The assets module provides robust, versioned access to embedded constitution documents with override capability via OVERRIDE.md. This is a mature, well-tested system for declarative governance.
2. **Todo Subsystem**: The todo.rs implementation features rich task properties (dependencies, blocking, ownership, claims, verification), event journaling for deterministic rebuild, and sophisticated claiming mechanisms for agent isolation.
3. **Validation Harness**: The validate.rs module implements a comprehensive, extensible validation suite with clear pass/fail/warn semantics and auto-remediation hints.
4. **Workspace Isolation**: The workspace.rs module provides sophisticated git worktree management with branch protection, containerization support, and todo-scoped branch naming.
5. **Specs as Living Contracts**: The project_specs.rs system creates a feedback loop between generated specs and user intent, with manifest-based change detection and update guidance.
6. **Proof System**: The proof.rs module enables configurable, auditable proof execution with health claim synchronization and event logging.
7. **Embedded Agent Contract**: The AGENTS.md file (and templates in assets.rs) provides a comprehensive, machine-readable contract that agents must follow, reducing ambiguity.

## Topology
{topology}

## Store Boundaries
```mermaid
flowchart LR
  I[Inbound Requests] --> C[Core Runtime]
  C --> W[(Write Store)]
  C --> R[(Read Store)]
  C --> E[External Dependency]
  E --> DLQ[(DLQ / Retry Queue)]
```

## Happy Path Sequence
{happy_path}

## Error Path
```mermaid
sequenceDiagram
  participant Client
  participant API
  participant Upstream
  Client->>API: Request
  API->>Upstream: Call with timeout budget
  Upstream--xAPI: Timeout / failure
  API-->>Client: Typed error + retry guidance + trace_id
```

## Execution Path
- Ingress parse + validation:
- Policy/interlock checks:
- Core execution + persistence:
- Verification and artifact emission:

## Concurrency and Runtime Model
- Execution model:
- Isolation boundaries:
- Backpressure strategy:
- Shared state synchronization:

## Deployment Topology
- Runtime units:
- Region/zone model:
- Rollout strategy (blue/green/canary):
- Rollback trigger and blast-radius scope:

## Data and Contracts
- Inbound contracts (CLI/API/events):
- Outbound dependencies (datastores/queues/external APIs):
- Data ownership boundaries:
- Schema evolution + migration policy:

## ADR Register
| ADR | Title | Status | Rationale | Date |
|---|---|---|---|---|
| ADR-001 | Initial topology choice | Proposed | Define first stable architecture | YYYY-MM-DD |

## Delivery Plan (first 3 slices)
- Slice 1 (ship first):
- Slice 2:
- Slice 3:

## Risks and Mitigations
| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Contract drift across components | Medium | High | Spec + schema checks in CI |
| Runtime saturation under peak load | Medium | High | Capacity model + load tests |
"#,
        topology = adaptive_topology_diagram(style, seed),
        happy_path = adaptive_happy_path_sequence(style, seed),
    )
}

fn specs_interfaces_template(seed: Option<&SpecsSeed>) -> String {
    let surfaces = joined_or_fallback(
        seed.map(|s| s.detected_surfaces.as_slice()).unwrap_or(&[]),
        "not detected yet",
    );
    let product_type = seed
        .and_then(|s| s.product_type.as_deref())
        .unwrap_or("not classified yet");
    let error_example = language_specific_error_example(seed);

    format!(
        r#"# Interfaces

## Contract Principles
- Prefer explicit schemas over implicit behavior.
- Every mutating interface defines idempotency semantics.
- Every failure path maps to a typed, documented error code.

## CLI Layer
Defines command-line interface and argument parsing for all Decapod commands. CLI contracts should describe command purpose, preconditions, expected side effects, output shape, recovery path, and whether the command may mutate `.decapod/` state.

## Agent Interface
Agent-facing documentation (AGENTS.md, CLAUDE.md, etc.) and the Universal Agent Contract mandate Decapod usage patterns. Product docs under `docs/agent` and `docs/book` teach Decapod itself; generated specs under `.decapod/generated/specs/` teach the current repository to active agents.

## Generated Contract Depth
Generated interface specs should include more than endpoint names:
- Command/RPC trigger conditions and required prior Decapod calls.
- Read/write ownership for each store and generated artifact path.
- Idempotency and retry behavior for mutations.
- Typed failure classes and concrete recovery instructions.
- Compatibility expectations for existing agent workflows.
- Evidence an agent must attach before claiming an interface change complete.

## API / RPC Contracts
| Interface | Method | Request Schema | Response Schema | Errors | Idempotency |
|---|---|---|---|---|---|
| `TODO` | `TODO` | `TODO` | `TODO` | `TODO` | `TODO` |

## Event Consumers
| Consumer | Event | Ordering Requirement | Retry Policy | DLQ Policy |
|---|---|---|---|---|
| `TODO` | `TODO` | `TODO` | `TODO` | `TODO` |

## Outbound Dependencies
| Dependency | Purpose | SLA | Timeout | Circuit-Breaker |
|---|---|---|---|---|
| `TODO` | `TODO` | `TODO` | `TODO` | `TODO` |

## Inbound Contracts
- API / RPC entrypoints:
- CLI surfaces:
- Event/webhook consumers:
- Repository-detected surfaces: {surfaces}

## Data Ownership
- Source-of-truth tables/collections:
- Cross-boundary read models:
- Consistency expectations:

## Error Taxonomy Example ({product_type})
{error_example}

## Failure Semantics
| Failure Class | Retry/Backoff | Client Contract | Observability |
|---|---|---|---|
| Validation | No retry | 4xx typed error | warn log + metric |
| Dependency timeout | Exponential backoff | 503 with retryable code | error log + alert |
| Conflict | Conditional retry | 409 with conflict detail | info log + metric |

## Timeout Budget
| Hop | Budget (ms) | Notes |
|---|---|---|
| Client -> Edge/API | 500 | Includes auth + routing |
| API -> Domain | 300 | Includes validation |
| Domain -> Store/Dependency | 200 | Includes retry overhead |

## Interface Versioning
- Version strategy (`v1`, date-based, semver):
- Backward-compatibility guarantees:
- Deprecation window and removal policy:
"#
    )
}

fn specs_validation_template(seed: Option<&SpecsSeed>) -> String {
    let commands = default_test_commands(seed);
    let test_commands = if commands.is_empty() {
        "- Add repository-specific test command(s) here.".to_string()
    } else {
        commands
            .into_iter()
            .map(|c| format!("- `{c}`"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        r#"# Validation

## Validation Philosophy
> Validation is a release gate, not documentation theater.

## Validation Harness
The validate.rs module implements a comprehensive, extensible validation suite with clear pass/fail/warn semantics and auto-remediation hints. Key features include:
- **Methodology Gates**: Numerous validation gates enforce intent-driven development practices
- **Auto-Remediation Hints**: Provides specific guidance on how to fix validation failures
- **Workspace Enforcement**: Ensures agents work in isolated git worktrees or containers
- **Specs Integrity**: Validates that living specs match repository state
- **Proof Requirements**: Requires evidence artifacts for promotion gates
- **Constitution Integration**: Validates adherence to embedded constitution directives

## Generated Spec Refresh Gates
Decapod must keep generated specs synchronized at governance pressure points. When repository surfaces change, validation should either fail with a concrete refresh instruction or, when explicitly requested through a refresh path, regenerate the existing spec files and update the manifest fingerprint. Refresh must update the canonical spec set rather than creating one-off analysis files.

Refresh-capable paths:
- `decapod validate --refresh-specs`
- `decapod rpc --op specs.refresh`
- initialization or scaffold refresh paths that regenerate `.decapod/generated/specs/*.md`

Refresh output requirements:
- Preserve hand-maintained epistemic custody fields where possible.
- Blend repo context into the existing canonical spec files.
- Update `.decapod/generated/specs/.manifest.json` after writing files.
- Avoid adding parallel project-state or architecture-survey documents outside the canonical spec set.

## Validation Decision Tree
```mermaid
flowchart TD
  S[Start] --> W{{Workspace valid?}}
  W -->|No| F1[Fail: workspace gate]
  W -->|Yes| T{{Tests pass?}}
  T -->|No| F2[Fail: test gate]
  T -->|Yes| D{{Docs + diagrams + changelog updated?}}
  D -->|No| F3[Fail: docs gate]
  D -->|Yes| V[Run decapod validate]
  V --> P{{All blocking gates pass?}}
  P -->|No| F4[Fail: promotion blocked]
  P -->|Yes| E[Emit promotion evidence]
```

## Promotion Flow
```mermaid
flowchart LR
  A[Plan] --> B[Implement]
  B --> C[Test]
  C --> D[Validate]
  D --> E[Assemble Evidence]
  E --> F[Promote]
```

## Proof Surfaces
- `decapod validate`
- Required test commands:
{test_commands}
- Required integration/e2e commands:

## Promotion Gates

## Blocking Gates
| Gate | Command | Evidence |
|---|---|---|
| Architecture + interface drift check | `decapod validate` | Gate output |
| Tests pass | project test command | CI + local logs |
| Docs + changelog current | repo docs checks | PR diff |
| Security critical checks pass | security scanner suite | scanner reports |

## Warning Gates
| Gate | Trigger | Follow-up SLA |
|---|---|---|
| Coverage regression warning | Coverage drops below target | 48h |
| Non-blocking perf drift | P95 regression below hard threshold | 72h |

## Evidence Artifacts
| Artifact | Path | Required For |
|---|---|---|
| Validation report | `.decapod/generated/artifacts/provenance/*` | Promotion |
| Test logs | CI artifact store | Promotion |
| Architecture diagram snapshot | `ARCHITECTURE.md` | Promotion |
| Changelog entry | `CHANGELOG.md` | Promotion |

## Regression Guardrails
- Baseline references:
- Statistical thresholds (if non-deterministic):
- Rollback criteria:

## Bounded Execution
| Operation | Timeout | Failure Mode |
|---|---|---|
| Validation | 30s | timeout or lock |
| Unit test suite | project-defined | non-zero exit |
| Integration suite | project-defined | non-zero exit |

## Coverage Checklist
- [ ] Unit tests cover critical branches.
- [ ] Integration tests cover key user flows.
- [ ] Failure-path tests cover retries/timeouts.
- [ ] Docs/diagram/changelog updates included.
"#
    )
}

fn specs_semantics_template(seed: Option<&SpecsSeed>) -> String {
    let lang = primary_language_name(seed);
    format!(
        r#"# Semantics

## State Machines
```mermaid
stateDiagram-v2
  [*] --> Draft
  Draft --> InProgress
  InProgress --> Verified
  InProgress --> Blocked
  Blocked --> InProgress
  Verified --> [*]
```

## Invariants
| Invariant | Type | Validation |
|---|---|---|
| No promoted change without proof | System | validation gate |
| Canonical source-of-truth per entity | Data | interface/spec review |
| Mutation events are replayable | Data | deterministic replay |

## Event Sourcing Schema
| Field | Type | Description |
|---|---|---|
| event_id | string | globally unique event id |
| aggregate_id | string | entity/workflow id |
| event_type | string | semantic transition |
| payload | object | transition data |
| recorded_at | timestamp | append time |

## Replay Semantics
- Replay order:
- Conflict resolution:
- Snapshot cadence:
- Determinism proof strategy:

## Error Code Semantics
- Namespace:
- Stable compatibility window:
- Mapping to retry/degrade behavior:

## Domain Rules
- Business rule 1:
- Business rule 2:
- Business rule 3:

## Idempotency Contracts
| Operation | Idempotency Key | Duplicate Behavior |
|---|---|---|
| create/update mutation | request_id | return original result |
| async enqueue | event_id | ignore duplicate enqueue |

## Language Note
- Primary language inferred: {lang}

## Trajectory Concepts
The current todo-centric model shows signs of strain when representing complex, evolving work:
- **Atomic Task Bias**: Treating work as discrete tickets loses the narrative of how intent evolves over time through iterations, pivots, and discoveries.
- **Context Loss**: Each todo is an isolated unit; there's no inherent mechanism to carry forward assumptions, context summaries, or learned lessons across a sequence of related work.
- **Manual Coordination**: Agents must manually maintain epistemic custody (preserving intent-context-assumptions-action-proof chains) across multiple todos, which is error-prone.
- **Limited Retrospection**: While the worker loop records lessons, there's no first-class construct for representing the evolutionary path of a feature or system.
- **Overloading Todo Fields**: Current attempts to store trajectory-like data (e.g., in description, tags, or custom fields) fight against the todo subsystem's primary purpose as a task tracker.
- **Missing Abstractions**: Concepts like "work streams," "feature branches," "evolutionary trajectories," or "custody chains" lack explicit representation, forcing agents to encode them implicitly in todo relationships or external documentation.

Trajectories as first-class entities would provide:
- Explicit modeling of work sequences with temporal ordering and state evolution.
- Built-in mechanisms for preserving uncertainty and recursive continuity of assumptions.
- Natural encapsulation of intent-context-specs-state-proof chains.
- Improved tooling for visualizing and navigating work histories.
- Better alignment with how humans and agents actually work in iterative, discovery-driven processes.

## Recommended Redesign Seam for Trajectories
The most appropriate seam for introducing trajectories is at the intersection of the todo, spec, and workunit subsystems, without replacing any existing functionality:
1. **Extend the Workunit Concept**: Elevate workunit.rs from a per-task manifest to a potential trajectory container. A trajectory could be a linked series of workunits representing the evolution of a feature, architectural decision, or system component.
2. **Trajectory Spec**: Introduce a new spec type (e.g., `TRAJECTORY.md`) in `.decapod/generated/specs/` that documents the evolutionary path, key decisions, and state transitions of a coherent work effort.
3. **Minimal Core Changes**:
   - Add a `trajectory_id` field to the workunit model (optional, for grouping).
   - Create a new `trajectory.rs` subsystem for trajectory-specific operations (creation, linking, state transitions).
   - Extend the validation harness to check trajectory integrity (e.g., monotonic state progression, proof linkage).
   - Add CLI commands under `decapod trajectory` for trajectory management (init, list, show, transition).
4. **Preserve Existing Todos**: Keep the todo subsystem unchanged for atomic task tracking. Trajectories would coordinate or group todos but not replace them.
5. **Leverage Existing Mechanisms**:
   - Use the existing event journaling pattern for trajectory mutations.
   - Reuse the spec manifest system for trajectory specs.
   - Utilize the workspace subsystem for trajectory-scoped isolation (optional).
   - Depend on the constitution for trajectory-related directives (e.g., methodology/trajectory_lifecycle).
"#
    )
}

fn specs_operations_template(seed: Option<&SpecsSeed>) -> String {
    let logging_hint = language_specific_logging_hint(seed);
    format!(
        r#"# Operations

## Operational Readiness Checklist
- [ ] On-call ownership defined.
- [ ] SLOs and alert thresholds defined.
- [ ] Dashboards for latency/errors/throughput are live.
- [ ] Runbooks linked for all Sev1/Sev2 alerts.
- [ ] Rollback plan validated.
- [ ] Capacity guardrails documented.

## Workspace Isolation
Git worktrees and optional Docker containers provide isolated workspaces scoped to specific todos, preventing interference with the main repository checkout. Key features:
- **Todo-scoped Worktrees**: Each todo gets an isolated git worktree with branch naming that includes todo IDs/hashes
- **Exclusive Agent Ownership**: Claiming mechanism ensures only one agent can work on a todo at a time
- **Event Journaling**: Todo state changes are journaled for deterministic rebuild
- **Health Subsystem Integration**: Proof events can be associated with todos via health claims

## Generated Artifact Operations
Generated artifacts are operational outputs, not static docs. Agents should expect Decapod to refresh `.decapod/generated/specs/*.md` during explicit refresh operations and validation-assisted refresh. The operation is bounded: product docs under `docs/` remain the human learning surface for Decapod itself, while generated specs carry repo-specific live architecture, interface, validation, semantic, operational, and security facts.

## Service Level Objectives
| SLI | SLO Target | Measurement Window | Owner |
|---|---|---|---|
| Availability | 99.9% | 30d | TBD |
| P95 latency | TBD | 7d | TBD |
| Error rate | < 1% | 7d | TBD |

## Monitoring
| Signal | Metric | Threshold | Alert |
|---|---|---|---|
| Traffic | requests/sec | baseline drift | warn |
| Latency | p95/p99 | threshold breach | page |
| Reliability | error ratio | threshold breach | page |
| Saturation | cpu/memory/queue depth | sustained high | page |

## Health Checks
- Liveness:
- Readiness:
- Dependency health:
- Synthetic transaction:

## Incident Response
- Detection:
- Triage:
- Mitigation:
- Communication:
- Post-mortem:

## Rollout Strategy
- Blue/green deployment:
- Canary release:
- Rolling update:
- Feature flags:

## Capacity Planning
- Traffic patterns:
- Resource utilization:
- Scaling triggers:

## Logging
{logging_hint}

## Secrets Management
| Secret | Source | Rotation | Consumer |
|---|---|---|---|
| External service auth material | managed runtime configuration | periodic | runtime services |
| Artifact signing material | managed signing service/local secure store | periodic | release pipeline |

## Security Testing
| Test Type | Cadence | Tooling |
|---|---|---|
| SAST | each PR | language linters/scanners |
| Dependency scan | each PR + weekly | supply-chain tools |
| DAST/pentest | scheduled | external/internal |

## Compliance and Audit
- Regulatory scope:
- Audit evidence location:
- Exception process:

## Pre-Promotion Security Checklist
- [ ] Threat model updated for changed surfaces.
- [ ] Auth/authz tests pass.
- [ ] Dependency vulnerability scan reviewed.
- [ ] No unresolved critical/high security findings.
"#
    )
}

fn specs_security_template(seed: Option<&SpecsSeed>) -> String {
    let scanners = language_specific_supply_chain_tools(seed)
        .into_iter()
        .map(|tool| format!("`{tool}`"))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        r#"# Security

## Threat Model
```mermaid
flowchart LR
   U[User/Client] --> A[Application Boundary]
   A --> D[(Data Stores)]
   A --> X[External Dependencies]
   I[Identity Provider] --> A
   A --> L[Audit Logs]
```

## STRIDE Table
| Threat | Surface | Mitigation | Verification |
|---|---|---|---|
| Spoofing | Auth boundary | strong auth + token validation | auth tests |
| Tampering | State mutation APIs | integrity checks + RBAC | integration tests |
| Repudiation | Critical actions | immutable audit logs | log review |
| Information disclosure | Data at rest/in transit | encryption + classification | security scans |
| Denial of service | Hot paths | rate limit + backpressure | load tests |
| Elevation of privilege | Admin interfaces | least privilege + policy checks | authz tests |

## Authentication
- Identity source:
- Token/session lifetime:
- Rotation and revocation:

## Authorization
- Role model:
- Resource-level policy:
- Privilege escalation controls:

## Data Classification
| Data Class | Examples | Storage Rules | Access Rules |
|---|---|---|---|
| Public | docs, non-sensitive metadata | standard | unrestricted |
| Internal | operational telemetry | controlled | team access |
| Sensitive | tokens, PII, secrets | encrypted | least privilege |

## Sensitive Data Handling
- Encryption at rest:
- Encryption in transit:
- Redaction in logs:
- Retention + deletion policy:

## Supply Chain Security
- Recommended scanners: {scanners}
- Dependency update cadence:
- Signed artifact/provenance strategy:

## Secrets Management
| Secret | Source | Rotation | Consumer |
|---|---|---|---|
| External service auth material | managed runtime configuration | periodic | runtime services |
| Artifact signing material | managed signing service/local secure store | periodic | release pipeline |

## Security Testing
| Test Type | Cadence | Tooling |
|---|---|---|
| SAST | each PR | language linters/scanners |
| Dependency scan | each PR + weekly | supply-chain tools |
| DAST/pentest | scheduled | external/internal |

## Compliance and Audit
- Regulatory scope:
- Audit evidence location:
- Exception process:

## Pre-Promotion Security Checklist
- [ ] Threat model updated for changed surfaces.
- [ ] Auth/authz tests pass.
- [ ] Dependency vulnerability scan reviewed.
- [ ] No unresolved critical/high security findings.

## Strongest Security Primitives
1. **Constitution Access System**: The assets module provides robust, versioned access to embedded constitution documents with override capability via OVERRIDE.md. This is a mature, well-tested system for declarative governance.
2. **Proof System**: The proof.rs module enables configurable, auditable proof execution with health claim synchronization and event logging.
3. **Workspace Isolation**: The workspace.rs module provides sophisticated git worktree management with branch protection, containerization support, and todo-scoped branch naming.

## Generated Security Analysis
Generated security specs should document the active trust boundaries exposed by repository facts: local state stores, generated artifacts, session tokens, workspace paths, command execution surfaces, policy gates, proof artifacts, and any external service integrations. Security output must distinguish confirmed repo facts from inferred risks and leave unresolved questions visible for future agents.
"#
    )
}

fn render_project_spec_content(
    rel_path: &str,
    diagram_style: DiagramStyle,
    seed: Option<&SpecsSeed>,
) -> Option<String> {
    match rel_path {
        LOCAL_PROJECT_SPECS_README => Some(specs_readme_template(seed)),
        LOCAL_PROJECT_SPECS_INTENT => Some(specs_intent_template(seed)),
        LOCAL_PROJECT_SPECS_ARCHITECTURE => Some(specs_architecture_template(diagram_style, seed)),
        LOCAL_PROJECT_SPECS_INTERFACES => Some(specs_interfaces_template(seed)),
        LOCAL_PROJECT_SPECS_VALIDATION => Some(specs_validation_template(seed)),
        LOCAL_PROJECT_SPECS_SEMANTICS => Some(specs_semantics_template(seed)),
        LOCAL_PROJECT_SPECS_OPERATIONS => Some(specs_operations_template(seed)),
        LOCAL_PROJECT_SPECS_SECURITY => Some(specs_security_template(seed)),
        _ => None,
    }
}

fn project_spec_scaffold_hash(rel_path: &str, diagram_style: DiagramStyle) -> Option<String> {
    render_project_spec_content(rel_path, diagram_style, None).map(|content| hash_text(&content))
}

fn preserve_intent_custody(project_root: &Path, rel_path: &str, mut content: String) -> String {
    if rel_path != LOCAL_PROJECT_SPECS_INTENT {
        return content;
    }

    let dest = project_root.join(rel_path);
    let Ok(existing_content) = fs::read_to_string(&dest) else {
        return content;
    };
    let Some(start_idx) = existing_content.find("## Epistemic Custody Fields") else {
        return content;
    };

    let end_marker = "## Tradeoffs Register";
    let custody_section = if let Some(end_idx) = existing_content[start_idx..].find(end_marker) {
        &existing_content[start_idx..start_idx + end_idx]
    } else {
        &existing_content[start_idx..]
    };

    if let Some(new_start_idx) = content.find("## Epistemic Custody Fields")
        && let Some(new_end_idx) = content[new_start_idx..].find(end_marker)
    {
        let mut new_merged = content[..new_start_idx].to_string();
        new_merged.push_str(custody_section.trim_end());
        new_merged.push_str("\n\n");
        new_merged.push_str(&content[new_start_idx + new_end_idx..]);
        content = new_merged;
    }

    content
}

pub fn refresh_project_specs_from_config(
    project_root: &Path,
) -> Result<ProjectSpecsManifest, error::DecapodError> {
    let config = crate::cli::DecapodProjectConfig::load(project_root)?;
    let diagram_style = match config.init.diagram_style {
        crate::cli::InitDiagramStyle::Ascii => DiagramStyle::Ascii,
        crate::cli::InitDiagramStyle::Mermaid => DiagramStyle::Mermaid,
    };
    let seed = SpecsSeed {
        product_name: config.repo.product_name,
        product_summary: config.repo.product_summary,
        architecture_direction: config.repo.architecture_direction,
        product_type: config.repo.product_type,
        primary_languages: config.repo.primary_languages,
        detected_surfaces: config.repo.detected_surfaces,
        done_criteria: config.repo.done_criteria,
    };
    refresh_project_specs(project_root, diagram_style, Some(&seed))
}

pub fn refresh_project_specs(
    project_root: &Path,
    diagram_style: DiagramStyle,
    seed: Option<&SpecsSeed>,
) -> Result<ProjectSpecsManifest, error::DecapodError> {
    let specs_dir = project_root.join(crate::core::project_specs::LOCAL_PROJECT_SPECS_DIR);
    fs::create_dir_all(&specs_dir).map_err(error::DecapodError::IoError)?;

    let existing_manifest = read_specs_manifest(project_root)?;
    let current_repo_fingerprint = repo_signal_fingerprint(project_root)?;
    let mut manifest_entries = Vec::new();
    let mut file_writes: Vec<(PathBuf, String)> = Vec::new();
    for spec in LOCAL_PROJECT_SPECS {
        let Some(mut content) = render_project_spec_content(spec.path, diagram_style, seed) else {
            continue;
        };

        if let Some(override_content) =
            assets::get_override_doc(project_root, spec.constitution_ref)
        {
            content = assets::merge_override_content(&content, &override_content);
        }

        let template_hash = project_spec_scaffold_hash(spec.path, diagram_style)
            .unwrap_or_else(|| hash_text(&content));
        let content = preserve_intent_custody(project_root, spec.path, content);
        let content_hash = hash_text(&content);
        let dest = project_root.join(spec.path);
        let existing_content = fs::read_to_string(&dest).ok();
        if existing_content.as_deref() != Some(content.as_str()) {
            file_writes.push((dest, content));
        }
        manifest_entries.push(ProjectSpecManifestEntry {
            path: spec.path.to_string(),
            template_hash,
            content_hash,
        });
    }

    let manifest = ProjectSpecsManifest {
        schema_version: LOCAL_PROJECT_SPECS_MANIFEST_SCHEMA.to_string(),
        template_version: PROJECT_SPEC_TEMPLATE_VERSION.to_string(),
        generated_at: existing_manifest
            .as_ref()
            .filter(|existing| {
                existing.schema_version == LOCAL_PROJECT_SPECS_MANIFEST_SCHEMA
                    && existing.template_version == PROJECT_SPEC_TEMPLATE_VERSION
                    && existing.repo_signal_fingerprint == current_repo_fingerprint
                    && existing.files == manifest_entries
                    && file_writes.is_empty()
            })
            .map(|existing| existing.generated_at.clone())
            .unwrap_or_else(crate::core::time::now_epoch_z),
        repo_signal_fingerprint: current_repo_fingerprint,
        files: manifest_entries,
    };

    if let Some(existing) = existing_manifest.as_ref()
        && existing.schema_version == manifest.schema_version
        && existing.template_version == manifest.template_version
        && existing.repo_signal_fingerprint == manifest.repo_signal_fingerprint
        && existing.files == manifest.files
        && file_writes.is_empty()
    {
        return Ok(existing.clone());
    }

    for (dest, content) in file_writes {
        ensure_parent(&dest)?;
        fs::write(dest, content).map_err(error::DecapodError::IoError)?;
    }

    let manifest_path = project_root.join(LOCAL_PROJECT_SPECS_MANIFEST);
    ensure_parent(&manifest_path)?;
    let manifest_body = serde_json::to_string_pretty(&manifest).map_err(|e| {
        error::DecapodError::ValidationError(format!("Failed to serialize specs manifest: {e}"))
    })?;
    fs::write(manifest_path, manifest_body).map_err(error::DecapodError::IoError)?;

    Ok(manifest)
}

/// Canonical .gitignore rules managed by `decapod init`.
///
/// These rules are appended (if missing) to the user's root `.gitignore`.
/// Keep this as the source of truth so new allowlists/denylists evolve through code review.
pub const DECAPOD_GITIGNORE_RULES: &[&str] = &[
    ".decapod/data",
    ".decapod/data/*",
    ".decapod/.stfolder",
    ".decapod/workspaces",
    ".decapod/generated/*",
    "!.decapod/data/",
    "!.decapod/data/knowledge.promotions.jsonl",
    "!.decapod/generated/Dockerfile",
    "!.decapod/generated/context/",
    "!.decapod/generated/context/*.json",
    "!.decapod/generated/policy/",
    "!.decapod/generated/policy/context_capsule_policy.json",
    "!.decapod/generated/artifacts/",
    "!.decapod/generated/artifacts/provenance/",
    "!.decapod/generated/artifacts/provenance/*.json",
    "!.decapod/generated/artifacts/provenance/kcr_trend.jsonl",
    "!.decapod/generated/artifacts/custody/",
    "!.decapod/generated/artifacts/custody/*.md",
    "!.decapod/generated/specs/",
    "!.decapod/generated/specs/*.md",
    "!.decapod/generated/specs/.manifest.json",
];

/// Ensure a given entry exists in the project's .gitignore file.
/// Creates the file if it doesn't exist. Appends the entry if not already present.
fn ensure_gitignore_entry(target_dir: &Path, entry: &str) -> Result<(), error::DecapodError> {
    let gitignore_path = target_dir.join(".gitignore");
    let content = fs::read_to_string(&gitignore_path).unwrap_or_default();

    // Check if the entry already exists (exact line match)
    if content.lines().any(|line| line.trim() == entry) {
        return Ok(());
    }

    let mut new_content = content;
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    new_content.push_str(entry);
    new_content.push('\n');
    fs::write(&gitignore_path, new_content).map_err(error::DecapodError::IoError)?;
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<(), error::DecapodError> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).map_err(error::DecapodError::IoError)?;
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
pub enum FileAction {
    Created,
    Unchanged,
    Preserved,
}

fn write_file(
    opts: &ScaffoldOptions,
    rel_path: &str,
    content: &str,
) -> Result<FileAction, error::DecapodError> {
    use sha2::{Digest, Sha256};

    let dest = opts.target_dir.join(rel_path);

    if dest.exists() {
        if let Ok(existing_content) = fs::read_to_string(&dest) {
            let mut template_hasher = Sha256::new();
            template_hasher.update(content.as_bytes());
            let template_hash = format!("{:x}", template_hasher.finalize());

            let mut existing_hasher = Sha256::new();
            existing_hasher.update(existing_content.as_bytes());
            let existing_hash = format!("{:x}", existing_hasher.finalize());

            if template_hash == existing_hash {
                return Ok(FileAction::Unchanged);
            }
        }

        if !opts.force {
            if opts.dry_run {
                return Ok(FileAction::Unchanged);
            }
            return Err(error::DecapodError::ValidationError(format!(
                "Refusing to overwrite existing path without --force: {}",
                dest.display()
            )));
        }
    }

    if opts.dry_run {
        return Ok(FileAction::Created);
    }

    ensure_parent(&dest)?;
    fs::write(&dest, content).map_err(error::DecapodError::IoError)?;

    Ok(FileAction::Created)
}

/// LegacyEntrypointContent holds the contents of backed-up agent entrypoint files.
///
/// These contents should be returned to the agent so it can manually consolidate
/// them into the appropriate sections of OVERRIDE.md (memory content into memory
/// sections, plugin content into plugin sections, etc.).
/// This is NOT auto-blended - the agent handles the consolidation.
#[derive(Debug, Default, serde::Serialize)]
pub struct LegacyEntrypointContent {
    pub agents_md: Option<String>,
    pub claude_md: Option<String>,
    pub gemini_md: Option<String>,
    pub entrypoint_md: Option<String>,
}

/// Read legacy agent entrypoint files (backed up to *.bak during init) and return their contents.
///
/// During `decapod init`, existing agent entrypoint files are backed up to *.bak.
/// This function reads those backups and returns their contents for the agent to process.
/// The agent should then manually blend the content into appropriate OVERRIDE.md sections.
pub fn get_legacy_entrypoint_contents(
    target_dir: &Path,
) -> Result<LegacyEntrypointContent, error::DecapodError> {
    let mut contents = LegacyEntrypointContent::default();

    for file in ["AGENTS.md", "CLAUDE.md", "GEMINI.md", "CODEX.md"] {
        let bak_path = target_dir.join(format!("{file}.bak"));
        if bak_path.exists()
            && let Ok(bak_content) = fs::read_to_string(&bak_path)
        {
            let trimmed = bak_content.trim();
            if !trimmed.is_empty() {
                match file {
                    "AGENTS.md" => contents.agents_md = Some(trimmed.to_string()),
                    "CLAUDE.md" => contents.claude_md = Some(trimmed.to_string()),
                    "GEMINI.md" => contents.gemini_md = Some(trimmed.to_string()),
                    "CODEX.md" => contents.entrypoint_md = Some(trimmed.to_string()),
                    _ => {}
                }
            }
        }
    }

    Ok(contents)
}

/// Delete legacy agent entrypoint backup files after agent has processed them.
pub fn cleanup_legacy_entrypoint_backups(target_dir: &Path) -> Result<(), error::DecapodError> {
    for file in ["AGENTS.md", "CLAUDE.md", "GEMINI.md", "CODEX.md"] {
        let bak_path = target_dir.join(format!("{file}.bak"));
        if bak_path.exists() {
            let _ = fs::remove_file(&bak_path);
        }
    }
    Ok(())
}

/// Blend new constitution sections into existing OVERRIDE.md.
pub fn blend_overrides(target_dir: &Path) -> Result<FileAction, error::DecapodError> {
    let override_path = target_dir.join(".decapod").join("OVERRIDE.md");
    if !override_path.exists() {
        return Ok(FileAction::Unchanged);
    }

    let existing_content =
        fs::read_to_string(&override_path).map_err(error::DecapodError::IoError)?;
    let template = assets::get_template("OVERRIDE.md").expect("Missing template: OVERRIDE.md");

    // Extract H3 headers from existing content
    let existing_headers: std::collections::HashSet<String> = existing_content
        .lines()
        .filter(|line| line.starts_with("### "))
        .map(|line| line.trim().to_string())
        .collect();

    // Find missing sections in template
    let mut missing_lines = Vec::new();
    let mut current_cat = String::new();
    let mut cat_emitted = std::collections::HashSet::new();

    for line in template.lines() {
        if line.starts_with("## ") {
            current_cat = line.to_string();
        } else if line.starts_with("### ") {
            let trimmed = line.trim();
            if !existing_headers.contains(trimmed) {
                if !cat_emitted.contains(&current_cat) && !current_cat.is_empty() {
                    // Check if category already exists in file
                    if !existing_content.contains(&current_cat) {
                        missing_lines.push(format!("\n{current_cat}"));
                    }
                    cat_emitted.insert(current_cat.clone());
                }
                missing_lines.push(line.to_string());
            }
        }
    }

    if missing_lines.is_empty() {
        return Ok(FileAction::Unchanged);
    }

    let mut updated_content = existing_content;
    if !updated_content.ends_with('\n') {
        updated_content.push('\n');
    }
    for line in missing_lines {
        updated_content.push_str(&line);
        updated_content.push('\n');
    }

    fs::write(&override_path, updated_content).map_err(error::DecapodError::IoError)?;
    Ok(FileAction::Created)
}

pub fn scaffold_project_entrypoints(
    opts: &ScaffoldOptions,
) -> Result<ScaffoldSummary, error::DecapodError> {
    eprintln!("DEBUG: target_dir: {:?}", opts.target_dir);
    let data_dir_rel = ".decapod/data";

    // Ensure .decapod/data directory exists (constitution is embedded, not scaffolded)
    fs::create_dir_all(opts.target_dir.join(data_dir_rel)).map_err(error::DecapodError::IoError)?;

    // Ensure Decapod-managed ignore/allowlist rules are present in the user's .gitignore.
    if !opts.dry_run {
        for rule in DECAPOD_GITIGNORE_RULES {
            ensure_gitignore_entry(&opts.target_dir, rule)?;
        }
    }

    // Determine which agent files to generate
    // If --all flag is set, force generate all five regardless of existing state
    // If agent_files is empty, generate all five
    // If agent_files has entries, only generate those
    let files_to_generate = if opts.all || opts.agent_files.is_empty() {
        vec!["AGENTS.md", "CLAUDE.md", "GEMINI.md", "CODEX.md"]
    } else {
        opts.agent_files.iter().map(|s| s.as_str()).collect()
    };

    // Root entrypoints from embedded templates
    let readme_md = assets::get_template("README.md").expect("Missing template: README.md");
    let override_md = assets::get_template("OVERRIDE.md").expect("Missing template: OVERRIDE.md");

    // AGENT ENTRYPOINTS - Neural Interfaces (only generate specified files)
    let mut ep_created = 0usize;
    let mut ep_unchanged = 0usize;
    let mut ep_preserved = 0usize;
    for file in files_to_generate {
        let content =
            assets::get_template(file).unwrap_or_else(|| panic!("Missing template: {file}"));
        match write_file(opts, file, &content)? {
            FileAction::Created => ep_created += 1,
            FileAction::Unchanged => ep_unchanged += 1,
            FileAction::Preserved => ep_preserved += 1,
        }
    }

    let mut cfg_created = 0usize;
    let mut cfg_unchanged = 0usize;
    let mut cfg_preserved = 0usize;

    match write_file(opts, ".decapod/README.md", &readme_md)? {
        FileAction::Created => cfg_created += 1,
        FileAction::Unchanged => cfg_unchanged += 1,
        FileAction::Preserved => cfg_preserved += 1,
    }

    // Blend into existing OVERRIDE.md or create new one
    let override_path = opts.target_dir.join(".decapod/OVERRIDE.md");
    if override_path.exists() {
        cfg_preserved += 1;
    } else {
        match write_file(opts, ".decapod/OVERRIDE.md", &override_md)? {
            FileAction::Created => cfg_created += 1,
            FileAction::Unchanged => cfg_unchanged += 1,
            FileAction::Preserved => cfg_preserved += 1,
        }
    }

    // CI Scaffolding - GitHub Action
    let mut ci_created = 0usize;
    let mut ci_unchanged = 0usize;
    let mut ci_preserved = 0usize;

    if opts.generate_ci {
        let github_workflow_rel = ".github/workflows/decapod-validate.yml";
        let github_workflow_content = assets::get_template("decapod-validate.yml")
            .expect("Missing template: decapod-validate.yml");

        match write_file(opts, github_workflow_rel, &github_workflow_content)? {
            FileAction::Created => ci_created += 1,
            FileAction::Unchanged => ci_unchanged += 1,
            FileAction::Preserved => ci_preserved += 1,
        }
    }

    // Legacy agent entrypoint backups (*.bak) are NOT auto-blended here.
    // Use get_legacy_entrypoint_contents() to read them and return to the agent.
    // The agent will manually consolidate content into appropriate OVERRIDE.md sections.

    // Generate .decapod/generated/Dockerfile from Rust-owned template component.
    let generated_dir = opts.target_dir.join(".decapod/generated");
    fs::create_dir_all(&generated_dir).map_err(error::DecapodError::IoError)?;
    fs::create_dir_all(generated_dir.join("context")).map_err(error::DecapodError::IoError)?;
    fs::create_dir_all(generated_dir.join("policy")).map_err(error::DecapodError::IoError)?;
    fs::create_dir_all(generated_dir.join("artifacts").join("provenance"))
        .map_err(error::DecapodError::IoError)?;
    fs::create_dir_all(generated_dir.join("artifacts").join("custody"))
        .map_err(error::DecapodError::IoError)?;
    let custody_readme_path = generated_dir
        .join("artifacts")
        .join("custody")
        .join("README.md");
    if !custody_readme_path.exists() {
        let custody_readme_content = r#"# Epistemic Custody Artifacts

This directory tracks the preserved chain of intent, context, assumptions, and proof for this repository.

## Directory Structure
- `assumptions.md`: Log of active and verified assumptions.
- `contradictions.md`: Log of evidence that conflicts with current plans or assumptions.
- `deferred_questions.md`: Questions identified during work that were postponed.
- `evidence/`: Detailed proof artifacts (logs, screenshots, data captures) tied to specific claims.

## Agent Guidance
Agents operating in this repo MUST maintain these artifacts to ensure long-horizon integrity. Do not compress away uncertainty; surface it here so it remains inspectable by humans and future agent passes.
"#;
        fs::write(&custody_readme_path, custody_readme_content)
            .map_err(error::DecapodError::IoError)?;
    }
    fs::create_dir_all(generated_dir.join("artifacts").join("inventory"))
        .map_err(error::DecapodError::IoError)?;
    fs::create_dir_all(
        generated_dir
            .join("artifacts")
            .join("diagnostics")
            .join("validate"),
    )
    .map_err(error::DecapodError::IoError)?;
    fs::create_dir_all(generated_dir.join("migrations")).map_err(error::DecapodError::IoError)?;
    let dockerfile_path = generated_dir.join("Dockerfile");
    if !dockerfile_path.exists() {
        let dockerfile_content = container::generated_dockerfile_for_repo(&opts.target_dir);
        fs::write(&dockerfile_path, dockerfile_content).map_err(error::DecapodError::IoError)?;
    }
    let version_counter_path = generated_dir.join("version_counter.json");
    if !version_counter_path.exists() {
        let now = crate::core::time::now_epoch_z();
        let version_counter = serde_json::json!({
            "schema_version": "1.0.0",
            "version_count": 1,
            "initialized_with_version": env!("CARGO_PKG_VERSION"),
            "last_seen_version": env!("CARGO_PKG_VERSION"),
            "updated_at": now,
        });
        let body = serde_json::to_string_pretty(&version_counter).map_err(|e| {
            error::DecapodError::ValidationError(format!(
                "Failed to serialize version counter: {e}"
            ))
        })?;
        fs::write(version_counter_path, body).map_err(error::DecapodError::IoError)?;
    }

    let generated_policy_path = opts.target_dir.join(GENERATED_POLICY_REL_PATH);
    if !generated_policy_path.exists() {
        let policy_body = default_policy_json_pretty()?;
        fs::write(generated_policy_path, policy_body).map_err(error::DecapodError::IoError)?;
    }

    let (specs_created, specs_unchanged, specs_preserved) = if opts.generate_specs {
        let mut created = 0usize;
        let mut unchanged = 0usize;
        let mut preserved = 0usize;
        let mut manifest_entries: Vec<ProjectSpecManifestEntry> = Vec::new();

        let seed = opts.specs_seed.as_ref();
        let mut specs_files: Vec<(&str, String)> = Vec::new();
        for spec in LOCAL_PROJECT_SPECS {
            let Some(mut content) =
                render_project_spec_content(spec.path, opts.diagram_style, seed)
            else {
                continue;
            };

            // Respect component-specific override in .decapod/OVERRIDE.md if present
            if let Some(override_content) =
                assets::get_override_doc(&opts.target_dir, spec.constitution_ref)
            {
                content = assets::merge_override_content(&content, &override_content);
            }

            specs_files.push((spec.path, content));
        }

        for (rel_path, mut content) in specs_files {
            // Epistemic Custody Preservation:
            // If we are regenerating INTENT.md and it already exists, try to preserve the Epistemic Custody Fields section.
            content = preserve_intent_custody(&opts.target_dir, rel_path, content);

            let template_hash = project_spec_scaffold_hash(rel_path, opts.diagram_style)
                .unwrap_or_else(|| hash_text(&content));
            match write_file(opts, rel_path, &content)? {
                FileAction::Created => created += 1,
                FileAction::Unchanged => unchanged += 1,
                FileAction::Preserved => preserved += 1,
            }
            manifest_entries.push(ProjectSpecManifestEntry {
                path: rel_path.to_string(),
                template_hash,
                content_hash: hash_text(&content),
            });
        }

        if !opts.dry_run {
            let manifest = ProjectSpecsManifest {
                schema_version: LOCAL_PROJECT_SPECS_MANIFEST_SCHEMA.to_string(),
                template_version: PROJECT_SPEC_TEMPLATE_VERSION.to_string(),
                generated_at: crate::core::time::now_epoch_z(),
                repo_signal_fingerprint: repo_signal_fingerprint(&opts.target_dir)?,
                files: manifest_entries,
            };
            let manifest_path = opts.target_dir.join(LOCAL_PROJECT_SPECS_MANIFEST);
            ensure_parent(&manifest_path)?;
            let manifest_body = serde_json::to_string_pretty(&manifest).map_err(|e| {
                error::DecapodError::ValidationError(format!(
                    "Failed to serialize specs manifest: {e}"
                ))
            })?;
            fs::write(manifest_path, manifest_body).map_err(error::DecapodError::IoError)?;
        }
        (created, unchanged, preserved)
    } else {
        (0usize, 0usize, 0usize)
    };

    Ok(ScaffoldSummary {
        entrypoints_created: ep_created,
        entrypoints_unchanged: ep_unchanged,
        entrypoints_preserved: ep_preserved,
        config_created: cfg_created,
        config_unchanged: cfg_unchanged,
        config_preserved: cfg_preserved,
        specs_created,
        specs_unchanged,
        specs_preserved,
        ci_created,
        ci_unchanged,
        ci_preserved,
    })
}
