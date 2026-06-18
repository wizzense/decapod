# Intent

## Product Outcome
- Decapod is the daemonless, local-first governance kernel behind AI coding agents. Agents call it on demand to converge on human intent, shape context before inference, enforce boundaries, and deliver proof-backed completion across concurrent multi-agent work.

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
  U[Primary User] --> P[decapod]
  P --> O[User-visible Outcome]
  P --> G[Proof Gates]
  G --> E[Evidence Artifacts]
```

## Inferred Baseline
- Repository: decapod
- Product type: cli
- Primary languages: Rust
- Detected surfaces: cargo

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
- [ ] Decapod validate passes, required tests pass, and promotion-relevant artifacts are present.
- [ ] Non-functional targets are met (latency, reliability, cost, etc.).
- [ ] Validation gates pass and artifacts are attached.
- [ ] `cargo test` passes for unit/integration coverage
- [ ] `cargo clippy -- -D warnings` passes with no denied lints
- [ ] `cargo fmt --check` passes on the repo

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
