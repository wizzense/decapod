# METHODOLOGY.md - Methodology Guides Registry

**Authority:** guidance (how-to guides and practice documents)
**Layer:** Guides
**Binding:** No
**Scope:** canonical index of methodology guidance
**Non-goals:** binding contracts and schema definitions

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Methodology Guides](#2-methodology-guides)
3. [Guide Consumption Patterns](#3-guide-consumption-patterns)
4. [Guide Authoring Standards](#4-guide-authoring-standards)
5. [Boundary Rule](#5-boundary-rule)
6. [Cross-Guide Dependencies](#6-cross-guide-dependencies)
7. [Guide Evolution](#7-guide-evolution)
8. [Anti-Patterns](#8-anti-patterns)
9. [Specialized Domains](#9-specialized-domains)
10. [Extraction Status](#10-extraction-status)

---

## 1. Introduction

Methodology guides are the operational conscience of the Decapod system. Unlike binding contracts in `specs/` and `interfaces/`, these guides exist to encode **practice** — the accumulated knowledge of what works, what breaks, and why. They teach execution behavior without creating legal obligations.

A methodology guide answers the question: "Given that I know what the system requires, how do I actually execute in this situation?"

The guides are designed to be:
- **Actionable**: step-by-step workflows with specific commands
- **Contextual**: when to use this approach vs. alternatives
- **Honest about tradeoffs**: what you gain, what you lose, what breaks
- **Illustrated**: examples of both success and failure modes
- **Linked**: every guide references related guides and binding contracts

The distinction between guidance and binding law is not a suggestion. If a guide conflicts with a binding document, the binding document wins. This is enforced by `decapod validate` for structural elements, and by human review for semantic conflicts.

---

## 2. Methodology Guides

| Document | Purpose | Primary Audience |
|----------|---------|------------------|
| `methodology/ARCHITECTURE.md` | Architectural tradeoff evaluation and design workflow practice | Architects, Principal Engineers |
| `methodology/SOUL.md` | Agent identity, communication style, and collaboration posture | All agents |
| `methodology/KNOWLEDGE.md` | Knowledge capture, curation, and lifecycle hygiene | All agents |
| `methodology/MEMORY.md` | Memory hygiene, retrieval discipline, and retention policies | All agents |
| `methodology/TESTING.md` | Testing workflow, pyramid emphasis, and quality assurance practice | All engineers |
| `methodology/CI_CD.md` | CI/CD pipeline patterns, release hygiene, and deployment safety | DevOps, Release Engineers |
| `architecture/UI.md` | UI architecture patterns and component design | Frontend Engineers |
| `methodology/INCIDENT_RESPONSE.md` | Incident detection, escalation, and post-mortem practice | On-call Engineers |
| `methodology/RELEASE_MANAGEMENT.md` | Release planning, versioning, and change coordination | Release Managers |
| `methodology/METRICS.md` | Metric collection, alerting philosophy, and observability | SRE, Platform Engineers |

---

## 3. Guide Consumption Patterns

### 3.1 When to Consult a Guide

Not every task requires reading a methodology guide. The following signals indicate guide consultation is valuable:

**High-Value Guide Consumption Triggers:**
- First time performing a particular class of task (e.g., first architecture decision, first incident)
- Encountering a non-obvious failure mode that seems systemic
- Uncertainty about which subsystem to use for a given problem
- Receiving conflicting signals from different parts of the system
- Preparing to make a multi-step change with uncertain outcomes
- Onboarding to a new domain or responsibility area
- Writing a new methodology guide (meta-circular consumption)

**Low-Value Guide Consumption Triggers:**
- Routine tasks with established patterns
- Tasks that are explicitly routed by other documents
- Situations where the binding contracts are unambiguous

### 3.2 How to Read a Guide

Each guide follows a standard structure designed for skimming and targeted retrieval:

1. **Header Block**: Authority, Layer, Binding, Scope — determines applicability
2. **Mission Statement**: What problem this guide solves, in one paragraph
3. **Core Principles**: 3-5 principles that govern all subsequent guidance
4. **Practical Workflows**: Numbered steps for specific scenarios
5. **Examples**: Both success cases and failure modes with context
6. **Anti-Patterns**: Explicit warnings about what NOT to do and why
7. **Links Section**: Navigation to related documents

**Reading Order Recommendation:**
1. Read the Mission Statement first — confirm the guide is relevant
2. Scan Core Principles for the governing philosophy
3. Find the specific workflow or scenario most relevant to your task
4. Read the anti-patterns — these often clarify the principles
5. Check the Links section for related guidance

### 3.3 Guide Authority Boundaries

Methodology guides are explicitly non-binding. This has concrete implications:

**What Guides CAN Do:**
- Suggest workflows with SHOULD, PREFER, CONSIDER language
- Provide examples that illustrate successful patterns
- Describe tradeoffs without mandating choices
- Offer heuristics that work in common cases
- Acknowledge uncertainty and edge cases

**What Guides MUST NOT Do:**
- Use MUST, SHALL, REQUIRED for new requirements
- Create invariants that are not in `interfaces/CLAIMS.md`
- Define subsystem behavior that belongs in `core/PLUGINS.md`
- Contradict binding documents (guide is wrong in this case)
- Create proof obligations not registered in CLAIMS

---

## 4. Guide Authoring Standards

### 4.1 When to Create a New Guide

A new methodology guide should be created when:

1. **Recurring Scenario**: A class of tasks occurs frequently enough to warrant documented practice
2. **Non-Obvious Execution**: The correct approach is not apparent from first principles
3. **Tradeoff Complexity**: Multiple options exist with significant tradeoffs that require context to navigate
4. **Failure Pattern**: Similar failures occur that can be prevented with better guidance
5. **Knowledge Preservation**: Institutional knowledge about execution exists only in people's heads

**Indicators That a Guide is Needed:**
- Agents repeatedly ask the same clarifying questions
- Similar tasks are executed inconsistently by different agents
- Failure modes repeat across unrelated changes
- Onboarding to a domain requires extensive verbal explanation
- A TODO or issue pattern suggests a practice gap

### 4.2 Required Elements of a Methodology Guide

Every methodology guide MUST include:

**Header Block:**
```markdown
# GUIDE_NAME.md - Short Description

**Authority:** guidance (one-line description of what this guide covers)
**Layer:** Guides
**Binding:** No
**Scope:** what this guide covers
**Non-goals:** what this guide explicitly does NOT cover
```

**Mission Statement (§1):**
One paragraph explaining what problem this guide solves and why the guidance exists.

**Core Principles (§2):**
3-5 governing principles with explanations of WHY they exist. These are the reasoning behind the practice, not just the practice itself.

**Practical Workflows (§3):**
Numbered steps for common scenarios. Each step should include:
- What to do
- Why to do it (brief)
- What can go wrong

**Examples (§4):**
At least two examples:
1. A success case showing correct application of the guide
2. A failure case showing what breaks and why

**Anti-Patterns (§5):**
Explicit warnings about what NOT to do, with explanations of failure modes.

**Links Section (§N):**
Complete links section with Core Router, Authority, Registry, Contracts, Practice, and Operations links.

### 4.3 Style Guidelines

**Tone:**
- Direct and practical, not academic
- Uses active voice ("Run `decapod validate`" not "Validation should be run")
- Acknowledges uncertainty and edge cases honestly
- Explains the reasoning behind recommendations

**Terminology:**
- Use terms consistently as defined in `interfaces/GLOSSARY.md`
- Avoid jargon unless it's the accepted term in the domain
- Define domain-specific terms when first used

**Examples:**
- Include specific commands, not just descriptions
- Show actual output (or realistic mock output) when instructive
- Include error messages and what they mean

**Formatting:**
- Code blocks for commands and code
- Tables for comparisons and registries
- Numbered lists for workflows
- Bold for key terms and critical warnings

---

## 5. Boundary Rule

Methodology guides occupy a specific layer in the document hierarchy:

```
┌─────────────────────────────────────────────────────────────┐
│ Constitution Layer (specs/) - Binding Authority             │
│ - INTENT.md: methodology contract                           │
│ - SYSTEM.md: system definition and authority doctrine       │
│ - GIT.md: git workflow contract                             │
│ - SECURITY.md: security contract                            │
│ - AMENDMENTS.md: change control process                     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Interfaces Layer (interfaces/) - Binding Machine Surfaces   │
│ - CONTROL_PLANE.md: sequencing patterns                    │
│ - CLAIMS.md: promise registry                              │
│ - STORE_MODEL.md: state semantics                          │
│ - DOC_RULES.md: compilation rules                          │
│ - GLOSSARY.md: term definitions                            │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│ Guides Layer (methodology/, architecture/) - Non-Binding    │
│ - SOUL.md: agent identity and behavior                     │
│ - ARCHITECTURE.md: architectural decision practice          │
│ - TESTING.md: testing workflow                              │
│ - CI_CD.md: delivery automation practice                   │
│ - KNOWLEDGE.md: knowledge curation                         │
│ - MEMORY.md: memory hygiene                                 │
│ - UI.md: UI architecture patterns                          │
└─────────────────────────────────────────────────────────────┘
```

**The boundary rule in practice:**

1. **If a binding document is ambiguous**, methodology guides provide contextual interpretation, but the interpretation must be consistent with the binding document's intent.

2. **If a guide conflicts with a binding document**, the binding document wins. The guide should be updated to reflect this.

3. **If a guide would create a new requirement**, the requirement must be registered in `interfaces/CLAIMS.md` and potentially elevated to an interface or spec.

4. **If a binding document references a guide**, the guide should be expanded to fully support that reference.

---

## 6. Cross-Guide Dependencies

Methodology guides form a dependency graph. Understanding these dependencies helps navigate the guide system effectively.

### 6.1 Primary Dependency Chain

```
SOUL.md (identity)
    │
    ├──► ARCHITECTURE.md (how to make decisions)
    │         │
    │         ├──► TESTING.md (how to verify decisions)
    │         │
    │         └──► CI_CD.md (how to deliver decisions)
    │
    ├──► KNOWLEDGE.md (how to preserve context)
    │
    └──► MEMORY.md (how to learn from experience)
```

### 6.2 Domain-Specific Guides

```
architecture/UI.md
    │
    ├──► methodology/SOUL.md (component identity)
    │
    └──► methodology/ARCHITECTURE.md (architectural principles)
```

```
architecture/WEB.md
    │
    ├──► methodology/ARCHITECTURE.md (API design principles)
    │
    └──► methodology/TESTING.md (integration testing patterns)
```

### 6.3 Cross-Guide Reference Patterns

When one guide references another, the reference should include:
- Document path
- Specific section (if applicable)
- Brief explanation of why the reference is relevant

**Example reference:**
> For memory hygiene patterns, see `methodology/MEMORY.md` §3 (Retrieval Discipline). The key insight is that memory should be pointers and residue, not comprehensive logs.

---

## 7. Guide Evolution

### 7.1 When to Update a Guide

Methodology guides should be updated when:

1. **Practice Changes**: The recommended approach has changed due to new tools, patterns, or understanding
2. **Failure Patterns Emerge**: Common failures suggest the current guidance is incomplete or incorrect
3. **Binding Documents Change**: When interfaces or specs change, guides that reference them must be updated
4. **New Examples Emerge**: Real-world examples (success or failure) should be captured
5. **Scope Expands**: A guide that was narrow grows to cover more territory

### 7.2 Update Process

1. **Read the current guide** in full
2. **Check binding documents** for relevant changes
3. **Identify specific sections** that need updating
4. **Draft changes** following the authoring standards
5. **Verify links** are still accurate
6. **Run validation**: `decapod validate` for structural validity
7. **Submit changes** following the amendment process for binding elements

### 7.3 Versioning and Changelog

For significant updates to methodology guides:
- Note the change in the document header (optional, not required for guides)
- Include a brief "Recent Changes" note if the guide has changed substantially
- If the change affects cross-guide dependencies, note the affected guides

---

## 8. Anti-Patterns

### 8.1 Guide Anti-Patterns

**The "Me Too" Guide**
- Copies structure from other guides without understanding why
- Includes generic advice that applies to any workflow
- Fails to capture domain-specific knowledge

**The Encyclopedia Guide**
- Attempts to cover every possible scenario
- Becomes so long that no one reads it
- Loses focus on the core mission

**The Command Manual**
- Lists commands without explaining when to use them
- Missing the "why" behind each step
- Becomes obsolete quickly as commands change

**The Contractual Guide**
- Uses MUST/SHALL language inappropriately
- Creates requirements without registering them
- Conflicts with binding documents

**The Orphaned Guide**
- No links to other documents
- No references from other documents
- Content becomes stale without anyone noticing

### 8.2 Consumption Anti-Patterns

**Guide Worship**
- Following a guide blindly without understanding the reasoning
- Applying guide recommendations to inappropriate contexts
- Treating guidance as binding when it is not

**Guide Rejection**
- Ignoring methodology guides entirely
- Assuming old patterns are still valid
- Dismissing guidance because "it doesn't apply here"

**Selective Consumption**
- Reading only the parts that confirm existing beliefs
- Ignoring anti-patterns and failure modes
- Taking examples out of context

### 8.3 Creation Anti-Patterns

**Requirements Creep**
- Adding binding requirements to a non-binding guide
- Registering claims without proper proof surfaces
- Contradicting binding documents

**Example Avoidance**
- Writing theoretical guidance without concrete examples
- Hiding failure modes instead of explaining them
- Avoiding discussion of tradeoffs

---

## 9. Specialized Domains

### 9.1 Architecture Practice

`methodology/ARCHITECTURE.md` is the primary guide for architectural decisions. It covers:

- Decision workflow (intent → constraints → options → tradeoffs → proof)
- Domain map navigation (data, caching, memory, web, cloud, etc.)
- Conway's Law alignment
- Migration-first design
- Debuggability requirements

**For domain-specific architecture:**
- `architecture/UI.md` — UI components, state management, rendering patterns
- `architecture/FRONTEND.md` — Frontend-specific architectural concerns
- `architecture/WEB.md` — API design, HTTP semantics, web security
- `architecture/DATA.md` — Data modeling, persistence, migration
- `architecture/SECURITY.md` — Threat modeling, security patterns
- `architecture/CLOUD.md` — Cloud deployment, scaling, resilience

### 9.2 Quality Assurance

`methodology/TESTING.md` covers the testing pyramid and change-coupled testing:

- Unit, integration, and E2E balance
- Test naming conventions
- Flaky test handling
- Evidence and reporting

**For binding testing contracts:**
- `interfaces/TESTING.md` — Machine-readable testing interface definitions
- `plugins/VERIFY.md` — Validation subsystem proof surfaces

### 9.3 Delivery Automation

`methodology/CI_CD.md` covers CI/CD pipelines and release hygiene:

- PR validation stages
- CD rollout strategies
- Branch hygiene
- Secret management

**For binding release contracts:**
- `specs/GIT.md` — Git workflow and branch management
- `plugins/VERIFY.md` — Proof surfaces for release validation

### 9.4 Knowledge and Memory

`methodology/KNOWLEDGE.md` and `methodology/MEMORY.md` together form the learning subsystem:

**Knowledge Management (KNOWLEDGE.md):**
- Capture discipline
- Curation workflow
- Lifecycle hygiene
- Provenance tracking

**Memory Management (MEMORY.md):**
- Memory creation and retrieval
- Confidence weighting
- Pruning and consolidation
- Distillation practices

**For binding knowledge contracts:**
- `interfaces/KNOWLEDGE_SCHEMA.md` — Schema definitions
- `interfaces/MEMORY_SCHEMA.md` — Memory schema definitions
- `interfaces/KNOWLEDGE_STORE.md` — Knowledge store semantics

### 9.5 Agent Identity and Behavior

`methodology/SOUL.md` defines agent persona and interaction patterns:

- Communication style (concise, precise, no artificial certainty)
- Behavioral defaults (smallest change, explicit assumptions)
- Boundary awareness (error handling in `EMERGENCY_PROTOCOL.md`)

**For emergency and error handling:**
- `core/EMERGENCY_PROTOCOL.md` — Emergency escalation procedures

---

## 10. Extraction Status

Dedicated files created for previously spliced contract content:

| Extracted Document | Source | Reason |
|--------------------|--------|--------|
| `interfaces/TESTING.md` | Was embedded in methodology/TESTING.md | Binding machine surface needed separation |
| `core/EMERGENCY_PROTOCOL.md` | Was embedded in various docs | Emergency procedures needed dedicated canonical location |
| `interfaces/KNOWLEDGE_SCHEMA.md` | Was embedded in methodology/KNOWLEDGE.md | Binding schema needed separation |
| `interfaces/MEMORY_SCHEMA.md` | Was embedded in methodology/MEMORY.md | Binding schema needed separation |
| `interfaces/DEMANDS_SCHEMA.md` | Was embedded in core/DEMANDS.md | Binding schema needed separation |

---

## Links

### Core Router
- [core/DECAPOD.md](core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards (CTO->Principal)**
- [core/GAPS.md](core/GAPS.md) - Gap analysis methodology

### Authority (Constitution Layer)
- [specs/INTENT.md](specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](specs/SYSTEM.md) - System definition and authority doctrine
- [specs/SECURITY.md](specs/SECURITY.md) - Security contract
- [specs/GIT.md](specs/GIT.md) - Git etiquette contract
- [specs/AMENDMENTS.md](specs/AMENDMENTS.md) - Change control

### Registry (Core Indices)
- [core/PLUGINS.md](core/PLUGINS.md) - Subsystem registry
- [core/INTERFACES.md](core/INTERFACES.md) - Interface contracts index
- [core/DEPRECATION.md](core/DEPRECATION.md) - Deprecation contract
- [core/DEMANDS.md](core/DEMANDS.md) - User demand patterns

### Contracts (Interfaces Layer)
- [interfaces/CONTROL_PLANE.md](interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/DOC_RULES.md](interfaces/DOC_RULES.md) - Doc compilation rules
- [interfaces/CLAIMS.md](interfaces/CLAIMS.md) - Promises ledger
- [interfaces/GLOSSARY.md](interfaces/GLOSSARY.md) - Term definitions
- [interfaces/STORE_MODEL.md](interfaces/STORE_MODEL.md) - Store semantics
- [interfaces/TESTING.md](interfaces/TESTING.md) - Testing contract

### Practice (Methodology Layer - This Registry)
- [methodology/SOUL.md](methodology/SOUL.md) - Agent identity and behavioral style
- [methodology/ARCHITECTURE.md](methodology/ARCHITECTURE.md) - Architecture practice
- [methodology/KNOWLEDGE.md](methodology/KNOWLEDGE.md) - Knowledge curation
- [methodology/MEMORY.md](methodology/MEMORY.md) - Memory and learning
- [methodology/TESTING.md](methodology/TESTING.md) - Testing practice and quality workflow
- [methodology/CI_CD.md](methodology/CI_CD.md) - CI/CD and release workflow practice

### Architecture Patterns (Domain Layer)
- [architecture/FRONTEND.md](../architecture/FRONTEND.md) - Frontend architecture patterns
- [architecture/WEB.md](../architecture/WEB.md) - Web architecture patterns
- [architecture/DATA.md](../architecture/DATA.md) - Data architecture patterns
- [architecture/SECURITY.md](../architecture/SECURITY.md) - Security architecture patterns
- [architecture/CLOUD.md](../architecture/CLOUD.md) - Cloud deployment patterns
- [architecture/CACHING.md](../architecture/CACHING.md) - Caching architecture patterns
- [architecture/MEMORY.md](../architecture/MEMORY.md) - Memory architecture patterns
- [architecture/OBSERVABILITY.md](../architecture/OBSERVABILITY.md) - Observability patterns

### Operations (Plugins Layer)
- [plugins/TODO.md](plugins/TODO.md) - Work tracking
- [plugins/VERIFY.md](plugins/VERIFY.md) - Validation subsystem
- [plugins/MANIFEST.md](plugins/MANIFEST.md) - Manifest patterns
- [plugins/KNOWLEDGE.md](plugins/KNOWLEDGE.md) - Knowledge subsystem
- [plugins/FEDERATION.md](plugins/FEDERATION.md) - Federation subsystem
- [plugins/EMERGENCY_PROTOCOL.md](plugins/EMERGENCY_PROTOCOL.md) - Emergency protocols