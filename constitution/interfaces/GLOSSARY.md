# GLOSSARY.md - Loaded Terms (Normative)

**Authority:** interface (normative term definitions)
**Layer:** Interfaces
**Binding:** Yes
**Scope:** defines loaded terms used across the doc stack to prevent semantic drift
**Non-goals:** tutorials; this is a reference

---

## Table of Contents

1. [Purpose and Usage](#1-purpose-and-usage)
2. [Core Terms](#2-core-terms)
3. [Document Layer Terms](#3-document-layer-terms)
4. [Interface Terms](#4-interface-terms)
5. [Store and State Terms](#5-store-and-state-terms)
6. [Subsystem Terms](#6-subsystem-terms)
7. [Proof and Validation Terms](#7-proof-and-validation-terms)
8. [Agent Terms](#8-agent-terms)
9. [Lifecycle Terms](#9-lifecycle-terms)
10. [Terminology Consistency Rules](#10-terminology-consistency-rules)

---

This glossary is binding: if a term is defined here, other canonical docs MUST use it consistently.

---

## 1. Purpose and Usage

The Loaded Terms glossary exists to prevent semantic drift — the gradual change in meaning of terms across documents and time.

**When to use this glossary:**
- When writing canonical docs, use defined terms consistently
- When adding new terms, check if a definition already exists
- When encountering ambiguous terms, refer here for meaning

**How definitions are structured:**
- Term (bold)
- Simple definition
- Context and usage notes
- Examples where helpful

---

## 2. Core Terms

### 2.1 Canonical

**Definition:** The repo-relative path in `**Canonical:** ...` identifies the authoritative location of a document.

**Usage:** Canonical does not imply binding; it implies "this path is the source-of-truth for the text."

**Example:**
```markdown
**Canonical:** core/DECAPOD.md
```

### 2.2 Binding

**Definition:** `**Binding:** Yes` means the document defines requirements, invariants, or interfaces. `**Binding:** No` means guidance only; if it conflicts with binding docs, it is wrong.

**Usage:** Binding documents create obligations. Non-binding documents provide guidance.

### 2.3 Layer

**Definition:** The hierarchy position of a document:
- Constitution: authority and behavioral doctrine
- Interfaces: machine surfaces, schemas, invariants, safety gates
- Guides: operational advice; non-binding

**Usage:** Layer determines how conflicts are resolved (Constitution > Interfaces > Guides).

### 2.4 Authority (header field)

**Definition:** A short statement describing what the document is allowed to define (e.g., routing vs interface vs constitution).

**Usage:** Used in doc headers to establish scope and prevent scope creep.

### 2.5 Router (routing authority)

**Definition:** A document that routes readers to canonical sources. A router does not create new behavioral requirements.

**Usage:** `core/DECAPOD.md` is the primary router. See Delegation Charter in DECAPOD.md.

### 2.6 Proof Surface

**Definition:** A named, runnable mechanism that can detect drift or validate invariants (e.g., `decapod validate`, schema checks).

**Usage:** Proof surfaces are the currency of trust. Claims without proof are not enforceable.

### 2.7 Claim

**Definition:** A registered promise/guarantee/invariant with a stable claim-id, tracked in `interfaces/CLAIMS.md`.

**Usage:** Every binding guarantee should have a claim-id for tracking.

### 2.8 Enforcement

**Definition:** Whether a claim is checked by a proof surface:
- `enforced`: proof surface exists and runs
- `partially_enforced`: proof exists but doesn't cover all cases
- `not_enforced`: only documented, not automatically checked

---

## 3. Document Layer Terms

### 3.1 Constitution Layer

**Definition:** The layer of documents that define authority and behavioral doctrine. Rarely edited. Short by design.

**Key documents:** `specs/SYSTEM.md`, `specs/INTENT.md`, `specs/SECURITY.md`

**Usage:** Constitution layer wins in all conflicts.

### 3.2 Interfaces Layer

**Definition:** The layer of documents that define machine surfaces: commands, schemas, store semantics, invariants, and safety gates.

**Key documents:** `interfaces/CLAIMS.md`, `interfaces/CONTROL_PLANE.md`, `interfaces/STORE_MODEL.md`

**Usage:** Interfaces layer defines contracts between components.

### 3.3 Guides Layer

**Definition:** The layer of documents that provide operational guidance. Non-binding.

**Key documents:** `methodology/SOUL.md`, `methodology/ARCHITECTURE.md`, `methodology/TESTING.md`

**Usage:** Guides provide how-to guidance. If a guide conflicts with binding docs, the guide is wrong.

### 3.4 Specs

**Definition:** Specifications that define system behavior, contracts, and requirements. Belong to Constitution or Interfaces layer.

**Usage:** `specs/` directory contains binding requirements.

### 3.5 Architecture

**Definition:** Domain-specific design patterns and practices. May be Guides (methodology) or Interfaces (contracts).

**Usage:** `architecture/` directory contains domain-specific architectural guidance.

---

## 4. Interface Terms

### 4.1 Thin Waist

**Definition:** A constrained interface that all components must pass through. In Decapod, the CLI is the thin waist.

**Usage:** All agent-to-subsystem communication should go through the CLI.

### 4.2 Truth Label

**Definition:** A label indicating the maturity of a subsystem:
- `REAL`: implemented and working
- `STUB`: interface exists, behavior incomplete
- `SPEC`: designed but not implemented
- `IDEA`: exploratory only
- `DEPRECATED`: superseded

**Usage:** Used in subsystem registry to communicate status.

### 4.3 Subsystem

**Definition:** A first-class Decapod surface with a CLI group and schema/proof hooks. See `core/PLUGINS.md`.

**Usage:** Subsystems are registered and tracked in PLUGINS.md.

### 4.4 Plugin-Grade

**Definition:** Meets the thin-waist requirements: stable CLI group, schema/discovery, store-awareness, proof hooks.

**Usage:** Not all subsystems are plugin-grade. Those that aren't are not yet part of the control plane.

### 4.5 Derived (artifact/state)

**Definition:** Computed output that must not be treated as source-of-truth.

**Usage:** Derived artifacts (compiled code, generated docs) should not be edited directly.

### 4.6 Manifest

**Definition:** A record of the inputs and process that produced an artifact. See `plugins/MANIFEST.md`.

**Usage:** Manifests enable reproducibility and audit.

---

## 5. Store and State Terms

### 5.1 Store

**Definition:** A state root that scopes reads/writes. See `interfaces/STORE_MODEL.md`.

**Types:**
- User store: `~/.decapod` (private)
- Repo store: `<repo>/.decapod/project` (shared)

**Usage:** Store is part of request context.

### 5.2 Blank Slate

**Definition:** The guarantee that a fresh user store contains nothing unless the user adds it.

**Usage:** Prevents repo-to-user contamination.

### 5.3 Auto-Seeding

**Definition:** Automatic population of user store from repo store.

**Usage:** Auto-seeding is forbidden (claim: `claim.store.no_auto_seeding`).

### 5.4 Cross-Store Contamination

**Definition:** Content appearing in a store it wasn't intended for.

**Usage:** This is a critical failure.

### 5.5 Store Purity

**Definition:** The property that each store contains only the data intended for it.

**Usage:** Enforced by validation gates.

---

## 6. Subsystem Terms

### 6.1 TODO (work tracking)

**Definition:** The subsystem for tracking work items, ownership, and resolution.

**CLI:** `decapod todo`

**Key concept:** Claim-before-work (must claim TODO before implementation).

### 6.2 Docs (documentation)

**Definition:** The subsystem for navigating canonical documentation.

**CLI:** `decapod docs`

**Key concept:** Doc graph reachability from DECAPOD.md.

### 6.3 Validate (validation)

**Definition:** The primary proof surface that checks documented invariants.

**CLI:** `decapod validate`

**Key concept:** Bounded termination, no cross-turn locks.

### 6.4 Session

**Definition:** The subsystem for managing authenticated sessions.

**CLI:** `decapod session`

**Key concept:** Agent identity + ephemeral password required.

### 6.5 Knowledge

**Definition:** The subsystem for curated knowledge entries.

**CLI:** `decapod data knowledge`

**Key concept:** Provenance required, directional flow enforced.

### 6.6 Federation

**Definition:** The subsystem for federated data with provenance tracking.

**CLI:** `decapod data federation`

**Key concept:** Store-scoped, provenance required for critical, append-only for critical.

---

## 7. Proof and Validation Terms

### 7.1 Validate

**Definition:** The primary proof surface (`decapod validate`) that checks documented invariants and drift gates.

**Usage:** Run validate before claiming correctness.

### 7.2 Proof Surface

**Definition:** A named, runnable mechanism that can detect drift.

**Examples:** `decapod validate`, `cargo test`, `cargo clippy`

### 7.3 Proof Currency

**Definition:** The principle that proof is the currency of trust. If validation exists, run it.

**Usage:** Agents should treat proof as currency.

### 7.4 Amendment

**Definition:** A binding meaning change governed by `specs/AMENDMENTS.md`.

**Usage:** Contradictions are resolved through amendment, not interpretation.

### 7.5 Deprecation

**Definition:** A non-binding marker on old meaning governed by `core/DEPRECATION.md`, with replacement + sunset.

**Usage:** Use deprecation for transitioning between meanings.

---

## 8. Agent Terms

### 8.1 Intent

**Definition:** The user's goal, expressed before implementation begins.

**Usage:** Agents must refine intent with user before inference-heavy work.

### 8.2 Checkpoint

**Definition:** A required Decapod call at a specific point in workflow:
- Before plan commitment (agent.init, context.resolve)
- Before mutation (todo claim, workspace ensure)
- After mutation (validate, test)

**Usage:** Skipping checkpoints invalidates completion claims.

### 8.3 Capability

**Definition:** An ability exposed by the Decapod command surface.

**Usage:** Agents must not claim capabilities absent from the command surface.

### 8.4 Gap

**Definition:** Missing or incomplete specifications, implementations, or capabilities.

**Usage:** Gaps should be reported, not worked around.

### 8.5 Memory

**Definition:** Agent session context and learned residue.

**Usage:** Memory is session-specific; knowledge is curated and shared.

---

## 9. Lifecycle Terms

### 9.1 Claim Lifecycle

**States:** Proposed → Accepted → [Enforced | Partially Enforced | Not Enforced] → Deprecated → Removed

### 9.2 Subsystem Lifecycle

**States:** IDEA → SPEC → STUB → REAL → DEPRECATED → Removed

### 9.3 Gap Lifecycle

**States:** Identified → Categorized → Routed → Documented → Ticketed → In Progress → Resolved → Verified

### 9.4 Knowledge Lifecycle

**States:** Draft → Published → Verified → Maintained → Superseded → Archived

---

## 10. Terminology Consistency Rules

### 10.1 Rule: Use Defined Terms

When a term is defined here, use it consistently. Don't use synonyms that might drift.

### 10.2 Rule: New Terms Need Definitions

Before introducing new loaded terms, add them to this glossary.

### 10.3 Rule: Conflicts Resolve Through Amendment

If two docs use the same term differently, resolve through amendment, not interpretation.

### 10.4 Rule: Proof Before Claims

A claim about system behavior requires proof surface to be credible.

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - **Oracle for Engineering Standards**
- `core/GAPS.md` - Gap analysis methodology

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security contract
- `specs/AMENDMENTS.md` - Change control

### Registry (Core Indices)
- `core/PLUGINS.md` - Subsystem registry
- `core/INTERFACES.md` - Interface contracts index
- `core/METHODOLOGY.md` - Methodology guides index
- `core/DEPRECATION.md` - Deprecation contract

### Contracts (Interfaces Layer - This Document)
- `interfaces/DOC_RULES.md` - Doc compilation rules
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/STORE_MODEL.md` - Store semantics
- `interfaces/CONTROL_PLANE.md` - Sequencing patterns
- `interfaces/TESTING.md` - Testing contract

### Practice (Methodology Layer)
- `methodology/SOUL.md` - Agent identity
- `methodology/ARCHITECTURE.md` - Architecture practice
- `methodology/TESTING.md` - Testing practice
- `methodology/KNOWLEDGE.md` - Knowledge curation
- `methodology/MEMORY.md` - Memory and learning