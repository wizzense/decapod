# Decapod Constitution (Core)

## What Decapod Is

Decapod is the daemonless, local-first, repo-native governance kernel behind AI coding agents. It helps agents:
1. Build what the human intends
2. Follow the rules the human intends  
3. Produce the quality the human intends

The human primarily interfaces with the agent as the UX. The agent acts; Decapod orients.

Decapod is called on demand inside agent loops to turn intent into context, then context into explicit specifications before inference. Each invocation rehydrates repo state, emits artifacts or proof when needed, and exits.

## What Decapod Is Not

- Not an agent framework.
- Not a prompt-pack.
- Not a user-facing workflow app.
- Not the executor; agents remain responsible for implementation.
- Not a daemonized control plane with hidden always-on state.

## Router Charter

`core/DECAPOD.md` is a router, not a competing instruction surface.

- Agent operating rules: use `AGENTS.md`.
- Current task state: use `decapod todo`, generated specs, and workspace/status surfaces.
- Generated specs: use `.decapod/generated/specs/*` through Decapod CLI surfaces.
- Proof and completion: use `decapod validate`, proof-plan/status surfaces, and TODO completion state.
- Provider-specific shims (`CLAUDE.md`, `GEMINI.md`, `CODEX.md`): point back to `AGENTS.md`.

Call Decapod at pressure points: intent, boundaries, context, coordination, proof, and completion. Do not turn this router into generic documentation noise or a wrapper around every file read, local edit, or mechanical command.

## Foundation Demands (Non-Negotiable)

1. **Intent MUST be explicit before mutation.** If a change alters "what must be true," update intent/spec first.
2. **Boundaries MUST be explicit.** Authority boundary (`specs/` and `interfaces/`), interface boundary (`decapod` CLI/RPC), and store boundary (repo vs user) are mandatory.
3. **Completion MUST be provable.** Promotion-relevant outcomes require executable proof surfaces (`decapod validate` + required tests/gates), not narrative claims.
4. **Decapod MUST remain daemonless and repo-native.** Promotion-relevant state must be auditable from repo artifacts and control-plane receipts.
5. **Validation liveness is mandatory.** Validation must terminate boundedly with typed failure under contention, never hang indefinitely.
6. **Operational agent guidance MUST live in entrypoint and constitution surfaces, not README.** README is human-facing product documentation.
7. **Recursive improvement MUST respect authority hierarchy.** Agents may suggest improvements, but must not silently rewrite repository constitution, project/spec intent, task boundaries, proof requirements, or generated artifacts.

## For Agents: Quick Start

**You MUST call `decapod rpc --op agent.init` before operating.**

This produces a session receipt and tells you what's allowed next.

## Core Posture

- **Local-first**: Everything is on disk, auditable, versioned
- **No workflow replacement**: Keep using your existing agent flow; Decapod is called inside it
- **Deterministic**: Same inputs produce same outputs
- **Agent-native**: Designed for programmatic access via `decapod rpc`
- **Daemonless**: No required long-lived control-plane process
- **Host-agnostic**: Works as a local utility under different agent hosts/providers
- **Workspace-enforced**: You cannot work on main/master - Decapod refuses
- **Liveness-aware**: Requires **invocation heartbeat** for continuous presence tracking

## Key Commands

```bash
# Agent initialization (required first step)
decapod rpc --op agent.init

# Workspace management
decapod workspace status
decapod workspace ensure
decapod workspace publish

# Interview for spec generation
decapod rpc --op scaffold.next_question
decapod rpc --op scaffold.generate_artifacts

# Validation (must pass before claiming done)
decapod validate

# Capabilities discovery
decapod capabilities --format json
```

## Workspace Rules (Non-Negotiable)

1. **Agents MUST NOT work on main/master** - Decapod validates and refuses
2. **Use `decapod workspace ensure`** to create an isolated worktree under `.decapod/workspaces/*`
3. **Use on-demand containers** for build/test execution (clean env)
4. **Validate before claiming done** - `decapod validate` is the gate
5. **Do not use non-canonical worktree roots**

