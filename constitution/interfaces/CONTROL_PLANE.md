# CONTROL_PLANE.md - Agent<->Decapod Control Plane Patterns

**Authority:** patterns (interoperability and sequencing; not a project contract)
**Layer:** Interfaces
**Binding:** Yes
**Scope:** sequencing and interoperability patterns between agents and the Decapod CLI
**Non-goals:** subsystem inventories (see PLUGINS registry) or authority definitions (see SYSTEM)

---

## Table of Contents

1. [The Contract: Agents Talk to Decapod, Not the Internals](#1-the-contract-agents-talk-to-decapod-not-the-internals)
2. [The Standard Sequence (Every Meaningful Change)](#2-the-standard-sequence-every-meaningful-change)
3. [Interoperability: The Thin Waist](#3-interoperability-the-thin-waist)
4. [Invocation Heartbeat and Liveness](#4-invocation-heartbeat-and-liveness)
5. [Subsystem Truth (No Phantom Features)](#5-subsystem-truth-no-phantom-features)
6. [Stores: How Multi-Agent Work Stays Sane](#6-stores-how-multi-agent-work-stays-sane)
7. [Concurrency Pattern: Request, Don't Poke](#7-concurrency-pattern-request-dont-poke)
8. [Ambiguity and Capability Boundaries](#8-ambiguity-and-capability-boundaries)
9. [Validate Doctrine (Proof Currency)](#9-validate-doctrine-proof-currency)
10. [Locking and Liveness Contract](#10-locking-and-liveness-contract)

---

This document is about *how* agents should use Decapod as a local control plane: sequencing, patterns, and interoperability rules.

It is intentionally higher-level than subsystem docs. It exists to prevent "agents poking files and DBs" from becoming the de facto interface.

General methodology lives in `specs/INTENT.md` and `methodology/ARCHITECTURE.md`.

---

## 1. The Contract: Agents Talk to Decapod, Not the Internals

The control plane exists to make multi-agent behavior converge.

**Golden rules:**

1. **Agents must not directly manipulate shared state** (databases, state files) if a Decapod command exists for it.
2. **Agents must not read or write `<repo>/.decapod/*` files directly**; access is only through `decapod` CLI surfaces. (claim: `claim.store.decapod_cli_only`)
3. **Agents must not invent parallel CLIs or parallel state roots.**
4. **Agents must claim a TODO** (`decapod todo claim --id <task-id>`) before substantive implementation work on that task. (claim: `claim.todo.claim_before_work`)
5. **If the command surface is missing, the work is to add the surface, not to bypass it.**
6. **Preserve control-plane opacity** at the operator interface: communicate intent/actions/outcomes, not command-surface mechanics, unless diagnostics are explicitly requested.
7. **Liveness must be maintained** through invocation heartbeat: each Decapod command invocation should refresh agent presence.
8. **Session access must be bound to agent identity plus ephemeral password** (`DECAPOD_AGENT_ID` + `DECAPOD_SESSION_PASSWORD`) for command authorization. (claim: `claim.session.agent_password_required`)
9. **Control-plane operations MUST remain daemonless and local-first**; no required always-on coordinator may become a hidden dependency.
10. **No single session may hold datastore locks** across user turns; lock scope must stay within a bounded command invocation.

This is how you get determinism, auditability, and eventually policy.

---

## 2. The Standard Sequence (Every Meaningful Change)

This is the default sequence when operating in a Decapod-managed repo:

### 2.1 The Ten-Step Sequence

```
1. Read the contract
   └─ constitution specs: INTENT.md, ARCHITECTURE.md, SYSTEM.md
   └─ local project specs: .decapod/generated/specs/*.md

2. Discover proof
   └─ identify smallest proof surface that can falsify success
   └─ e.g., decapod validate, tests, schema checks

3. Use Decapod as the interface
   └─ read/write shared state through `decapod ...` commands
   └─ never directly manipulate `<repo>/.decapod/*` files

4. Add a repo TODO for multi-step work (dogfood mode)
   └─ decapod todo add "Expand METHODOLOGY.md" --priority high

5. Claim the task before implementation
   └─ decapod todo claim --id <task-id>

6. Implement the change
   └─ make changes, following methodology guides
   └─ keep changes focused (smallest change)

7. Run proof and report results
   └─ decapod validate
   └─ cargo test (if applicable)
   └─ report: what passed, what failed

8. Update documentation
   └─ update relevant docs
   └─ add ## Links sections

9. Close the TODO
   └─ decapod todo done --id <task-id>
   └─ record the event

10. Report completion
    └─ what was verified
    └─ what was not verified
    └─ any remaining gaps
```

### 2.2 Invocation Checkpoints (Required)

For every meaningful task, agents MUST call Decapod at three checkpoints:

| Checkpoint | Decapod Command | Purpose |
|------------|-----------------|---------|
| **Before plan commitment** | `decapod rpc --op agent.init`<br>`decapod rpc --op context.resolve` | Initialize/resolve context |
| **Before mutation** | `decapod todo claim`<br>`decapod workspace ensure` | Claim work and ensure canonical workspace |
| **After mutation** | `decapod validate`<br>`cargo test` | Run proof surfaces before completion claims |

**Skipping a checkpoint invalidates completion claims.**

### 2.3 Proof Before Claims

If you cannot name the proof surface, you're not ready to claim correctness.

---

## 3. Interoperability: The Thin Waist

Decapod is a thin waist only if subsystems share the same interface qualities.

### 3.1 Subsystem Requirements (Agent-Visible)

| Requirement | Description |
|-------------|-------------|
| **Stable command group** | `decapod <subsystem> ...` |
| **Stable JSON envelope** | `--format json` or equivalent |
| **Store-aware behavior** | `--store user\|repo` plus `--root <path>` escape hatch |
| **Schema/discovery surface** | `decapod <subsystem> schema` |

### 3.2 Cross-Cutting Requirements

| Requirement | Description |
|-------------|-------------|
| **One place to validate repo invariants** | `decapod validate` |
| **One place to discover what exists** | schema/discovery, doc map |
| **One place to manage entrypoints to agents** | link subsystem (planned) |

If a subsystem cannot meet these, it is not a control-plane subsystem yet. Treat it as planned.

### 3.3 Thin Waist Diagram

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Agent A   │    │   Agent B   │    │   Agent C   │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌──────▼──────┐
                    │   Decapod   │  ← thin waist
                    │  (CLI only)  │
                    └──────┬──────┘
                           │
       ┌───────────────────┼───────────────────┐
       │                   │                   │
┌──────▼──────┐    ┌───────▼──────┐    ┌──────▼──────┐
│  Subsystem  │    │  Subsystem   │    │  Subsystem  │
│    todo     │    │    docs      │    │  knowledge  │
└─────────────┘    └──────────────┘    └─────────────┘
```

---

## 4. Invocation Heartbeat and Liveness

### 4.1 Heartbeat Mechanism

Decapod uses invocation heartbeat for agent presence:

- **Decapod auto-clocks liveness** on normal command invocation
- **Explicit `decapod todo heartbeat`** remains available for forced/manual heartbeat and optional autoclaim
- **Control-plane checks** must detect regressions where heartbeat decoration is removed

### 4.2 Heartbeat Rules

1. Each Decapod command invocation refreshes agent presence
2. If no command is run for a configured interval, agent may be considered stale
3. Explicit heartbeat can be used to maintain presence without other commands
4. Heartbeat is not a substitute for progress; it's a liveness signal

### 4.3 Liveness vs. Progress

| Concept | Description |
|---------|-------------|
| **Liveness** | Agent is present and responsive |
| **Progress** | Agent is doing useful work |

An agent can be live but not making progress (stuck, waiting). This is acceptable. An agent that is not live (no heartbeat) should be investigated.

---

## 5. Subsystem Truth (No Phantom Features)

### 5.1 Single Source of Truth

Subsystem status is defined only in the subsystem registry:
- `core/PLUGINS.md` §2 (Subsystem Registry)

Other docs must not restate subsystem lists. They must route to the registry.

### 5.2 Phantom Feature Prevention

| Anti-Pattern | Prevention |
|--------------|------------|
| Claiming subsystem exists that isn't in registry | Check PLUGINS.md before claiming |
| Claiming feature is REAL when it's STUB | Check truth labels |
| Building on DEPRECATED surfaces | Route to replacement |

---

## 6. Stores: How Multi-Agent Work Stays Sane

### 6.1 Store Model

Decapod supports multiple stores. The store is part of the request context.

| Store | Path | Purpose | Default |
|-------|------|---------|---------|
| **User store** | `~/.decapod` | User's personal state | Yes (default) |
| **Repo store** | `<repo>/.decapod/project` (store directory) | Project-specific state | No |

### 6.2 Store Rules

1. **Default store is the user store**
2. **Repo dogfooding must be explicit**: Use `--store repo`, or narrowly auto-detected via sentinel
3. **Store boundary is a hard boundary**: No auto-seeding from repo to user (claim: `claim.store.no_auto_seeding`)

### 6.3 Store Selection in Commands

```bash
# Default: user store
decapod todo list

# Explicit: repo store
decapod todo list --store repo

# Escape hatch: custom root (dangerous)
decapod todo list --root /path/to/store
```

### 6.4 When to Use Which Store

| Task | Store |
|------|-------|
| Personal work tracking | user |
| Constitution dogfooding | repo |
| Project-specific TODOs | repo |
|跨-agent shared state | repo |
| Experimenting | user |

---

## 7. Concurrency Pattern: Request, Don't Poke

### 7.1 The Pattern

SQLite is fast and simple until there are multiple writers and long-lived reads across multiple agents.

The desired pattern is:

```
Agents → Decapod request surface → serialized mutations + coalesced reads → shared state
```

### 7.2 Scope Discipline

| Stage | Approach |
|-------|----------|
| **Start** | local-first and boring (in-process broker) |
| **Grow** | prove value by solving two concrete problems first: serialized writes, in-flight read de-duplication |
| **Scale** | Only then consider distributed approaches |

### 7.3 The Win

The win is the protocol: once all access goes through one request layer, you can add:
- Tracing
- Priorities
- Idempotency keys
- Audit trails

...without rewriting the world.

---

## 8. Ambiguity and Capability Boundaries

### 8.1 When Intent Is Ambiguous

1. **If intent is ambiguous or policy boundaries conflict, agents MUST stop and ask for clarification** before irreversible implementation.
2. **Agents MUST NOT claim capabilities absent from the command surface**; missing capability is a gap to report, not permission to improvise hidden behavior.
3. **Lock/contention failures** (`VALIDATE_TIMEOUT_OR_LOCK` and related typed failures) are blocking failures until explicitly resolved or retried successfully.

### 8.2 Capability Boundary Rule

```
CLI surface says: decapod docs search --query X
CLI surface does NOT say: decapod docs index --rebuild

Therefore:
- search IS a capability
- index rebuild is NOT a capability
- If you need index rebuild, add the surface, don't manually poke
```

### 8.3 Missing Capability Protocol

When you need a capability that doesn't exist:

1. **Do not work around it**: Don't manually edit files
2. **Report it as a gap**: Create TODO with tag `missing-surface`
3. **Proceed without it if possible**: Find an alternative approach that uses existing surfaces
4. **Escalate if blocked**: If the gap blocks critical work, escalate

---

## 9. Validate Doctrine (Proof Currency)

### 9.1 Proof as Currency

Agents should treat proof as the control plane's currency:

- If validation exists, run it
- If validation doesn't exist, add the smallest validation gate that prevents drift
- If something is claimed in docs, validation should be able to detect it

This is how the repo avoids "doc reality" diverging from "code reality."

### 9.2 Validate Taxonomy (Current)

| Category | What It Checks |
|----------|----------------|
| **structural** | Directory rules, template buckets, namespace purge |
| **store** | Blank-slate user store, repo dogfood invariants |
| **interfaces** | Schema presence, output envelopes |
| **provenance** | Audit trails (planned) |
| **docs** | Doc graph reachability, subsystem registry consistency |

### 9.3 Severity Levels

| Level | Behavior |
|-------|----------|
| **error** | Fails validation (blocks claims) |
| **warn** | Allowed but noisy |
| **info** | Telemetry |

### 9.4 Validate Coverage Matrix

| Claim | Check |
|-------|-------|
| docs are machine-traceable | Doc Graph Gate (reachability via `## Links`) |
| subsystems don't drift | Plugins<->CLI Gate (registry matches `decapod --help`) |
| user store is blank-slate | Store: user blank-slate gate |
| repo backlog is reproducible | repo todo rebuild fingerprint gate |

---

## 10. Locking and Liveness Contract

### 10.1 Locking Requirements

Validation and promotion-critical checks must preserve control-plane liveness:

1. **`decapod validate` MUST terminate boundedly** (success or typed failure).
2. **Lock/contention failures MUST return structured, machine-readable error markers** (`VALIDATE_TIMEOUT_OR_LOCK` family), never silent hangs.
3. **Transactions in validation paths MUST be short-lived and scoped to a single invocation.**
4. **Promotion-relevant commands MUST treat typed timeout/lock failures as blocking failures** by default.

### 10.2 Lock Contention Protocol

When `VALIDATE_TIMEOUT_OR_LOCK` occurs:

1. **Stop**: Do not proceed with operation
2. **Report**: State the failure explicitly
3. **Retry or escalate**: Depending on context
4. **Do not bypass**: Lock failures are blocking, not advisory

### 10.3 Command-Scoped Locking

```
Turn 1: Agent calls decapod validate
        └─ Lock acquired, validation runs, lock released
        └─ Result returned

Turn 2: Agent calls decapod validate again
        └─ New lock acquired (no residual from Turn 1)
        └─ Lock released on completion
```

**No single session may hold locks across turns.**

---

## Links

### Core Router
- [core/DECAPOD.md](core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards**
- [core/GAPS.md](core/GAPS.md) - Gap analysis methodology

### Authority (Constitution Layer)
- [specs/INTENT.md](specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](specs/SYSTEM.md) - System definition and authority doctrine
- [specs/SECURITY.md](specs/SECURITY.md) - Security contract
- [specs/GIT.md](specs/GIT.md) - Git workflow contract

### Registry (Core Indices)
- [core/PLUGINS.md](core/PLUGINS.md) - **Subsystem registry**
- [core/INTERFACES.md](core/INTERFACES.md) - Interface contracts index
- [core/METHODOLOGY.md](core/METHODOLOGY.md) - Methodology guides index
- [core/DEPRECATION.md](core/DEPRECATION.md) - Deprecation contract

### Contracts (Interfaces Layer - This Document)
- [interfaces/DOC_RULES.md](interfaces/DOC_RULES.md) - Doc compilation rules
- [interfaces/STORE_MODEL.md](interfaces/STORE_MODEL.md) - Store semantics
- [interfaces/CLAIMS.md](interfaces/CLAIMS.md) - **Promises ledger**
- [interfaces/GLOSSARY.md](interfaces/GLOSSARY.md) - Term definitions
- [interfaces/TESTING.md](interfaces/TESTING.md) - Testing contract
- [interfaces/AGENT_CONTEXT_PACK.md](interfaces/AGENT_CONTEXT_PACK.md) - Agent context-pack contract
- [interfaces/PLAN_GOVERNED_EXECUTION.md](interfaces/PLAN_GOVERNED_EXECUTION.md) - Plan-governed execution
- [interfaces/KNOWLEDGE_STORE.md](interfaces/KNOWLEDGE_STORE.md) - Knowledge store semantics

### Practice (Methodology Layer)
- [methodology/SOUL.md](methodology/SOUL.md) - Agent identity
- [methodology/ARCHITECTURE.md](methodology/ARCHITECTURE.md) - Architecture practice
- [methodology/TESTING.md](methodology/TESTING.md) - Testing practice
- [methodology/KNOWLEDGE.md](methodology/KNOWLEDGE.md) - Knowledge curation
- [methodology/MEMORY.md](methodology/MEMORY.md) - Memory and learning

### Operations (Plugins Layer)
- [plugins/TODO.md](plugins/TODO.md) - Work tracking
- [plugins/VERIFY.md](plugins/VERIFY.md) - **Validation subsystem (PROOF SURFACES)**
- [plugins/MANIFEST.md](plugins/MANIFEST.md) - Manifest patterns
- [plugins/EMERGENCY_PROTOCOL.md](plugins/EMERGENCY_PROTOCOL.md) - Emergency protocols