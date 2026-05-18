# ARCHITECTURE.md - Architecture Practice

**Authority:** guidance (architectural tradeoff evaluation and design workflow)
**Layer:** Guides
**Binding:** No
**Scope:** architectural thinking, tradeoff evaluation, and design workflow
**Non-goals:** test contracts, interface schemas, and binding system rules

---

## Table of Contents

1. [Architecture Mission](#1-architecture-mission)
2. [Core Principles](#2-core-principles)
3. [The Architecture Decision Workflow](#3-the-architecture-decision-workflow)
4. [Tradeoff Evaluation Framework](#4-tradeoff-evaluation-framework)
5. [Domain Map Reference](#5-domain-map-reference)
6. [Layer Boundaries](#6-layer-boundaries)
7. [Architecture Documentation (ADRs)](#7-architecture-documentation-adrs)
8. [Common Architectural Situations](#8-common-architectural-situations)
9. [Architectural Anti-Patterns](#9-architectural-anti-patterns)
10. [Decision Verification and Rollback](#10-decision-verification-and-rollback)

---

## 1. Architecture Mission

Architecture exists to improve delivery outcomes across five dimensions:

| Dimension | What It Means | Why It Matters |
|-----------|---------------|----------------|
| **Velocity** | How fast can we ship? | Competitive advantage, learning speed |
| **Reliability** | Does it work correctly? | User trust, reduced firefighting |
| **Maintainability** | Can we understand and modify it? | Technical debt, onboarding speed |
| **Operability** | Can we run it in production? | Operational cost, incident response |
| **Cost Efficiency** | What's the resource cost? | Business sustainability, scaling economics |

**If a design adds complexity without improving outcomes, reject it.**

Architecture is not about elegance for its own sake. A boring, clear design that solves the problem is superior to an elegant, clever design that creates new problems.

---

## 2. Core Principles

The following principles govern architectural decisions in Decapod-managed repos. These are not suggestions — they are the accumulated lessons from system failures and successes.

### 2.1 Innovation Tokens

**Spend innovation tokens on the product, not the infrastructure.**

Infrastructure complexity must be paid for by every engineer who joins after you. Before introducing new infrastructure components, ask:
- What specific product problem does this solve?
- Could we solve it with boring technology?
- What is the switching cost if this technology fails?

**This does not mean never innovate on infrastructure.** It means be intentional. Every innovation token spent on infrastructure is a token not spent on product differentiation.

### 2.2 Conway's Law

**Conway's Law is descriptive, not prescriptive — but it is enforced.**

Your system architecture will mirror your team communication structure. This is not a suggestion — it is an empirical observation that has held for decades.

**Practical implications:**
- If you want independent deployable services, you need independent teams
- If you want a modular monolith, you need team ownership of modules
- If you want shared infrastructure, you need a platform team
- Fighting Conway's Law leads to architecture that doesn't match how the organization works

**Design the architecture you want, then organize the team to match it.** Deliberate alignment with Conway's Law produces clean, independently deployable boundaries.

### 2.3 Debuggability

**An architecture that cannot be debugged at 3am is a failed architecture.**

Elegance on a whiteboard is not engineering. When production fails at 3am, you need:
- Clear error messages
- Observable system state
- Logged decisions and actions
- Runbooks for common failures
- Known failure modes

**Observability, operational runbooks, and debuggable failure modes are architectural requirements, not afterthoughts.** If a component cannot be reasoned about under pressure, it is not ready for production.

### 2.4 Incremental Migration

**Incremental migration is the only safe migration.**

Any architectural change that cannot be done while the system remains online is too large. The patterns that enable this:
- Strangle pattern: gradually replace old system with new
- Dual-write: write to both old and new, migrate readers
- Feature flags: enable/disable without redeployment
- Parallel run: verify new system before cutting over

**If your change requires a maintenance window, revisit the approach.** The goal is always online, always working, gradually better.

### 2.5 Domain Boundaries

**Domain boundaries matter more than service topology.**

The monolith vs. microservices debate is a distraction. What matters is whether your domain model is correct and whether boundaries are meaningful.

**A well-modularized monolith with clear domain ownership is superior to a distributed system with tangled cross-service data access.** Draw the boundaries correctly, then decide whether to deploy them separately.

### 2.6 Architecture for Deletion

**Architecture must be designed for deletion.**

If removing a feature requires coordinating a dozen services, the boundaries are wrong. Good architecture allows components to be removed cleanly.

**The truest test of isolation is deletion.** Can you remove this component without breaking others? Can you delete this feature in one sprint?

### 2.7 Documentation of Decisions

**Undocumented architecture does not exist.**

An architectural decision that lives only in someone's head has a half-life. Decisions without documentation:
- Cannot be reviewed or challenged
- Cannot be understood by new team members
- Cannot be traced when requirements change
- Will be rediscovered (and possibly reinterpreted) repeatedly

**Capture the context:** what the constraints were, what alternatives were rejected, and why. The code tells you what was built; only the documentation tells you why.

### 2.8 YAGNI Applied

**YAGNI applies to architecture too.**

Do not build generic interfaces, extension mechanisms, or multi-tenant scaffolding for problems you do not have. Premature architectural abstraction is how systems accumulate layers of indirection that no one understands.

**Build for today's requirements first.** Abstract when you have concrete evidence that abstraction is needed, not when you imagine future requirements.

---

## 3. The Architecture Decision Workflow

### 3.1 When to Use This Workflow

This workflow applies to:
- Adding new subsystems
- Changing integration patterns between subsystems
- Selecting new infrastructure components
- Modifying data models that cross domain boundaries
- Any change with significant scope and uncertain tradeoffs

It does not apply to:
- Routine code changes
- Changes within a well-defined domain with existing patterns
- Small, reversible decisions

### 3.2 The Seven-Step Workflow

**Step 1: State the Intent and Impact**

Before evaluating options, clearly articulate:
- What are you trying to accomplish?
- Why does this matter now?
- What are the consequences of not addressing this?

```markdown
# Intent Statement Template
## What
[Clear description of what needs to happen]

## Why Now
[Why this can't wait / what will break]

## Impact If Not Done
[Consequences of inaction]
```

**Step 2: Identify Constraints**

Constraints are fixed requirements that options must satisfy. Categorize them:

| Constraint Type | Examples | How to Handle |
|----------------|----------|---------------|
| **Non-negotiable** | Security requirements, compliance, SLA | Must satisfy, no tradeoffs |
| **Significant** | Scale requirements, latency budgets, team size | Major factor in evaluation |
| **Minor** | Preferences, conventions | Can be traded away |

**Step 3: Define Success Criteria**

How will you know if the architecture is successful? Define measurable criteria before evaluating options:

- Performance: latency, throughput, capacity
- Reliability: availability, error rate, recovery time
- Maintainability: time to understand, ease of change
- Operability: deployment frequency, time to debug
- Cost: infrastructure cost, team cost

**Step 4: Generate and Evaluate Options**

Generate at least three viable options. For each:

```
Option: [Name]
Description: [What it is]
How it satisfies constraints: [Evaluation]
Tradeoffs:
  - Pros: [Benefits]
  - Cons: [Costs]
Risk: [What could go wrong]
Effort: [Implementation complexity]
```

**Step 5: Record Tradeoffs and Select Default**

Document your decision using ADR format (see §7). Include:
- Which option was selected and why
- Which options were rejected and why
- What tradeoffs were accepted

**Step 6: Define Proof Strategy**

How will you verify the architecture works?

| Proof Type | What It Validates | Tools |
|------------|------------------|-------|
| Static validation | Schema contracts, type safety | `decapod validate` |
| Unit tests | Individual component behavior | `cargo test` |
| Integration tests | Cross-component contracts | Integration test suite |
| Performance tests | Non-functional requirements | Benchmarks, load tests |
| Security review | Threat model coverage | Audit, penetration testing |

**Step 7: Define Rollback Path**

For every architectural decision, define:
- What would cause us to roll back?
- How would we rollback?
- What is the cost of rollback?

If you cannot define a rollback path, the change is too risky to proceed.

---

## 4. Tradeoff Evaluation Framework

### 4.1 The Tradeoff Matrix

For each option, evaluate against these dimensions:

| Dimension | Score 1-5 | Why | Can We Live With It? |
|-----------|-----------|-----|---------------------|
| **Simplicity** | | | |
| **Flexibility** | | | |
| **Performance** | | | |
| **Reliability** | | | |
| **Maintainability** | | | |
| **Operability** | | | |
| **Cost** | | | |

### 4.2 Common Tradeoff Patterns

**Simplicity vs. Flexibility**
- Simple systems do one thing well
- Flexible systems handle many cases
- Most systems must trade one for the other
- **Default to simplicity** unless you have concrete evidence flexibility is needed

**Performance vs. Abstraction**
- Abstractions add overhead
- Performance-critical paths may need to bypass abstractions
- **Measure before optimizing** — most code is not on hot paths

**Consistency vs. Availability**
- CAP theorem applies to distributed systems
- Strong consistency requires coordination
- Eventual consistency allows faster responses
- **Choose based on user expectations**, not theoretical purity

**Coupling vs. Independence**
- Tight coupling is simpler to understand initially
- Loose coupling enables independent change
- **Prefer loose coupling** unless integration cost is prohibitive

**Build vs. Buy vs. Open Source**
- Build: full control, full cost
- Buy: faster, dependent on vendor
- Open source: free, but maintenance cost
- **Calculate true cost**, including maintenance and support

### 4.3 Documenting Tradeoffs

For each tradeoff you accept, document:

```markdown
## Tradeoff: [Name]

**What we gain:** [Benefit]
**What we pay:** [Cost]
**When to revisit:** [Trigger condition]
**How to mitigate the cost:** [Mitigation strategy]
```

---

## 5. Domain Map Reference

Use `constitution/architecture/*` documents as deeper references for domain-specific architectural concerns:

### 5.1 Architecture Documents by Domain

| Domain | Document | Key Topics |
|--------|----------|-----------|
| **UI** | `architecture/UI.md` | Component design, state management, rendering patterns |
| **Frontend** | `architecture/FRONTEND.md` | Framework choices, build tooling, performance |
| **Web** | `architecture/WEB.md` | API design, HTTP semantics, web security |
| **Data** | `architecture/DATA.md` | Data modeling, persistence, migration strategies |
| **Security** | `architecture/SECURITY.md` | Threat modeling, security patterns, compliance |
| **Cloud** | `architecture/CLOUD.md` | Deployment, scaling, resilience patterns |
| **Caching** | `architecture/CACHING.md` | Cache strategies, invalidation, consistency |
| **Memory** | `architecture/MEMORY.md` | Memory architecture, retention, eviction |
| **Observability** | `architecture/OBSERVABILITY.md` | Logging, metrics, tracing, alerting |
| **Algorithms** | `architecture/ALGORITHMS.md` | Algorithm selection, complexity analysis |
| **Concurrency** | `architecture/CONCURRENCY.md` | Parallelism, synchronization, deadlock prevention |

### 5.2 When to Consult Domain Architecture Docs

| Situation | Primary Doc | Related Docs |
|-----------|------------|--------------|
| Designing UI components | `architecture/UI.md` | `architecture/FRONTEND.md` |
| Building API layer | `architecture/WEB.md` | `architecture/DATA.md` |
| Defining data model | `architecture/DATA.md` | `architecture/WEB.md`, `methodology/ARCHITECTURE.md` |
| Security review | `architecture/SECURITY.md` | `specs/SECURITY.md` |
| Cloud deployment | `architecture/CLOUD.md` | `methodology/CI_CD.md` |
| Performance optimization | Specific domain doc | `architecture/CONCURRENCY.md` |
| Adding observability | `architecture/OBSERVABILITY.md` | `methodology/METRICS.md` |

---

## 6. Layer Boundaries

This file provides guidance. Binding constraints live elsewhere.

| Layer | Documents | Type | Governs |
|-------|-----------|------|---------|
| **Constitution** | `specs/SYSTEM.md`, `specs/INTENT.md` | Binding | Authority hierarchy, proof doctrine |
| **Interfaces** | `interfaces/CLAIMS.md`, `interfaces/CONTROL_PLANE.md` | Binding | Machine surfaces, guarantees |
| **Guides** | This file, `methodology/*` | Guidance | How to practice architecture |

**Key principle**: If this guide conflicts with a binding document, the binding document wins. This guide is wrong in that case.

**Binding contracts related to architecture:**
- `interfaces/TESTING.md` — Testing contracts
- `interfaces/CONTROL_PLANE.md` — Sequencing patterns
- `interfaces/GLOSSARY.md` — Term definitions
- `core/PLUGINS.md` — Subsystem registry

---

## 7. Architecture Documentation (ADRs)

### 7.1 What Is an ADR

An Architecture Decision Record (ADR) captures an important architectural decision, the context that led to it, and the consequences.

**Why ADRs matter:**
- They preserve context that would otherwise be lost
- They enable future architects to understand past decisions
- They make it possible to review and challenge decisions
- They create a record of the system's evolution

### 7.2 ADR Format

```markdown
# ADR-[NUMBER]: [Title]

**Date:** YYYY-MM-DD
**Status:** Proposed | Accepted | Deprecated | Superseded

## Context

[What is the issue or situation that prompted this decision?]

## Decision

[What is the decision being made?]

## Consequences

### Positive
[What benefits does this decision bring?]

### Negative
[What costs or negative consequences does this decision bring?]

### Tradeoffs Accepted
[What did we explicitly choose not to do?]

## Alternatives Considered

### [Alternative 1]
**Why not:** [Reason for rejection]

### [Alternative 2]
**Why not:** [Reason for rejection]

## Related Decisions

[Links to related ADRs]

## Review Triggers

[What conditions would cause us to revisit this decision?]
```

### 7.3 When to Write an ADR

Write an ADR when:
- The decision affects multiple subsystems
- The decision has significant tradeoffs
- The decision is not easily reversible
- The decision deviates from existing patterns
- The decision was difficult to make

Do not write an ADR when:
- The decision is routine and easily reversible
- The decision only affects one component
- The reasoning is obvious and well-understood

### 7.4 ADR Lifecycle

```
Proposed → Accepted → [Deprecated | Superseded]
    ↑
    └── Review and feedback
```

- **Proposed**: Initial draft, seeking feedback
- **Accepted**: Finalized and in effect
- **Deprecated**: No longer preferred, but not removed
- **Superseded**: Replaced by another ADR

---

## 8. Common Architectural Situations

### 8.1 Adding a New Subsystem

**Workflow:**
1. State intent and impact
2. Define subsystem boundaries (what it owns, what it doesn't)
3. Define interfaces with existing subsystems
4. Select implementation approach
5. Plan migration path if replacing existing approach
6. Define proof strategy

**Common mistakes:**
- Building too much scope into the new subsystem
- Not defining clear interfaces with neighbors
- Not planning for data migration if replacing existing functionality

### 8.2 Changing Integration Patterns

**Workflow:**
1. Map current integration flow
2. Identify all consumers
3. Define new interface contract
4. Plan migration (parallel run, feature flag, or strangle)
5. Implement new integration
6. Validate with all consumers
7. Decommission old integration

**Common mistakes:**
- Not identifying all consumers
- Not having rollback plan
- Breaking changes without deprecation period

### 8.3 Selecting Infrastructure Components

**Workflow:**
1. Define requirements (performance, scale, operational needs)
2. Evaluate options against requirements
3. Consider operational complexity
4. Assess vendor/supplier risk
5. Plan for data portability
6. Define exit strategy

**Common mistakes:**
- Selecting based on features without considering operational cost
- Not planning for vendor lock-in
- Underestimating migration cost

### 8.4 Data Model Changes

**Workflow:**
1. Analyze current data model and usage
2. Define new model
3. Plan migration path
4. Implement new model with backward compatibility
5. Migrate data
6. Remove legacy model

**Common mistakes:**
- Not considering impact on existing queries
- Insufficient rollback plan
- Not testing with production-scale data

---

## 9. Architectural Anti-Patterns

### 9.1 Big Ball of Mud

**What it is:** A system with no discernible structure, where everything is coupled to everything.

**Symptoms:**
- Any change affects many parts of the system
- Fear of making changes (even small ones)
- Duplicated logic scattered across the codebase
- No clear boundaries between features

**How it happens:**
- Evolutionary growth without upfront design
- Short-term speed at the expense of structure
- Ignoring Conway's Law (team structure doesn't match architecture)

**How to fix:**
- Identify natural domains and boundaries
- Introduce seams (interfaces between modules)
- Apply strangler pattern to migrate domain by domain
- Invest in testing to prevent regressions

### 9.2 Bridge Pattern Abuse

**What it is:** Excessive layers of abstraction to the point where understanding the system requires tracing through many indirection layers.

**Symptoms:**
- "Just one more abstraction layer" requests
- Finding the actual implementation requires following five levels of interfaces
- Developers confused about which abstraction to use
- Interface methods that just delegate to another interface

**How it happens:**
- Over-engineering for future flexibility
- YAGNI violations
- Adding abstraction to solve a problem that doesn't exist yet

**How to fix:**
- Collapse unnecessary layers
- Make implementation details visible
- Prefer composition over excessive abstraction

### 9.3 Database as IPC

**What it is:** Using the database as a communication mechanism between services/components instead of proper API calls.

**Symptoms:**
- Components read directly from tables owned by other components
- Schema changes require coordination across teams
- Circular dependencies hidden in foreign keys
- "Eventual consistency" as excuse for asynchronous database coupling

**How it happens:**
- Convenience of direct data access
- "It's just a quick query"
- Distributed system without proper API design

**How to fix:**
- Define proper API boundaries
- Create explicit data ownership
- Use events for async communication
- Treat shared schema like shared library API

### 9.4 Synchronous Islands

**What it is:** Multiple services that appear independent but are actually tightly coupled through synchronous calls, creating distributed monolith.

**Symptoms:**
- One service failure cascades to many
- Can't deploy one service without others
- "Microservices" that require all-or-nothing deployment
- Latency compounds across service boundaries

**How it happens:**
- Treating microservices as distributed monolith
- Synchronous everywhere
- Ignoring circuit breaker patterns

**How to fix:**
- Introduce async communication where possible
- Implement circuit breakers
- Design for independent deployability
- Consider whether true microservices are needed

### 9.5 Reinventing the Wheel

**What it is:** Building custom solutions for problems that have established, well-tested solutions.

**Symptoms:**
- Custom encryption instead of TLS
- Custom authentication instead of established protocols
- Custom queuing instead of message broker
- Custom retry logic instead of established patterns

**How it happens:**
- "Not invented here" syndrome
- Believing custom solution is better
- Not knowing what established solutions exist

**How to fix:**
- Research existing solutions before building
- Prefer boring technology for infrastructure
- Build custom only when established solutions don't fit

---

## 10. Decision Verification and Rollback

### 10.1 Verification Strategy

For each architectural decision, define:

| Verification Type | When | How |
|------------------|------|-----|
| **Immediate validation** | After implementation | Run proof surfaces (`decapod validate`) |
| **Short-term monitoring** | First week | Watch for unexpected behavior |
| **Long-term validation** | After 3 months | Review against success criteria |
| **Cost validation** | After 6 months | Measure actual vs. projected costs |

### 10.2 Rollback Triggers

Define explicit conditions that would trigger rollback:

- Performance degrades below threshold
- Error rate increases beyond acceptable level
- Operational cost exceeds projection by >50%
- New information invalidates core assumptions

### 10.3 Rollback Planning

For every significant architectural change, document:

**What to rollback:**
- Code changes (revert to previous version)
- Data migration (restore previous schema)
- Configuration changes (revert to previous config)
- Infrastructure changes (teardown new resources)

**How to rollback:**
1. Document the rollback procedure
2. Test the rollback procedure before going to production
3. Ensure rollback doesn't lose data
4. Define notification process for rollback

**How long rollback takes:**
- Target: < 30 minutes for full rollback
- If rollback takes longer, the change is too risky

---

## Links

### Core Router
- [core/DECAPOD.md](../../core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](../../core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards (CTO->Principal)**
- [core/GAPS.md](../../core/GAPS.md) - Gap analysis methodology
- [core/METHODOLOGY.md](../../core/METHODOLOGY.md) - Methodology guides index

### Authority (Constitution Layer)
- [specs/INTENT.md](../specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](../specs/SYSTEM.md) - System definition and authority doctrine
- [specs/SECURITY.md](../specs/SECURITY.md) - Security contract
- [specs/GIT.md](../specs/GIT.md) - Git etiquette contract
- [specs/AMENDMENTS.md](../specs/AMENDMENTS.md) - Change control

### Registry (Core Indices)
- [core/PLUGINS.md](../../core/PLUGINS.md) - Subsystem registry
- [core/INTERFACES.md](../../core/INTERFACES.md) - Interface contracts index
- [core/DEPRECATION.md](../../core/DEPRECATION.md) - Deprecation contract

### Contracts (Interfaces Layer)
- [interfaces/TESTING.md](../../interfaces/TESTING.md) - Testing contract
- [interfaces/CONTROL_PLANE.md](../../interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/CLAIMS.md](../../interfaces/CLAIMS.md) - Promises ledger
- [interfaces/GLOSSARY.md](../../interfaces/GLOSSARY.md) - Term definitions
- [interfaces/DOC_RULES.md](../../interfaces/DOC_RULES.md) - Doc compilation rules

### Practice (Methodology Layer - This Document)
- [methodology/SOUL.md](./SOUL.md) - Agent identity
- [methodology/KNOWLEDGE.md](./KNOWLEDGE.md) - Knowledge curation
- [methodology/MEMORY.md](./MEMORY.md) - Memory and learning
- [methodology/TESTING.md](./TESTING.md) - Testing practice
- [methodology/CI_CD.md](./CI_CD.md) - CI/CD practice

### Domain Architecture Patterns
- [architecture/UI.md](architecture/UI.md) - **UI architecture patterns and component design**
- [architecture/FRONTEND.md](architecture/FRONTEND.md) - Frontend architecture patterns
- [architecture/WEB.md](architecture/WEB.md) - Web architecture patterns
- [architecture/DATA.md](architecture/DATA.md) - Data architecture patterns
- [architecture/SECURITY.md](architecture/SECURITY.md) - Security architecture patterns
- [architecture/CLOUD.md](architecture/CLOUD.md) - Cloud deployment patterns
- [architecture/CACHING.md](architecture/CACHING.md) - Caching architecture patterns
- [architecture/MEMORY.md](architecture/MEMORY.md) - Memory architecture patterns
- [architecture/OBSERVABILITY.md](architecture/OBSERVABILITY.md) - Observability patterns
- [architecture/CONCURRENCY.md](architecture/CONCURRENCY.md) - Concurrency patterns
- [architecture/ALGORITHMS.md](architecture/ALGORITHMS.md) - Algorithm patterns

### Operations (Plugins Layer)
- [plugins/TODO.md](../plugins/TODO.md) - Work tracking
- [plugins/VERIFY.md](../plugins/VERIFY.md) - Validation subsystem
- [plugins/DECIDE.md](../plugins/DECIDE.md) - Architecture decision prompting
- [plugins/MANIFEST.md](../plugins/MANIFEST.md) - Manifest patterns

---

## Project Override Context

**Project architecture emphasis:**
- Organize by responsibility domains (agent loop, channels, tools, storage, orchestration)
- Keep service-specific logic at the edge; preserve a reusable core
- Use interface contracts and state transitions to reduce hidden coupling
- Prefer evolvable extension points over one-off feature branches in core flow
- Design for testability: if it's hard to test, the design is wrong

**Current architectural challenges:**
- Balancing core stability with extension flexibility
- Managing state transitions across distributed components
- Ensuring observability without adding excessive overhead

**Architecture review process:**
- All significant architectural decisions require ADR
- ADRs reviewed by at least one architect
- Implementation must include proof surfaces