## Worktree + On-Demand Sandbox

Decapod enforces a two-tier isolation model:

1.  **Git Worktree (Default):**
    - All file modifications happen here.
    - Provides concurrency (multiple agents on different branches).
    - Prevents pollution of the main checkout.

2.  **On-Demand Sandbox (Container):**
    - Call `decapod workspace ensure --container` to instantiate.
    - Maps the *current* worktree into a clean Docker/OCI env.
    - **REQUIRED** for: `cargo build`, `npm install`, `pytest`, etc.
    - Ensures build reproducibility and environment hygiene.

## Response Envelope

Every RPC response includes:
- `receipt`: What happened, hashes, touched paths
- `context_capsule`: Relevant spec/arch/security slices
- `allowed_next_ops`: What you can do next
- `blocked_by`: What's preventing progress

## Standards Resolution

Decapod resolves standards from:
1. **Constitutional Core** - Industry Engineering Excellence (see `ENGINEERING_EXCELLENCE.md`)
2. **Security Standards** - Threat modeling, cryptography, supply chain, SECCOMP (see `architecture/SECURITY.md`)
3. **Coding Standards** - Uncle Bob Martin, Fowler, Pragmatic, GoF, DRY, Unix (see `architecture/CODING_STANDARDS.md`)
4. **Infrastructure** - Cloud patterns, networking, storage (see `architecture/CLOUD.md`)
5. **Data Engineering** - Data modeling, pipelines, governance (see `architecture/DATA.md`)
6. **Quality Assurance** - Testing strategies, TDD, BDD (see `methodology/TESTING.md`)
7. **Project Overrides** - `.decapod/OVERRIDE.md` (project-specific deviations)

Query with: `decapod rpc --op standards.resolve`

## Subsystems

- **todo**: Task tracking with event sourcing
- **workspace**: Branch protection and isolation
- **interview**: Spec/architecture generation
- **federation**: Knowledge graph with provenance
- **validate**: Authoritative completion gates

## Emergency

If Decapod is blocking legitimate work:
1. Check `decapod workspace status`
2. Ensure you're not on main/master
3. Run `decapod validate` to see specific failures
4. Review blockers in RPC response envelope

---

## Links

### Core Entry Points
- [core/DECAPOD.md](core/DECAPOD.md) - **Router and navigation charter (START HERE)** ← You are here
- [core/INTERFACES.md](core/INTERFACES.md) - Interface contracts index
- [core/METHODOLOGY.md](core/METHODOLOGY.md) - Methodology guides index
- [core/PLUGINS.md](core/PLUGINS.md) - Subsystem registry
- [core/ENGINEERING_EXCELLENCE.md](core/ENGINEERING_EXCELLENCE.md) - Engineering standards oracle
- [core/GAPS.md](core/GAPS.md) - Gap analysis methodology

### Governance
- [core/DEMANDS.md](core/DEMANDS.md) - Non-negotiable demands
- [core/DEPRECATION.md](core/DEPRECATION.md) - Deprecation contract
- [core/EMERGENCY_PROTOCOL.md](core/EMERGENCY_PROTOCOL.md) - Emergency procedures

### Architecture (by Domain)
- [architecture/SECURITY.md](architecture/SECURITY.md) - Threat modeling, cryptography, supply chain
- [architecture/CLOUD.md](architecture/CLOUD.md) - Cloud patterns, networking, storage
- [architecture/DATA.md](architecture/DATA.md) - Data modeling, pipelines, governance
- [architecture/CACHING.md](architecture/CACHING.md) - Caching patterns and strategies
- [architecture/OBSERVABILITY.md](architecture/OBSERVABILITY.md) - Observability and monitoring

### Methodology
- [methodology/TESTING.md](methodology/TESTING.md) - Testing strategies, TDD, BDD
- [methodology/CI_CD.md](methodology/CI_CD.md) - CI/CD and release workflow
- [methodology/SOUL.md](methodology/SOUL.md) - Agent identity and behavioral style
