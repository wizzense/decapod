# Nemotron Architectural Reconnaissance

## What Decapod Already Is

Decapod is a daemonless, local-first governance kernel for AI coding agents. It provides agents with a control plane that enriches context, turns natural-language intent into explicit specs, enforces workspace and policy boundaries, coordinates mutable state, and requires proof-backed completion. Key aspects include:

- **Agent-agent communication**: Agents call Decapod at governance boundaries (pre-inference and post-inference) to shape intent, bound context, and verify completion.
- **Repo-native state**: All state lives in the `.decapod/` directory, including todos, specs, context capsules, artifacts, and workspaces.
- **Constitution-driven**: An embedded constitution (via `assets/constitution.json`) provides declarative guidance on architecture, security, performance, and testing that agents can query via `decapod constitution get`.
- **Specs as living contracts**: Generated specs in `.decapod/generated/specs/` (INTENT.md, ARCHITECTURE.md, etc.) are dynamic documents that align with evolving intent and reality.
- **Todo-based coordination**: A robust todo system (`decapod todo`) enables task tracking, claiming, and isolation across concurrent agents.
- **Validation and proof**: The `decapod validate` command runs a comprehensive validation suite enforcing methodology gates, while `decapod proof` runs configurable proof checks with audit trails.
- **Workspace isolation**: Git worktrees and optional Docker containers provide isolated workspaces scoped to specific todos, preventing interference with the main repository checkout.

## Current Architecture Map

Decapod's architecture consists of several interconnected subsystems:

