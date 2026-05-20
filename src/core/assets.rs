//! Embedded constitution and template assets.
//!
//! This module provides compile-time embedded access to Decapod's methodology documents.
//! All constitution files (core, specs, plugins) are baked into the binary for
//! hermetic deployment - no external files required.

use std::path::Path;

/// Macro to embed constitution documents at compile time as text.
///
/// Generates:
/// - Public constants for each embedded document
/// - `get_embedded_doc(path)` function for lookup
/// - `list_docs()` function for discovery
macro_rules! embedded_docs {
    ($($path:expr => $const_name:ident),* $(,)?) => {
        $(
            pub const $const_name: &str =
                include_str!(concat!("../../constitution/", $path));
        )*

        pub fn get_embedded_doc(path: &str) -> Option<String> {
            // Support both bare paths and legacy "embedded/" prefix
            let key = path.strip_prefix("embedded/").unwrap_or(path);
            match key {
                $( $path => Some($const_name.to_string()), )*
                _ => None,
            }
        }

        pub fn list_docs() -> Vec<String> {
            vec![ $( $path.to_string(), )* ]
        }
    };
}

embedded_docs! {
    // Core: Routers and indices
    "core/ENGINEERING_EXCELLENCE.md" => EMBEDDED_CORE_ENGINEERING_EXCELLENCE,
    "core/DECAPOD.md" => EMBEDDED_CORE_DECAPOD,
    "core/INTERFACES.md" => EMBEDDED_CORE_INTERFACES,
    "core/METHODOLOGY.md" => EMBEDDED_CORE_METHODOLOGY,
    "core/PLUGINS.md" => EMBEDDED_CORE_PLUGINS,
    "core/GAPS.md" => EMBEDDED_CORE_GAPS,
    "core/DEMANDS.md" => EMBEDDED_CORE_DEMANDS,
    "core/DEPRECATION.md" => EMBEDDED_CORE_DEPRECATION,

    // Specs: System contracts
    "specs/INTENT.md" => EMBEDDED_SPECS_INTENT,
    "specs/SYSTEM.md" => EMBEDDED_SPECS_SYSTEM,
    "specs/AMENDMENTS.md" => EMBEDDED_SPECS_AMENDMENTS,
    "specs/SECURITY.md" => EMBEDDED_SPECS_SECURITY,
    "specs/GIT.md" => EMBEDDED_SPECS_GIT,
    "specs/evaluations/VARIANCE_EVALS.md" => EMBEDDED_SPECS_VARIANCE_EVALS,
    "specs/evaluations/JUDGE_CONTRACT.md" => EMBEDDED_SPECS_JUDGE_CONTRACT,
    "specs/engineering/FRONTEND_BACKEND_E2E.md" => EMBEDDED_SPECS_FRONTEND_BACKEND_E2E,
    "specs/skills/SKILL_GOVERNANCE.md" => EMBEDDED_SPECS_SKILL_GOVERNANCE,

    // Interfaces: Binding contracts
    "interfaces/CLAIMS.md" => EMBEDDED_INTERFACES_CLAIMS,
    "interfaces/CONTROL_PLANE.md" => EMBEDDED_INTERFACES_CONTROL_PLANE,
    "interfaces/DOC_RULES.md" => EMBEDDED_INTERFACES_DOC_RULES,
    "interfaces/GLOSSARY.md" => EMBEDDED_INTERFACES_GLOSSARY,
    "interfaces/STORE_MODEL.md" => EMBEDDED_INTERFACES_STORE_MODEL,
    "interfaces/TESTING.md" => EMBEDDED_INTERFACES_TESTING,
    "interfaces/KNOWLEDGE_SCHEMA.md" => EMBEDDED_INTERFACES_KNOWLEDGE_SCHEMA,
    "interfaces/KNOWLEDGE_STORE.md" => EMBEDDED_INTERFACES_KNOWLEDGE_STORE,
    "interfaces/MEMORY_SCHEMA.md" => EMBEDDED_INTERFACES_MEMORY_SCHEMA,
    "interfaces/DEMANDS_SCHEMA.md" => EMBEDDED_INTERFACES_DEMANDS_SCHEMA,
    "interfaces/TODO_SCHEMA.md" => EMBEDDED_INTERFACES_TODO_SCHEMA,
    "interfaces/PLAN_GOVERNED_EXECUTION.md" => EMBEDDED_INTERFACES_PLAN_GOVERNED_EXECUTION,
    "interfaces/AGENT_CONTEXT_PACK.md" => EMBEDDED_INTERFACES_AGENT_CONTEXT_PACK,

    // Methodology: Practice guides
    "methodology/ARCHITECTURE.md" => EMBEDDED_METHODOLOGY_ARCHITECTURE,
    "methodology/SOUL.md" => EMBEDDED_METHODOLOGY_SOUL,
    "methodology/KNOWLEDGE.md" => EMBEDDED_METHODOLOGY_KNOWLEDGE,
    "methodology/MEMORY.md" => EMBEDDED_METHODOLOGY_MEMORY,
    "methodology/TESTING.md" => EMBEDDED_METHODOLOGY_TESTING,
    "methodology/CI_CD.md" => EMBEDDED_METHODOLOGY_CI_CD,

    // Architecture: Domain patterns
    "architecture/ALGORITHMS.md" => EMBEDDED_ARCHITECTURE_ALGORITHMS,
    "architecture/API_DESIGN.md" => EMBEDDED_ARCHITECTURE_API_DESIGN,
    "architecture/AUTH.md" => EMBEDDED_ARCHITECTURE_AUTH,
    "architecture/CACHING.md" => EMBEDDED_ARCHITECTURE_CACHING,
    "architecture/CI_CD_PIPELINES.md" => EMBEDDED_ARCHITECTURE_CI_CD_PIPELINES,
    "architecture/CLOUD.md" => EMBEDDED_ARCHITECTURE_CLOUD,
    "architecture/CODING_STANDARDS.md" => EMBEDDED_ARCHITECTURE_CODING_STANDARDS,
    "architecture/COMPLIANCE.md" => EMBEDDED_ARCHITECTURE_COMPLIANCE,
    "architecture/CONCURRENCY.md" => EMBEDDED_ARCHITECTURE_CONCURRENCY,
    "architecture/CONTAINERS.md" => EMBEDDED_ARCHITECTURE_CONTAINERS,
    "architecture/COST_OPTIMIZATION.md" => EMBEDDED_ARCHITECTURE_COST_OPTIMIZATION,
    "architecture/DATA.md" => EMBEDDED_ARCHITECTURE_DATA,
    "architecture/DATABASE.md" => EMBEDDED_ARCHITECTURE_DATABASE,
    "architecture/DISTRIBUTED_SYSTEMS.md" => EMBEDDED_ARCHITECTURE_DISTRIBUTED_SYSTEMS,
    "architecture/DR.md" => EMBEDDED_ARCHITECTURE_DR,
    "architecture/ENCRYPTION.md" => EMBEDDED_ARCHITECTURE_ENCRYPTION,
    "architecture/EVENT_DRIVEN.md" => EMBEDDED_ARCHITECTURE_EVENT_DRIVEN,
    "architecture/FRONTEND.md" => EMBEDDED_ARCHITECTURE_FRONTEND,
    "architecture/GRAPHQL.md" => EMBEDDED_ARCHITECTURE_GRAPHQL,
    "architecture/GRPC.md" => EMBEDDED_ARCHITECTURE_GRPC,
    "architecture/INFRASTRUCTURE.md" => EMBEDDED_ARCHITECTURE_INFRASTRUCTURE,
    "architecture/KNOWLEDGE_BASE.md" => EMBEDDED_ARCHITECTURE_KNOWLEDGE_BASE,
    "architecture/KUBERNETES.md" => EMBEDDED_ARCHITECTURE_KUBERNETES,
    "architecture/MEMORY.md" => EMBEDDED_ARCHITECTURE_MEMORY,
    "architecture/MESSAGING.md" => EMBEDDED_ARCHITECTURE_MESSAGING,
    "architecture/METRICS.md" => EMBEDDED_ARCHITECTURE_METRICS,
    "architecture/MICROSERVICES.md" => EMBEDDED_ARCHITECTURE_MICROSERVICES,
    "architecture/NETWORKING.md" => EMBEDDED_ARCHITECTURE_NETWORKING,
    "architecture/OBSERVABILITY.md" => EMBEDDED_ARCHITECTURE_OBSERVABILITY,
    "architecture/PERFORMANCE.md" => EMBEDDED_ARCHITECTURE_PERFORMANCE,
    "architecture/SCALING.md" => EMBEDDED_ARCHITECTURE_SCALING,
    "architecture/SECRETS.md" => EMBEDDED_ARCHITECTURE_SECRETS,
    "architecture/SECURITY.md" => EMBEDDED_ARCHITECTURE_SECURITY,
    "architecture/TESTING_STRATEGY.md" => EMBEDDED_ARCHITECTURE_TESTING_STRATEGY,
    "architecture/UI.md" => EMBEDDED_ARCHITECTURE_UI,
    "architecture/WEB.md" => EMBEDDED_ARCHITECTURE_WEB,

    // Embedded docs used by entrypoints/operators
    "docs/ARCHITECTURE_OVERVIEW.md" => EMBEDDED_DOCS_ARCHITECTURE_OVERVIEW,
    "docs/CONTROL_PLANE_API.md" => EMBEDDED_DOCS_CONTROL_PLANE_API,
    "docs/MAINTAINERS.md" => EMBEDDED_DOCS_MAINTAINERS,
    "docs/MIGRATIONS.md" => EMBEDDED_DOCS_MIGRATIONS,
    "docs/NEGLECTED_ASPECTS_LEDGER.md" => EMBEDDED_DOCS_NEGLECTED_ASPECTS_LEDGER,
    "docs/PLAYBOOK.md" => EMBEDDED_DOCS_PLAYBOOK,
    "docs/README.md" => EMBEDDED_DOCS_README,
    "docs/RELEASE_PROCESS.md" => EMBEDDED_DOCS_RELEASE_PROCESS,
    "docs/SECURITY_THREAT_MODEL.md" => EMBEDDED_DOCS_SECURITY_THREAT_MODEL,
    "docs/EVAL_TRANSLATION_MAP.md" => EMBEDDED_DOCS_EVAL_TRANSLATION_MAP,
    "docs/SKILL_TRANSLATION_MAP.md" => EMBEDDED_DOCS_SKILL_TRANSLATION_MAP,

    "plugins/ARCHIVE.md" => EMBEDDED_PLUGINS_ARCHIVE,
    "plugins/AUTOUPDATE.md" => EMBEDDED_PLUGINS_AUTOUPDATE,
    "plugins/CONTEXT.md" => EMBEDDED_PLUGINS_CONTEXT,
    "plugins/CRON.md" => EMBEDDED_PLUGINS_CRON,
    "plugins/DB_BROKER.md" => EMBEDDED_PLUGINS_DB_BROKER,
    "plugins/DECIDE.md" => EMBEDDED_PLUGINS_DECIDE,
    "plugins/EMERGENCY_PROTOCOL.md" => EMBEDDED_PLUGINS_EMERGENCY_PROTOCOL,
    "plugins/FEDERATION.md" => EMBEDDED_PLUGINS_FEDERATION,
    "plugins/FEEDBACK.md" => EMBEDDED_PLUGINS_FEEDBACK,
    "plugins/HEALTH.md" => EMBEDDED_PLUGINS_HEALTH,
    "plugins/HEARTBEAT.md" => EMBEDDED_PLUGINS_HEARTBEAT,
    "plugins/KNOWLEDGE.md" => EMBEDDED_PLUGINS_KNOWLEDGE,
    "plugins/MANIFEST.md" => EMBEDDED_PLUGINS_MANIFEST,
    "plugins/POLICY.md" => EMBEDDED_PLUGINS_POLICY,
    "plugins/REFLEX.md" => EMBEDDED_PLUGINS_REFLEX,
    "plugins/APTITUDE.md" => EMBEDDED_PLUGINS_APTITUDE,
    "plugins/TODO.md" => EMBEDDED_PLUGINS_TODO,
    "plugins/TRUST.md" => EMBEDDED_PLUGINS_TRUST,
    "plugins/VERIFY.md" => EMBEDDED_PLUGINS_VERIFY,
    "plugins/WATCHER.md" => EMBEDDED_PLUGINS_WATCHER,
}

