# ARCHITECTURE_FOUNDATIONS.md - Industry-Grade Engineering Foundations

**Authority:** interface (binding architecture directives)  
**Layer:** Interfaces  
**Binding:** Yes  
**Scope:** architecture fundamentals that keep intent alignment and production-grade engineering explicit in the constitution  
**Non-goals:** runtime architecture files under mutable state roots, framework-specific style guides, language-specific implementation detail

## Purpose

Decapod MUST keep architecture guidance in constitution documents and enforce quality through deterministic gates.
Architecture directives are policy, not mutable runtime state.

## Mandatory Primitives

1. **Intent primitive**: governed PLAN defines intent, scope, unknowns, and proof hooks.
2. **Architecture directive primitive**: constitution interfaces define required architecture thinking before promotion.
3. **Proof primitive**: executable checks (`decapod validate`, tests, linters) verify outcomes.

## Golden Path Expectations

For production-grade delivery, agents MUST:

1. Preserve deterministic behavior and typed failure semantics.
2. Maintain explicit boundaries (state, interfaces, ownership) and avoid hidden side effects.
3. Document compatibility and migration impact before promotion.
4. Define verification strategy tied to concrete proof hooks.
5. Keep rollback/remediation path explicit.
6. Make tradeoffs explicit (what was chosen, what was rejected, why).

## Required Architecture Reasoning Surfaces

Architecture reasoning MUST be present in governed artifacts and reviewable evidence, including:

- intent alignment (problem, user outcome, non-goals)
- system design (interfaces, boundaries, data ownership)
- invariants and failure modes
- tradeoffs and risk posture
- verification strategy
- rollout and operations

## Proof Surfaces

- `decapod validate` Plan-Governed Execution Gate enforces plan state, intent resolution, unknown resolution, and verification readiness.
- CI proof surfaces (`cargo fmt`, `cargo clippy`, `cargo test`, `decapod validate`) remain mandatory before promotion.

## Claim Mapping

- `claim.architecture.artifact_required_for_governed_execution`
- `claim.architecture.intent_to_design_traceability`

---

## Links

### Core Router
- [core/DECAPOD.md](core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/INTERFACES.md](core/INTERFACES.md) - Interface contracts index

### Authority (Constitution Layer)
- [specs/INTENT.md](specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](specs/SYSTEM.md) - System definition and authority doctrine

### Contracts (Interfaces Layer)
- [interfaces/CLAIMS.md](interfaces/CLAIMS.md) - **Promises ledger**
- [interfaces/CONTROL_PLANE.md](interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/PLAN_GOVERNED_EXECUTION.md](interfaces/PLAN_GOVERNED_EXECUTION.md) - Plan-governed execution

### Architecture (Domain-Specific)
- [architecture/ALGORITHMS.md](architecture/ALGORITHMS.md) - Algorithm design patterns
- [architecture/CACHING.md](architecture/CACHING.md) - Caching strategies
- [architecture/CLOUD.md](architecture/CLOUD.md) - Cloud architecture
- [architecture/CONCURRENCY.md](architecture/CONCURRENCY.md) - Concurrency patterns
- [architecture/COST_OPTIMIZATION.md](architecture/COST_OPTIMIZATION.md) - Cost optimization
- [architecture/DATA.md](architecture/DATA.md) - Data architecture
- [architecture/DISTRIBUTED_SYSTEMS.md](architecture/DISTRIBUTED_SYSTEMS.md) - Distributed systems
- [architecture/ENCRYPTION.md](architecture/ENCRYPTION.md) - Encryption and security
- [architecture/EVENT_DRIVEN.md](architecture/EVENT_DRIVEN.md) - Event-driven architecture
- [architecture/FRONTEND.md](architecture/FRONTEND.md) - Frontend architecture
- [architecture/INFRASTRUCTURE.md](architecture/INFRASTRUCTURE.md) - Infrastructure patterns
- [architecture/MEMORY.md](architecture/MEMORY.md) - Memory architecture
- [architecture/MICROSERVICES.md](architecture/MICROSERVICES.md) - Microservices patterns
- [architecture/NETWORKING.md](architecture/NETWORKING.md) - Networking patterns
- [architecture/OBSERVABILITY.md](architecture/OBSERVABILITY.md) - Observability
- [architecture/SECRETS.md](architecture/SECRETS.md) - Secrets management
- [architecture/SECURITY.md](architecture/SECURITY.md) - Security architecture
- [architecture/TESTING_STRATEGY.md](architecture/TESTING_STRATEGY.md) - Testing strategy
- [architecture/UI.md](architecture/UI.md) - UI architecture
- [architecture/WEB.md](architecture/WEB.md) - Web architecture
