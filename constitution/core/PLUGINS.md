# PLUGINS.md - Subsystem Registry

**Authority:** interface (subsystem truth registry)
**Layer:** Interfaces
**Binding:** Yes
**Scope:** canonical list of subsystem surfaces, status, truth labels, and deprecation routing
**Non-goals:** tutorial workflows and architecture doctrine

This is the single source of truth for Decapod subsystem status. Every agent, human or artificial, must consult this registry to understand what capabilities exist and their current implementation state.

---

## Table of Contents

1. [Truth Labels](#1-truth-labels)
2. [Subsystem Registry](#2-subsystem-registry)
3. [Deprecation Routing](#3-deprecation-routing)
4. [Registry Discipline](#4-registry-discipline)
5. [Subsystem Detailed Reference](#5-subsystem-detailed-reference)
6. [Plugin-Grade Requirements](#6-plugin-grade-requirements)
7. [Truth Label Transition Paths](#7-truth-label-transition-paths)
8. [Anti-Patterns](#8-anti-patterns)

---

## 1. Truth Labels

Truth labels communicate the maturity and reliability of a subsystem. Using the correct label is not optional — it is the primary mechanism by which agents assess risk and make promises about system behavior.

| Label | Meaning | Promise to Users |
|-------|---------|------------------|
| `REAL` | Implemented and supported | The surface works as documented and has a proof surface |
| `STUB` | Interface exists, behavior incomplete | The surface exists but doesn't fully deliver the documented behavior |
| `SPEC` | Designed contract, not implemented | The surface is designed but not yet built |
| `IDEA` | Exploratory only | The surface is a concept, not a commitment |
| `DEPRECATED` | Superseded; do not target | The surface is replaced; new work must not use it |

**Critical constraint**: `REAL` entries MUST name an executable proof surface. If no proof surface exists, the entry MUST be labeled `STUB` or `SPEC`, not `REAL`.

**What breaks when you misuse labels:**
- `REAL` without proof surface → agents make promises the system can't keep → trust erosion
- `STUB` marked as `REAL` → agents try to use unimplemented behavior → failed workflows
- `DEPRECATED` still in use → new work builds on removed foundations → refactoring debt

---

## 2. Subsystem Registry

The table below is the authoritative source of truth for Decapod subsystem status. Tools, scripts, and documentation that reference subsystems MUST check this registry.

| Name | CLI Surface | Status | Truth | Owner Doc | Proof Surface | Deprecation Replacement |
|------|-------------|--------|-------|-----------|---------------|--------------------------|
| todo | `decapod todo` | implemented | REAL | `plugins/TODO.md` | `decapod data schema --subsystem todo` | — |
| docs | `decapod docs` | implemented | REAL | `core/DECAPOD.md` | `decapod docs list` | — |
| validate | `decapod validate` | implemented | REAL | `plugins/VERIFY.md` | `decapod validate` | — |
| health | `decapod govern health` | implemented | REAL | `plugins/HEALTH.md` | `decapod govern health get` | — |
| policy | `decapod govern policy` | implemented | REAL | `plugins/POLICY.md` | `decapod govern policy riskmap verify` | — |
| watcher | `decapod govern watcher` | implemented | REAL | `plugins/WATCHER.md` | `decapod govern watcher run` | — |
| feedback | `decapod govern feedback` | implemented | REAL | `plugins/FEEDBACK.md` | `decapod govern feedback propose` | — |
| knowledge | `decapod data knowledge` | implemented | REAL | `plugins/KNOWLEDGE.md` | `decapod data knowledge search` | — |
| aptitude | `decapod data aptitude` (aliases: `memory`, `skills`) | implemented | REAL | `plugins/APTITUDE.md` | `decapod data aptitude schema` | — |
| context | `decapod data context` | implemented | REAL | `plugins/CONTEXT.md` | `decapod data context audit` | — |
| archive | `decapod data archive` | implemented | REAL | `plugins/ARCHIVE.md` | `decapod data archive verify` | — |
| cron | `decapod auto cron` | implemented | REAL | `plugins/CRON.md` | `decapod data schema --subsystem cron` | — |
| reflex | `decapod auto reflex` | implemented | REAL | `plugins/REFLEX.md` | `decapod data schema --subsystem reflex` | — |
| workflow | `decapod auto workflow` | implemented | REAL | `plugins/REFLEX.md` | `decapod data schema --subsystem workflow` | — |
| container | `decapod auto container` | implemented | REAL | `plugins/CONTAINER.md` | `decapod data schema --subsystem container` | — |
| federation | `decapod data federation` | implemented | REAL | `plugins/FEDERATION.md` | `decapod data schema --subsystem federation` | — |
| primitives | `decapod data primitives` | implemented | REAL | `plugins/TODO.md` | `decapod data primitives validate` | — |
| decide | `decapod decide` | implemented | REAL | `plugins/DECIDE.md` | `decapod data schema --subsystem decide` | — |
| internalize | `decapod internalize` | implemented | REAL | `interfaces/INTERNALIZATION_SCHEMA.md` | `decapod internalize inspect --id <id>` | — |
| session | `decapod session` | implemented | REAL | `specs/SECURITY.md` | `decapod session acquire` + validation | — |
| lcm | `decapod lcm` | implemented | REAL | `interfaces/LCM.md` | `decapod lcm rebuild --validate` | — |
| map | `decapod map` | implemented | REAL | `interfaces/LCM.md` | `decapod map agentic --retain` | — |
| workunit | `decapod workunit` | implemented | REAL | `interfaces/PLAN_GOVERNED_EXECUTION.md` | `decapod workunit publish` gate | — |
| eval | `decapod eval` | implemented | REAL | `specs/evaluations/*.md` | `decapod eval gate` + variance checks | — |
| capsule | `decapod govern capsule` | implemented | REAL | `interfaces/AGENT_CONTEXT_PACK.md` | `decapod govern capsule query` policy checks | — |
| skill | `decapod data aptitude skill` | implemented | REAL | `specs/skills/SKILL_GOVERNANCE.md` | `decapod data aptitude skill import --write-card` | — |
| db_broker | `decapod data broker` | planned | SPEC | `plugins/DB_BROKER.md` | not yet enforced | — |
| heartbeat | `decapod heartbeat` | removed | DEPRECATED | `plugins/HEARTBEAT.md` | replacement: `decapod govern health summary` | `govern health summary` |
| trust | `decapod trust` | removed | DEPRECATED | `plugins/TRUST.md` | replacement: `decapod govern health autonomy` | `govern health autonomy` |

---

## 3. Deprecation Routing

When a subsystem is deprecated, this registry provides the canonical replacement path. Agents encountering deprecated surfaces MUST route users to the replacement.

### 3.1 Current Deprecations

**`heartbeat` → `govern health summary`**
- **Deprecated surface**: `decapod heartbeat`
- **Replacement surface**: `decapod govern health summary`
- **Migration steps**:
  1. Replace `decapod heartbeat` calls with `decapod govern health summary`
  2. The replacement provides the same liveness signal plus additional subsystem health detail
  3. Scripts calling `heartbeat` should be updated before the next deployment cycle
- **Why deprecated**: The health subsystem provides richer health signals beyond simple liveness, including per-subsystem status and autonomy metrics

**`trust` → `govern health autonomy`**
- **Deprecated surface**: `decapod trust`
- **Replacement surface**: `decapod govern health autonomy`
- **Migration steps**:
  1. Replace `decapod trust` calls with `decapod govern health autonomy`
  2. The replacement provides the same trust/autonomy signals with better policy integration
- **Why deprecated**: Trust semantics were subsumed into a broader health/autonomy model

### 3.2 Deprecation Policy

1. Deprecated surfaces remain functional for a minimum of 90 days after deprecation notice
2. Documentation MUST point to replacement surfaces, not deprecated command groups
3. Deprecation notice must be visible in CLI help output (`--help`)
4. Deprecated surfaces must be marked `DEPRECATED` in this registry
5. After sunset period, deprecated surfaces may return "command not found" or "deprecated" errors

---

## 4. Registry Discipline

### 4.1 Single Source of Truth

1. **If a subsystem is not listed here, it is not canonical.** No agent or doc may claim a subsystem exists if it's not in this registry.
2. **Other docs may reference subsystems but MUST NOT define competing lists.** All subsystem references must route to this registry.
3. **Status changes MUST update this registry and corresponding owner docs together.** A change to subsystem status without updating both locations creates drift.
4. **Proof surfaces listed here must be runnable.** If a proof surface cannot be executed, the subsystem truth label should be downgraded.

### 4.2 Registry Update Process

When adding or changing a subsystem:

1. **Identify the truth label**: Is it implemented? Partially implemented? Designed but not built? Exploratory?
2. **Find or create the owner doc**: Each subsystem needs a canonical owner document
3. **Define the proof surface**: What executable check verifies the subsystem works?
4. **Add to this registry**: Include all columns, especially truth label and proof surface
5. **Update the owner doc**: Reference this registry and the proof surface
6. **Run validation**: `decapod validate` must pass after the change

### 4.3 Truth Label Decisions

Use this decision tree to determine the correct truth label:

```
Is the subsystem implemented and fully functional?
├── YES → Is there a named proof surface?
│         ├── YES → REAL
│         └── NO → STUB (add proof surface or it's not really REAL)
└── NO → Is there a complete design document?
          ├── YES → SPEC
          └── NO → Is this an exploratory concept?
                    ├── YES → IDEA
                    └── NO → You probably need to write the design first
```

---

## 5. Subsystem Detailed Reference

### 5.1 Core Operational Subsystems

**`todo` — Work Tracking**
- CLI: `decapod todo`
- Purpose: Track work items, ownership, and resolution
- Key commands: `add`, `claim`, `done`, `list`, `prioritize`
- Store: Operates on both user and repo stores
- Proof: `decapod data schema --subsystem todo`
- **Critical invariant**: Claim-before-work (claim: `claim.todo.claim_before_work`)

**`docs` — Documentation Navigation**
- CLI: `decapod docs`
- Purpose: List, show, search, and navigate canonical documentation
- Key commands: `list`, `show`, `search`, `ingest`
- Proof: `decapod docs list`
- **Critical invariant**: Doc graph reachability verified by validate

**`validate` — Proof and Invariant Verification**
- CLI: `decapod validate`
- Purpose: Run all proof surfaces and check documented invariants
- Key commands: (no subcommands; runs full suite by default)
- Proof: `decapod validate` itself
- **Critical invariants**:
  - Bounded termination (claim: `claim.validate.bounded_termination`)
  - No cross-turn lock residency (claim: `claim.validate.no_cross_turn_lock_residency`)

**`session` — Session Management**
- CLI: `decapod session`
- Purpose: Acquire and manage authenticated sessions
- Key commands: `acquire`, `ensure`, `revoke`
- Proof: `decapod session acquire` + password check
- **Critical invariant**: Agent identity + ephemeral password required (claim: `claim.session.agent_password_required`)

### 5.2 Governance Subsystems

**`health` — System Health Monitoring**
- CLI: `decapod govern health`
- Purpose: Monitor and report subsystem health status
- Key commands: `get`, `summary`, `autonomy`
- Proof: `decapod govern health get`

**`policy` — Policy Management**
- CLI: `decapod govern policy`
- Purpose: Define, verify, and enforce operational policies
- Key commands: `riskmap verify`, `policy check`
- Proof: `decapod govern policy riskmap verify`

**`watcher` — Change Watching**
- CLI: `decapod govern watcher`
- Purpose: Monitor for external changes and trigger responses
- Key commands: `run`, `status`
- Proof: `decapod govern watcher run`

**`feedback` — Feedback Collection**
- CLI: `decapod govern feedback`
- Purpose: Collect and process feedback on system operation
- Key commands: `propose`, `list`
- Proof: `decapod govern feedback propose`

**`capsule` — Context Capsule Management**
- CLI: `decapod govern capsule`
- Purpose: Issue and manage deterministic context capsules
- Key commands: `query`, `issue`
- Proof: `decapod govern capsule query` policy checks
- **Critical invariant**: Policy-bound issuance (claim: `claim.context.capsule.policy_enforced`)

### 5.3 Data Subsystems

**`knowledge` — Knowledge Base**
- CLI: `decapod data knowledge`
- Purpose: Store and retrieve curated knowledge entries
- Key commands: `add`, `search`, `promote`
- Proof: `decapod data knowledge search`
- **Critical invariants**:
  - Provenance required (claim: `claim.knowledge.provenance_required`)
  - Directional flow enforced (claim: `claim.knowledge.directional_flow`)

**`federation` — Federated Data**
- CLI: `decapod data federation`
- Purpose: Manage federated data with provenance and lifecycle tracking
- Key commands: `query`, `ingest`
- Proof: `decapod data schema --subsystem federation`
- **Critical invariants**:
  - Store-scoped (claim: `claim.federation.store_scoped`)
  - Provenance required for critical (claim: `claim.federation.provenance_required_for_critical`)
  - Append-only for critical (claim: `claim.federation.append_only_critical`)
  - No lifecycle DAG cycles (claim: `claim.federation.lifecycle_dag_no_cycles`)

**`context` — Context Management**
- CLI: `decapod data context`
- Purpose: Manage agent context and working memory
- Key commands: `audit`, `compact`
- Proof: `decapod data context audit`

**`archive` — Long-Term Storage**
- CLI: `decapod data archive`
- Purpose: Archive and retrieve historical data
- Key commands: `store`, `retrieve`, `verify`
- Proof: `decapod data archive verify`

### 5.4 Automation Subsystems

**`cron` — Scheduled Jobs**
- CLI: `decapod auto cron`
- Purpose: Define and execute scheduled tasks
- Key commands: `schedule`, `list`, `cancel`
- Proof: `decapod data schema --subsystem cron`

**`reflex` — Event-Driven Responses**
- CLI: `decapod auto reflex`
- Purpose: Define and execute event-driven reactions
- Key commands: `define`, `trigger`, `list`
- Proof: `decapod data schema --subsystem reflex`

**`workflow` — Workflow Orchestration**
- CLI: `decapod auto workflow`
- Purpose: Define and execute multi-step workflows
- Key commands: `define`, `run`, `status`
- Proof: `decapod data schema --subsystem workflow`

**`container` — Ephemeral Execution**
- CLI: `decapod auto container`
- Purpose: Run isolated operations in ephemeral containers
- Key commands: `run`, `status`
- Proof: `decapod data schema --subsystem container`
- **Critical invariant**: Git workspace isolation (claim: `claim.git.container_workspace_required`)

### 5.5 Skill and Aptitude Subsystems

**`aptitude` — Skill Management**
- CLI: `decapod data aptitude`
- Aliases: `memory`, `skills`
- Purpose: Import, resolve, and manage agent skills
- Key commands: `skill import`, `skill resolve`, `schema`
- Proof: `decapod data aptitude schema`
- **Critical invariants**:
  - Deterministic skill cards (claim: `claim.skill.card.deterministic`)
  - Deterministic resolution (claim: `claim.skill.resolve.deterministic`)
  - No unverified authority (claim: `claim.skill.no_unverified_authority`)

**`decide` — Decision Support**
- CLI: `decapod decide`
- Purpose: Structured decision support and architecture reasoning
- Key commands: `analyze`, `recommend`
- Proof: `decapod data schema --subsystem decide`

### 5.6 SPEC-Status Subsystems

**`db_broker` — Database Broker**
- CLI: `decapod data broker`
- Status: Planned, not implemented
- Truth: SPEC
- Owner: `plugins/DB_BROKER.md`
- Purpose: Serialized writes and audit trail for database operations
- Proof: Not yet enforced
- **Note**: Will graduate to REAL in Epoch 4 per project roadmap

---

## 6. Plugin-Grade Requirements

For a subsystem to be considered "plugin-grade" and included in this registry, it MUST meet the following requirements:

### 6.1 Command Surface Requirements

1. **Stable command group**: Commands must be grouped under `decapod <subsystem>` with consistent subcommand structure
2. **Stable JSON envelope**: All commands must support `--format json` with consistent response envelope
3. **Store-aware behavior**: Commands must respect `--store user|repo` and `--root <path>` parameters
4. **Schema/discovery surface**: Must expose `decapod <subsystem> schema` or equivalent for capability discovery

### 6.2 Integration Requirements

1. **Validate integration**: Must be verifiable by `decapod validate` (proof surface required for REAL)
2. **Help surface**: `--help` must return meaningful documentation
3. **Error handling**: Must return typed errors, not panics
4. **Store isolation**: Must not leak state between stores

### 6.3 Documentation Requirements

1. **Owner document**: Must have a canonical doc describing the subsystem
2. **Registry entry**: Must be listed in this registry with accurate truth label
3. **Proof surface**: Must have a runnable proof surface for REAL status

---

## 7. Truth Label Transition Paths

Subsystems progress through truth labels over time. The following paths are canonical:

### 7.1 Happy Path: IDEA → SPEC → STUB → REAL

```
IDEA (exploratory concept)
    │
    │ Decision: Design is sound, implementation begins
    ▼
SPEC (designed contract)
    │
    │ Decision: Implementation complete, proof surface exists
    ▼
STUB (interface exists, behavior incomplete — still needs work)
    │
    │ Decision: Behavior is complete and verified
    ▼
REAL (implemented and supported)
```

### 7.2 Deprecation Path: REAL → DEPRECATED → (removed)

```
REAL (implemented and working)
    │
    │ Decision: Superseded by better approach
    ▼
DEPRECATED (do not use for new work)
    │
    │ 90+ days pass, migration complete
    ▼
Removed (command returns error or redirect)
```

### 7.3 Downgrade Path: REAL → STUB

```
REAL (implemented and working)
    │
    │ Regression discovered, proof surface fails
    ▼
STUB (behavior incomplete or broken)
    │
    │ Fix implemented, proof surface passes
    ▼
REAL (restored)
```

### 7.4 Reclassification Path: SPEC → IDEA

```
SPEC (designed but not implemented)
    │
    │ Decision: Design no longer viable, demote to exploration
    ▼
IDEA (exploratory — may be revived with new design)
```

---

## 8. Anti-Patterns

### 8.1 Registry Anti-Patterns

**Phantom REAL**
- Listing a subsystem as REAL without a working proof surface
- **What breaks**: Agents trust the surface, work fails, trust erodes
- **How to detect**: Run the proof surface; if it fails or doesn't exist, it's not REAL

**Stale STUB**
- STUB entries that have been STUB for months without a graduation path
- **What breaks**: Teams work around missing functionality instead of resolving it
- **How to detect**: Check STUB entries for old timestamps or missing TODO items

**Orphan SPEC**
- SPEC entries without an implementation plan or timeline
- **What breaks**: Design rots; eventually implementation attempts fail because context is lost
- **How to detect**: SPEC entries older than 6 months without implementation tracking

**Duplicate Subsystem**
- Two subsystems that do the same thing
- **What breaks**: Agents confused about which to use; maintenance burden doubled
- **How to detect**: Similar CLI surfaces or overlapping functionality

### 8.2 Truth Label Misuse

**Marketing REAL**
- Calling something REAL because it's "good enough" without proof surface
- **What breaks**: Promise to users that can't be kept; agents make incorrect assumptions
- **Fix**: If no proof surface, it's STUB or SPEC

**Stub as REAL**
- Marking incomplete behavior as REAL because "it mostly works"
- **What breaks**: Agents try to use unimplemented behavior; workflows fail unexpectedly
- **Fix**: Mark as STUB; complete the implementation before promoting to REAL

**IDEA as SPEC**
- Calling exploratory work "designed" when it's just a concept
- **What breaks**: Implementation attempts founder on undefined requirements
- **Fix**: Keep at IDEA until there's a real design document

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - **Oracle for Engineering Standards (CTO->Principal)**
- `core/GAPS.md` - Gap analysis methodology

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security contract
- `specs/GIT.md` - Git etiquette contract
- `specs/AMENDMENTS.md` - Change control

### Registry (Core Indices)
- `core/INTERFACES.md` - Interface contracts index
- `core/METHODOLOGY.md` - Methodology guides index
- `core/DEPRECATION.md` - Deprecation contract
- `core/DEMANDS.md` - User demand patterns

### Contracts (Interfaces Layer)
- `interfaces/CONTROL_PLANE.md` - Sequencing patterns
- `interfaces/DOC_RULES.md` - Doc compilation rules
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/GLOSSARY.md` - Term definitions
- `interfaces/STORE_MODEL.md` - Store semantics
- `interfaces/TESTING.md` - Testing contract

### Operations (Plugins - This Registry)
- `plugins/TODO.md` - **Work tracking (PRIMARY)**
- `plugins/VERIFY.md` - Validation subsystem
- `plugins/MANIFEST.md` - Canonical vs derived vs state
- `plugins/EMERGENCY_PROTOCOL.md` - Emergency protocols
- `plugins/FEDERATION.md` - Federation (governed agent memory)
- `plugins/DECIDE.md` - Architecture decision prompting
- `plugins/CONTAINER.md` - Ephemeral isolated container execution
- `plugins/DB_BROKER.md` - Database broker (SPEC)
- `plugins/HEALTH.md` - Health monitoring
- `plugins/POLICY.md` - Policy management
- `plugins/WATCHER.md` - Change watching
- `plugins/FEEDBACK.md` - Feedback collection
- `plugins/APTITUDE.md` - Skill management
- `plugins/CONTEXT.md` - Context management
- `plugins/ARCHIVE.md` - Archive storage
- `plugins/CRON.md` - Scheduled jobs
- `plugins/REFLEX.md` - Event-driven responses
- `plugins/INTERNALIZATION_SCHEMA.md` - Internalization schema
- `plugins/HEARTBEAT.md` - Deprecated: use `govern health summary`
- `plugins/TRUST.md` - Deprecated: use `govern health autonomy`