/// Legacy function - now just forwards to get_embedded_doc
pub fn get_doc(path: &str) -> Option<String> {
    get_embedded_doc(path)
}

/// Get only the override document from .decapod/OVERRIDE.md for a specific component
pub fn get_override_doc(repo_root: &Path, relative_path: &str) -> Option<String> {
    let override_path = repo_root.join(".decapod").join("OVERRIDE.md");

    if !override_path.exists() {
        return None;
    }

    let override_content = std::fs::read_to_string(&override_path).ok()?;
    extract_component_override(&override_content, relative_path)
}

/// List component override section headings from .decapod/OVERRIDE.md.
pub fn list_override_sections(repo_root: &Path) -> Vec<String> {
    let override_path = repo_root.join(".decapod").join("OVERRIDE.md");
    let Ok(override_content) = std::fs::read_to_string(&override_path) else {
        return Vec::new();
    };

    extract_override_section_names(&override_content)
}

fn extract_override_section_names(override_content: &str) -> Vec<String> {
    let Some(override_start) = override_content.find("CHANGES ARE NOT PERMITTED ABOVE THIS LINE")
    else {
        return Vec::new();
    };
    let searchable_content = &override_content[override_start..];

    searchable_content
        .lines()
        .filter_map(|line| line.strip_prefix("### "))
        .map(str::trim)
        .filter(|section| !section.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// Extract a specific component's override content from OVERRIDE.md
fn extract_component_override(override_content: &str, component_path: &str) -> Option<String> {
    // Only look after the "CHANGES ARE NOT PERMITTED ABOVE THIS LINE" marker
    let override_start = override_content.find("CHANGES ARE NOT PERMITTED ABOVE THIS LINE")?;
    let searchable_content = &override_content[override_start..];

    // Look for the section heading: ### core/DECAPOD.md (or other path)
    let section_marker = format!("### {}", component_path);

    let start = searchable_content.find(&section_marker)?;

    // Ensure it's a real header (either at start of searchable_content or after a newline)
    if start > 0 && searchable_content.as_bytes()[start - 1] != b'\n' {
        // This is a partial match inside a line, not a header.
        return None;
    }

    let content_start = start + section_marker.len();

    // Find the next ### heading or end of file
    let content_after = &searchable_content[content_start..];
    let end = content_after
        .find("\n### ")
        .map(|pos| content_start + pos)
        .unwrap_or(searchable_content.len());

    let extracted = searchable_content[content_start..end].trim();

    if extracted.is_empty() {
        None
    } else {
        Some(extracted.to_string())
    }
}

/// Get merged document (embedded base + optional project override from OVERRIDE.md)
pub fn get_merged_doc(repo_root: &Path, relative_path: &str) -> Option<String> {
    // Get embedded base
    let embedded_content = get_embedded_doc(relative_path)?;

    // Check for component-specific override in .decapod/OVERRIDE.md
    if let Some(override_content) = get_override_doc(repo_root, relative_path) {
        return Some(merge_override_content(&embedded_content, &override_content));
    }

    Some(embedded_content)
}

/// Merge embedded content with override additions
fn merge_override_content(embedded_content: &str, override_content: &str) -> String {
    format!(
        "{}\n\n---\n\n## Project Overrides\n\n{}",
        embedded_content.trim(),
        override_content.trim()
    )
}

fn agent_entrypoint_body() -> &'static str {
    r#"You are working in a Decapod-managed repository.
See `AGENTS.md` for the universal contract.

## Project Context

- Read `.decapod/config.toml` before planning; it captures project name, summary, architecture, primary languages, and entrypoint preferences.
- Treat `.decapod/config.toml` as human-editable project context. You may update it when user intent or project direction changes.
- Read `.decapod/OVERRIDE.md` when present; it is the repo-local place for constitution overrides.
- Do not mutate Decapod-owned state under `.decapod/` directly; use Decapod CLI surfaces for generated specs, data, workspaces, and sessions.

## Quick Start

```bash
cargo install decapod
decapod validate && decapod docs ingest && decapod session acquire
decapod rpc --op agent.init
decapod workspace status
decapod todo add "<task>" && decapod todo claim --id <task-id>
decapod infer orientation --task-id <task-id>
decapod workspace ensure
cd .decapod/workspaces/<your-worktree>
decapod rpc --op context.resolve
```

## Control-Plane First

```bash
decapod capabilities --format json
decapod rpc --op context.scope --params '{"query":"<problem>","limit":8}'
decapod data schema --deterministic
```

## Operating Mode

- Use Docker git workspaces and execute in `.decapod/workspaces/*`.
- Call `decapod workspace status` at startup and before implementation work.
- request elevated permissions before Docker/container workspace commands.
- `.decapod files are accessed only via decapod CLI`.
- Read and update `.decapod/config.toml` as project context; use Decapod CLI for other `.decapod/` state.
- Read `.decapod/OVERRIDE.md` for repo-local constitution overrides when present.
- `DECAPOD_SESSION_PASSWORD` is required for session-scoped operations.
- Read canonical router: `decapod docs show core/DECAPOD.md`.
- Use shared aptitude memory for human-taught preferences across sessions/providers: `decapod data memory add|get` (aliases: `decapod data aptitude`).
- Operator reference: `decapod docs show docs/PLAYBOOK.md`.
- Capability authority: `decapod capabilities --format json`.
- Scoped context feature: `decapod docs search --query "<problem>" --op <op>` or `decapod rpc --op context.scope`.

Stop if requirements are ambiguous or conflicting.
"#
}

fn template_agents() -> String {
    r#"# AGENTS.md — Universal Agent Contract

This is a Decapod-managed repository. **Strict Dependency: You are strictly bound to the Decapod control plane.**

This contract applies equally to Claude, Codex, Gemini, Cursor, Kilo, and any other agent operating here.

## Mandatory Initialization

```bash
cargo install decapod
decapod validate && decapod docs ingest && decapod session acquire
decapod rpc --op agent.init
decapod workspace status
decapod todo add "<task>" && decapod todo claim --id <task-id>
decapod infer orientation --task-id <task-id>
decapod workspace ensure
cd .decapod/workspaces/<your-worktree>
decapod rpc --op context.resolve
```

## Control-Plane First Loop

```bash
# Discover what this binary actually supports in this repo
decapod capabilities --format json
decapod data schema --deterministic

# Resolve precise orientation before implementation
decapod infer orientation --intent "<your-goal>" --task-id <id>
decapod govern capsule query --topic "<topic>" --scope interfaces --task-id <task-id>
decapod rpc --op context.scope --params '{"query":"<problem>","limit":8}'
```

## Golden Rules (Non-Negotiable)

1. **MUST** refine intent with the user before inference-heavy work.
2. **MUST** use `decapod infer orientation` before non-trivial implementation.
3. **MUST** stop and ask the human when Decapod emits a **Decision Gate**.
4. **MUST NOT** work on main/master or modify the root repository's active branch. **MUST** use `decapod workspace ensure` and `cd .decapod/workspaces/<worktree>`.
5. **MUST** read [.decapod/config.toml](.decapod/config.toml) as user-editable project context and may update it when user intent changes.
6. **MUST NOT** claim done without `decapod validate` passing.
7. **MUST NOT** invent capabilities that are not exposed by the binary.
8. **MUST** stop if requirements conflict, intent is ambiguous, or policy boundaries are unclear.
9. **MUST** respect the Interface abstraction boundary.

## Decapod Invocation Contract

Agents act. Decapod orients.

Decapod is not your executor, model runtime, or workflow replacement. You remain responsible for implementation. Call Decapod as the repo-native pressure relief valve when the next responsible step requires explicit intent, boundaries, context, coordination, or proof.

Call Decapod before proceeding when continuing would require guessing about:
- **Intent pressure:** what you are actually trying to do.
- **Boundary pressure:** what you are allowed to touch.
- **Context pressure:** what matters right now.
- **Coordination pressure:** whether this collides with other work.
- **Proof pressure:** what evidence makes this complete.
- **Completion pressure:** whether you can truthfully claim done.

Concrete triggers: ambiguous requests, public behavior/security/data/migration/generated/release/architecture impact, unclear proof, todo create/update/split/complete, scope expansion, conflicting intent/specs/instructions/repo state, context loss, multi-agent collision risk, or readiness to claim completion.

Do not call Decapod for every trivial file read, local edit, or mechanical command. Call it at decision boundaries that need governance, memory, boundaries, coordination, or proof. Decapod calls should produce or update explicit artifacts: intent, context, constraints, todos, decisions, proof, and completion state.

When using `decapod infer orientation`, treat the returned packet as starting context; stop on decision gates; use `allowed_scope` and `proof_required` to bound work.

## Invariants (Normative)

These invariants are directly enforced by tests. Violations will cause CI failure.

- **INV-DAEMONLESS**: Decapod MUST NOT leave background processes running. (enforced by `tests/daemonless_lifecycle.rs`)
- **INV-BOUNDED-VALIDATE**: `decapod validate` MUST terminate within bounded time. (enforced by `tests/validate_termination.rs`)
- **INV-STORE-BOUNDARY**: Agents MUST NOT directly mutate `.decapod/*`; all access MUST use CLI. (enforced by validation gates)
- **INV-SESSION-AUTH**: Mutations require active session with valid credentials. (enforced by session commands)
- **INV-PROOF-GATED**: Workunit status `VERIFIED` MUST have passed proof-plan gates. (enforced by `tests/workunit_publish_gate.rs`)
- **INV-ROOT-ISOLATION**: Agents MUST NOT check out branches or mutate files in the main repository checkout. All work must happen in isolated `.decapod/workspaces/*` worktrees to avoid disrupting the human user's environment. (enforced by workspace validation)

## Safety Invariants
- ✅ Router pointer: `core/DECAPOD.md` | ✅ Validation gate: `decapod validate`
- ✅ Constitution ingestion gate: `decapod docs ingest`
- ✅ Workspace status gate: `decapod workspace status`
- ✅ Claim-before-work gate: `decapod todo claim --id <task-id>`
- ✅ Session auth gate: `DECAPOD_SESSION_PASSWORD`
- ✅ Workspace gate: Docker git workspaces
- ✅ Privilege gate: request elevated permissions before Docker/container workspace commands

## Operating Notes

- Read `.decapod/config.toml` (human-editable) for project context and architecture direction.
- Read `.decapod/OVERRIDE.md` for repo-local constitution overrides.
- DO NOT mutate `.decapod/` state directly; use Decapod CLI for specs, data, workspaces, and sessions. Access to `.decapod/` is strictly via decapod CLI.
- Use `decapod docs show core/DECAPOD.md` for binding contracts.
- Use `decapod capabilities --format json` to discover available operations.
- Stop if requirements conflict, intent is ambiguous, or policy boundaries are unclear.
- Respect the Interface abstraction boundary.
- Treat lock/contention failures as blocking until resolved.
"#
        .to_string()
}

fn template_named_agent(file_stem: &str) -> String {
    format!(
        "# {}.md - Agent Entrypoint\n\n{}",
        file_stem,
        agent_entrypoint_body()
    )
}

fn template_readme() -> String {
    r#"# .decapod - Decapod Control Plane

Decapod is the daemonless, local-first governance kernel behind AI coding agents. Agents call it on demand to turn intent into context, then context into explicit specifications before inference, enforce boundaries, and deliver proof-backed completion across concurrent multi-agent work.

GitHub: https://github.com/DecapodLabs/decapod
Canonical Contract: [constitution/core/DECAPOD.md](constitution/core/DECAPOD.md)

## What This Directory Is

This `.decapod/` directory is the local control plane for this repository.
It keeps Decapod-owned state, generated artifacts, and isolated workspaces separate from your product source tree.

`OVERRIDE.md` and `README.md` intentionally stay at this top level.

## Quick Start

1. `decapod init`
2. `decapod validate`
3. `decapod docs ingest`
4. `decapod session acquire`
5. `decapod rpc --op agent.init`
6. `decapod workspace status`
7. `decapod todo add \"<task>\" && decapod todo claim --id <task-id>`
8. `decapod workspace ensure`

## Skills - Your Personal Optimization Layer

**Skills are how you shape agent behavior.** Import skills to train agents how to interact with your codebase, your conventions, and your preferences.

### Why Skills Matter

- **Controls**: Add security reviews, code quality checks, or custom validation
- **Optimization**: Encode your team's conventions, patterns, and best practices
- **Context**: Give agents project-specific knowledge that persists across sessions

### Quick Skills Workflow

```bash
# Import a skill from a SKILL.md file
decapod data aptitude skill import --path path/to/your/SKILL.md

# List available skills
decapod data aptitude skill list

# Resolve skills for a specific task
decapod data aptitude skill resolve --query "how to write tests"

# Query aptitude memory for learned preferences
decapod data aptitude prompt --query "git"
```

### Creating Your Own Skills

Skills are just Markdown files with YAML frontmatter:

```yaml
---
name: my-security-review
description: Custom security checks for our codebase
allowed-tools: Bash
---

# Security Review Skill

## Triggers
- "check security"
- "review for vulnerabilities"

## Workflow
1. Run `semgrep --config=auto .`
2. Check for hardcoded secrets
3. Validate dependency vulnerabilities
4. Report findings
```

Place SKILL.md files in `constitution/metadata/skills/` and import them:

```bash
decapod data aptitude skill import --path constitution/metadata/skills/my-security-review/SKILL.md
```

### Aptitude Memory

Decapod learns from interactions. Use aptitude to record preferences:

```bash
# Record a preference
decapod data aptitude add --category git --key branch_prefix --value "feature/" --confidence 90

# Get contextual prompts
decapod data aptitude prompt --query "commit"

# Record an observation
decapod data aptitude observe --category code_style --content "Team prefers async/await over tokio::spawn"
```

## Canonical Layout

- `README.md`: operator onboarding and control-plane map.
- `OVERRIDE.md`: project-local override layer for embedded constitution.
- `data/`: canonical control-plane state (SQLite + ledgers).
- `skills/`: imported skill cards (auto-generated, tracked for reproducibility).
- `generated/specs/`: living project specs scaffolded by `decapod init`.
- `generated/context/`: deterministic context capsule artifacts.
- `generated/artifacts/provenance/`: promotion manifests and convergence checklist.
- `generated/artifacts/inventory/`: deterministic release inventory artifacts.
- `generated/artifacts/diagnostics/`: opt-in diagnostics artifacts.
- `workspaces/`: isolated todo-scoped git worktrees for implementation.

## Why Teams Use This

- Agent-first interface with explicit governance.
- Local-first execution without daemon overhead.
- Integrated TODO, claims, context, validation, and proof in one harness.
- Cleaner repos: Decapod concerns stay in `.decapod/`.

## Override Workflow

Edit `.decapod/OVERRIDE.md` to add project-specific policy overlays without forking Decapod.
Keep overrides minimal, explicit, and committed.
"#
    .to_string()
}

fn template_override() -> String {
    r#"# OVERRIDE.md - Project-Specific Decapod Overrides

> **IMPORTANT:** For detailed usage instructions and examples, see [README.md](README.md).

**Canonical:** OVERRIDE.md
**Authority:** override
**Layer:** Project
**Binding:** Yes (overrides embedded constitution)

<!-- ═══════════════════════════════════════════════════════════════════════ -->
<!-- ⚠️  CHANGES ARE NOT PERMITTED ABOVE THIS LINE                           -->
<!-- ═══════════════════════════════════════════════════════════════════════ -->

## Core Overrides (Routers and Indices)

### core/ENGINEERING_EXCELLENCE.md

### core/DECAPOD.md

### core/INTERFACES.md

### core/METHODOLOGY.md

### core/PLUGINS.md

### core/GAPS.md

### core/DEMANDS.md

### core/DEPRECATION.md

---

## Specs Overrides (System Contracts)

### specs/INTENT.md

### specs/SYSTEM.md

### specs/AMENDMENTS.md

### specs/SECURITY.md

### specs/GIT.md

---

## Interfaces Overrides (Binding Contracts)

### interfaces/CLAIMS.md

### interfaces/CONTROL_PLANE.md

### interfaces/DOC_RULES.md

### interfaces/GLOSSARY.md

### interfaces/STORE_MODEL.md

---

## Methodology Overrides (Practice Guides)

### methodology/ARCHITECTURE.md

### methodology/SOUL.md

### methodology/KNOWLEDGE.md

### methodology/MEMORY.md

---

## Architecture Overrides (Domain Patterns)

### architecture/DATA.md

### architecture/CACHING.md

### architecture/MEMORY.md

### architecture/WEB.md

### architecture/CLOUD.md

### architecture/FRONTEND.md

### architecture/ALGORITHMS.md

### architecture/SECURITY.md

### architecture/OBSERVABILITY.md

### architecture/CONCURRENCY.md

---

## Plugins Overrides (Operational Subsystems)

### plugins/TODO.md

### plugins/MANIFEST.md

### plugins/EMERGENCY_PROTOCOL.md

### plugins/DB_BROKER.md

### plugins/CRON.md

### plugins/REFLEX.md

### plugins/HEALTH.md

### plugins/POLICY.md

### plugins/WATCHER.md

### plugins/KNOWLEDGE.md

### plugins/ARCHIVE.md

### plugins/FEDERATION.md

### plugins/FEEDBACK.md

### plugins/TRUST.md

### plugins/CONTEXT.md

### plugins/HEARTBEAT.md

### plugins/APTITUDE.md

### plugins/VERIFY.md

### plugins/DECIDE.md

### plugins/AUTOUPDATE.md
"#
    .to_string()
}

pub fn get_template(name: &str) -> Option<String> {
    match name {
        "AGENTS.md" => Some(template_agents()),
        "CLAUDE.md" => Some(template_named_agent("CLAUDE")),
        "GEMINI.md" => Some(template_named_agent("GEMINI")),
        "CODEX.md" => Some(template_named_agent("CODEX")),
        "README.md" => Some(template_readme()),
        "OVERRIDE.md" => Some(template_override()),
        _ => None,
    }
}