1. **CLI Layer** (`src/cli.rs`): Defines command-line interface and argument parsing for all Decapod commands (init, todo, validate, proof, workspace, etc.).
2. **Core Runtime** (`src/core/`):
   - **Todo System** (`todo.rs`): SQLite-backed task management with claims, ownership, dependencies, and event journaling.
   - **Validation** (`validate.rs`: Intent-driven methodology validation harness with numerous gates (namespace purge, embedded self-contained, repo map, etc.).
   - **Proof** (`proof.rs`): Configurable proof execution from `proofs.toml` with health claim synchronization.
   - **Workspace** (`workspace.rs`): Git worktree and Docker container management for isolated agent workspaces.
   - **Constitution Access** (`assets.rs`): Embedded constitution retrieval and merging with project overrides (OVERRIDE.md).
   - **Specs Generation** (`project_specs.rs`: Scaffolding and manifest management for `.decapod/generated/specs/`.
   - **Storage** (`store.rs`, `db.rs`): Store abstraction and SQLite database access.
   - **Broker** (`broker.rs`): Audit log mutation broker with replay verification.
   - **Workunit** (`workunit.rs`): Work unit manifests linking tasks to specs, state, and proof.
   - **Plan Governance** (`plan_governance.rs`: Governed PLAN artifacts with state transitions.
   - **Context Capsules** (`context_capsule.rs`: Deterministic context resolution from embedded constitution.
   - **Obligation Graph** (`obligation.rb`: Obligation tracking for governance.
   - **Flight Recorder** (`flight_recorder.rs`: Timeline rendering from event logs.
3. **Plugins** (`src/plugins/`): Extensible subsystems (aptitude, federation, health, knowledge, policy, verify, workflow, etc.).
4. **Constitution** (`src/constitution/`): Core constitution nodes (core, interfaces, methodology, specs) re-exported via `src/constitution/core.rs`.
5. **Agent Interface**: Agent-facing documentation (AGENTS.md, CLAUDE.md, etc.) and the Universal Agent Contract (AGENTS.md) that mandates Decapod usage patterns.

Data flows:
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

## Where Todos Currently Sit

Todos are currently a standalone subsystem within the core layer (`src/core/todo.rs`) with the following characteristics:

- **Storage**: SQLite database (`todo.db`) and event journal (`todo.events.jsonl`) in `.decapod/data/`.
- **CLI Interface**: Full-featured todo commands (add, list, get, claim, done, archive, comment, edit, handoff, etc.) via `todo::TodoCli`.
- **Core Integrations**:
  - **Workspace**: Todos are used to scope git worktrees (branch names include todo IDs/hashes) and enforce exclusive agent ownership via claiming.
  - **Validation**: The validation harness checks todo store integrity (blank-slate semantics for user store, dogfood backlog for repo store) and event log consistency.
  - **Proof**: Proof events can be associated with todos via the health subsystem (proof claims sync).
  - **Workunit**: Work unit manifests link to todos via task_id.
  - **Plugins**: The todo subsystem is referenced by plugins (aptitude, federation, knowledge, policy, verify) for various integrations.
- **Limitations**:
  - Todos are primarily task-tracking focused (title, description, status, priority, etc.) but lack broader workflow trajectory concepts.
  - The todo subsystem does not inherently model temporal sequences, state transitions, or causal relationships between tasks beyond simple dependencies (depends_on/blocks).
  - While todos support claiming and ownership, they don't encapsulate broader governance contexts like intent, specifications, or proof expectations in a unified trajectory object.
  - The todo system is optimized for discrete task management rather than representing continuous work streams or evolutionary trajectories.

## Why Trajectories May Need to Become First-Class

The current todo-centric model shows signs of strain when representing complex, evolving work:

1. **Atomic Task Bias**: Treating work as discrete tickets loses the narrative of how intent evolves over time through iterations, pivots, and discoveries.
2. **Context Loss**: Each todo is an isolated unit; there's no inherent mechanism to carry forward assumptions, context summaries, or learned lessons across a sequence of related work.
3. **Manual Coordination**: Agents must manually maintain epistemic custody (preserving intent-context-assumptions-action-proof chains) across multiple todos, which is error-prone.
4. **Limited Retrospection**: While the worker loop records lessons, there's no first-class construct for representing the evolutionary path of a feature or system.
5. **Overloading Todo Fields**: Current attempts to store trajectory-like data (e.g., in description, tags, or custom fields) fight against the todo subsystem's primary purpose as a task tracker.
6. **Missing Abstractions**: Concepts like "work streams," "feature branches," "evolutionary trajectories," or "custody chains" lack explicit representation, forcing agents to encode them implicitly in todo relationships or external documentation.

Trajectories as first-class entities would provide:
- Explicit modeling of work sequences with temporal ordering and state evolution.
- Built-in mechanisms for preserving uncertainty and recursive continuity of assumptions.
- Natural encapsulation of intent-context-specs-state-proof chains.
- Improved tooling for visualizing and navigating work histories.
- Better alignment with how humans and agents actually work in iterative, discovery-driven processes.

## Recommended Redesign Seam

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

This staged approach preserves all working behavior while introducing trajectories as an opt-in, higher-order construct that agents can adopt when needed for complex work.

## Risks and Traps to Avoid

1. **Over-Specification**: Avoid defining too rigid a trajectory model upfront. Start with minimal fields (id, intent, current_state, spec_refs, workunit_chain) and allow evolution.
2. **Duplicate Storage**: Don't create parallel storage mechanisms that duplicate todo or workunit data. Instead, link via references.
3. **Complexity Creep**: Resist adding unnecessary trajectory states or transitions. Begin with simple linear progression (e.g., drafted → in_progress → validated → archived).
4. **Agent Confusion**: Ensure clear documentation distinguishes between todos (discrete tasks) and trajectories (evolutionary work). Provide migration paths for existing todo-heavy workflows.
5. **Performance Impact**: Ensure trajectory operations don't add significant overhead to common todo or validation operations. Use lazy loading where appropriate.
6. **Override Conflicts**: Plan for how trajectory directives in OVERRIDE.md would interact with existing constitution overrides.
7. **Tooling Gaps**: Don't forget to update related tooling (docs, validation gates, plugin integrations) to recognize trajectories.
8. **Backwards Compatibility**: Guarantee that existing workflows using only todos continue to function unchanged without requiring trajectory adoption.

## Suggested First GitHub Issue

**Title**: Explore: Design minimal trajectory model extending workunit for evolutionary work tracking

**Description**: 
Investigate and prototype a minimal first-class trajectory construct that builds upon the existing workunit subsystem to represent sequences of related work. The goal is to enable agents to explicitly model and track the evolution of intent, specifications, state, and proof over time, without disrupting current todo-based task tracking.

**Key Tasks**:
- [ ] Review workunit.rs and identify extension points for trajectory grouping (e.g., optional trajectory_id).
- [ ] Draft a minimal trajectory model (struct Trajectory) with fields: id, intent, current_state, spec_refs (Vec<String>), workunit_chain (Vec<String>), created_at, updated_at.
- [ ] Propose storage strategy: Extend workunit manifest or create new trajectory journal (similar to todo.events.jsonl).
- [ ] Outline validation gates for trajectory integrity (e.g., state transition monotonicity, proof coverage).
- [ ] Sketch CLI command structure (`decapod trajectory init`, `list`, `show`, `transition`).
- [ ] Identify constitution nodes that might govern trajectory lifecycle (e.g., methodology/trajectory_lifecycle).
- [ ] Ensure backwards compatibility: existing workunits and todos must function without trajectory awareness.
- [ ] Deliver: A design document (in .decapod/generated/specs/) and/or proof-of-concept code in a branch.

## Suggested Second GitHub Issue

**Title**: Implement: Add trajectory-aware validation gate and basic CLI scaffolding

**Description**: 
Building on the exploration from the first issue, implement the minimal viable trajectory functionality: a validation gate to check trajectory integrity and basic CLI commands for trajectory creation and inspection.

**Key Tasks**:
- [ ] Add optional trajectory_id field to Workunit manifest (backwards compatible).
- [ ] Create trajectory.rs subsystem with core functions (create, get, transition).
- [ ] Implement a validation gate (e.g., validate_trajectory_integrity) in validate.rs that checks:
   - Trajectory-referenced workunits exist and are valid.
   - State transitions follow a defined lifecycle (draft → in_progress → validated → archived).
   - Each trajectory has an associated intent spec.
   - Proof expectations are linked to trajectory milestones.
- [ ] Implement CLI commands under `decapod trajectory`:
   - `init`: Create a new trajectory with intent.
   - `list`: Show all trajectories.
   - `show`: Display trajectory details.
   - `transition`: Advance trajectory to next state.
- [ ] Update project_specs.rs to optionally scaffold a TRAJECTORY.md spec (guided by update_guidance).
- [ ] Ensure all changes are backwards compatible: existing workflows must continue to work without modification.
- [ ] Deliver: Working implementation in a branch with unit tests and updated documentation.

## Files Worth Reading Next

1. **src/core/todo.rs** - The current task tracking subsystem (core focus area for potential trajectory integration).
2. **src/core/workunit.rs** - Work unit manifests linking tasks to specs, state, and proof (natural extension point for trajectories).
3. **src/core/project_specs.rs** - Specs generation and manifest system (where trajectory specs would live).
4. **src/core/validate.rs** - Validation harness (where trajectory integrity gates would be added).
5. **src/core/assets.rs** - Constitution access system (for retrieving trajectory-related directives).
6. **src/core/workspace.rs** - Workspace management (for potential trajectory-scoped isolation).
7. **docs/agent/api-index.md** - Agent orientation on core Decapod concepts.
8. **docs/agent/command-contracts.md** - Detailed command specifications (for understanding current CLI structure).
9. **.decapod/generated/specs/** - Example of current living specs (INTENT.md, ARCHITECTURE.md, etc.).
10. **src/constitution/** - Embedded constitution structure (for where trajectory directives might reside).
