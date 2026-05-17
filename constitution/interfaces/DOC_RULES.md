# DOC_RULES.md - Doc Compiler Contract

**Authority:** interface (doc compilation rules)
**Layer:** Interfaces
**Binding:** Yes

---

## Table of Contents

1. [Purpose and Scope](#1-purpose-and-scope)
2. [Canonical Doc Header (Required)](#2-canonical-doc-header-required)
3. [Layers (Meaning)](#3-layers-meaning)
4. [Links Footer (Graph Contract)](#4-links-footer-graph-contract)
5. [Subsystem Truth (Single Source)](#5-subsystem-truth-single-source)
6. [Truth Labels (For Interfaces)](#6-truth-labels-for-interfaces)
7. [No Duplicate Authority](#7-no-duplicate-authority)
8. [Claims Ledger (Promises Must Be Registered)](#8-claims-ledger-promises-must-be-registered)
9. [Decision Rights Matrix (Authority Routing)](#9-decision-rights-matrix-authority-routing)
10. [Compliance Verification](#10-compliance-verification)

---

This document defines how markdown behaves as a machine interface in Decapod-managed repos.

If a rule is not declared here, it is not enforceable (claim: `claim.doc.no_shadow_policy`). If it is declared here, it is intended to become enforceable (via `decapod validate`).

---

## 1. Purpose and Scope

The Doc Compiler Contract serves two purposes:

1. **Define structural requirements** that can be machine-verified
2. **Establish the document graph** that enables navigation

**What this contract governs:**
- Document header format
- Layer classification meaning
- Link graph requirements
- Truth label usage
- Authority routing

**What this contract does not govern:**
- Content of documents (that's the owner's job)
- Methodology guidance (that's the Guides layer)
- Subsystem behavior (that's PLUGINS.md)

---

## 2. Canonical Doc Header (Required)

Every canonical doc under `constitution/` MUST include the following header fields (exact spelling):

### 2.1 Required Fields

| Field | Description | Example |
|-------|-------------|---------|
| `**Canonical:**` | Repo-relative path to this doc | `core/DECAPOD.md` |
| `**Authority:**` | Short role describing what this doc defines | `routing (navigation charter)` |
| `**Layer:**` | Hierarchy position | `Constitution \| Interfaces \| Guides` |
| `**Binding:**` | Whether violations block claims | `Yes \| No` |

### 2.2 Optional Fields

| Field | Description | Example |
|-------|-------------|---------|
| `**Scope:**` | What this doc is allowed to define | `canonical index of subsystem surfaces` |
| `**Non-goals:**` | What it must not define | `tutorial workflows and architecture doctrine` |

### 2.3 Example Headers

**Binding Interface Document:**
```markdown
# PLUGINS.md - Subsystem Registry

**Authority:** interface (subsystem truth registry)
**Layer:** Interfaces
**Binding:** Yes
**Scope:** canonical list of subsystem surfaces, status, truth labels, and deprecation routing
**Non-goals:** tutorial workflows and architecture doctrine
```

**Non-Binding Guide:**
```markdown
# SOUL.md - Agent Identity & Behavioral Style

**Authority:** guidance (agent persona and interaction style)
**Layer:** Guides
**Binding:** No
**Scope:** identity, communication style, and operating posture
**Non-goals:** emergency procedures, failure protocol contracts, or system authority rules
```

---

## 3. Layers (Meaning)

Each document must be classified into exactly one layer.

### 3.1 Constitution Layer

**Definition:** Defines authority and behavior. Rarely edited. Short by design.

**Authority keywords:** `constitution`, `authority`, `doctrine`

**Allowed:**
- Authority hierarchy
- Proof doctrine
- Agent persona/interaction contract
- Methodology contract (intent-first flow)

**Forbidden:**
- Enumerating subsystem commands
- Describing storage layouts in detail
- Describing planned features as if implemented

### 3.2 Interfaces Layer

**Definition:** Defines machine surfaces: commands, schemas, store semantics, invariants, and safety gates.

**Authority keywords:** `interface`, `registry`, `contract`, `patterns`

**Allowed:**
- Subsystem registry and truth labeling
- Interface envelopes and schema surfaces
- Store selection and purity model
- Validate taxonomy and coverage matrix

**Forbidden:**
- Tutorial prose that introduces new requirements (route to Guides instead)
- Methodology guidance

### 3.3 Guides Layer

**Definition:** Operational guidance only. Guides may be verbose.

**Authority keywords:** `guidance`, `how-to`, `practice`, `guide`

**Allowed:**
- Suggested workflows
- Examples and operator steps
- Practical advice

**Forbidden:**
- New requirements (no "MUST", "NEVER", "REQUIRED" for binding rules)
- Machine-interface definitions

**Required disclaimer:**
Guides MUST include a disclaimer: if a guide conflicts with Constitution/Interfaces, the guide is wrong.

---

## 4. Links Footer (Graph Contract)

The canonical markdown dependency graph is defined exclusively by `## Links` footers.

### 4.1 Links Section Requirements

| Requirement | Description |
|-------------|-------------|
| **Required** | Every canonical doc MUST have a `## Links` footer |
| **Format** | Repo-relative paths in backticks (e.g., `` `core/DECAPOD.md` ``) |
| **Reachability** | `core/DECAPOD.md` MUST reach every canonical doc via `## Links` graph (claim: `claim.doc.decapod_reaches_all_canonical`) |

### 4.2 Hop Constraints

**Constitution hop constraint (intended invariant):**
- Every Constitution doc with `**Binding:** Yes` SHOULD be linked directly from `core/DECAPOD.md`
- No buried law (direct reachability)

**Interfaces hop constraint (intended invariant):**
- Every Interfaces doc with `**Binding:** Yes` SHOULD be reachable from `core/DECAPOD.md` within 2 hops
- Direct or via a single router doc

### 4.3 Links Section Format

```markdown
## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine

### Registry (Core Indices)
- `core/PLUGINS.md` - Subsystem registry
- `core/INTERFACES.md` - Interface contracts index

### Contracts (Interfaces Layer - This Document)
- `interfaces/DOC_RULES.md` - Doc compilation rules
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/GLOSSARY.md` - Term definitions

### Practice (Methodology Layer)
- `methodology/SOUL.md` - Agent identity
- `methodology/ARCHITECTURE.md` - Architecture practice
```

### 4.4 Derived Documents

`docs/DOC_MAP.md` is derived from this graph and MUST NOT be edited by hand.

---

## 5. Subsystem Truth (Single Source)

### 5.1 Single Source Rule

The only canonical place allowed to list subsystems and their statuses is:
- `core/PLUGINS.md` (Subsystem Registry)

Any other doc that needs to refer to subsystems MUST point to the registry instead of restating it.

### 5.2 Reference Format

**Correct:**
```markdown
Subsystem status is defined in `core/PLUGINS.md`.
```

**Incorrect:**
```markdown
Subsystems:
- todo (REAL)
- docs (REAL)
- validate (REAL)
```

---

## 6. Truth Labels (For Interfaces)

Any interface statement that looks like an API (commands, schemas, guarantees) MUST be tagged with one of:

| Label | Meaning | Requirement |
|-------|---------|-------------|
| `REAL` | Implemented and working now | Must have named proof surface |
| `STUB` | Surface exists, behavior incomplete | Document what's missing |
| `SPEC` | Intended interface; not implemented | Design doc must exist |
| `IDEA` | Exploratory; not a commitment | No design required |
| `DEPRECATED` | Do not use | Must have replacement |

### 6.1 REAL Label Requirements

`REAL` requires a named proof surface.
- If no proof surface exists, the statement MUST be labeled `STUB` or `SPEC` instead.
- This is claim: `claim.doc.real_requires_proof`

**Example:**
```markdown
| todo | `decapod todo` | implemented | REAL | `plugins/TODO.md` | `decapod data schema --subsystem todo` |
```

### 6.2 Where Truth Labels Are Required

Truth labels are required in:
- Subsystem registry rows
- Command lists (if present)
- Schema descriptions (if present)
- Feature status tables

---

## 7. No Duplicate Authority

### 7.1 The Rule

No requirement may be defined in multiple places (claim: `claim.doc.no_duplicate_authority`).

### 7.2 Conflict Resolution

If two docs define the same requirement:
1. **Constitution wins** over Interfaces
2. **Interfaces wins** over Guides
3. Guides must delete or soften conflicting statements (guidance only)

### 7.3 Meta-Rule: Contradiction is Invalid

If two canonical binding docs appear to disagree, the system is in an invalid state.
- Resolution is NOT interpretation
- Resolution is AMENDMENT (see `specs/AMENDMENTS.md`)

---

## 8. Claims Ledger (Promises Must Be Registered)

### 8.1 Claim Registration Requirements

Any guarantee/invariant in a canonical doc MUST:
1. Include a claim-id (e.g., `(claim: claim.store.blank_slate)`) near the guarantee
2. Be registered in `interfaces/CLAIMS.md`
3. Declare its proof surface if labeled `REAL`

### 8.2 Claim ID Format

Format: `claim.<domain>.<name>`

Examples:
- `claim.store.blank_slate`
- `claim.doc.decapod_reaches_all_canonical`
- `claim.agent.invocation_checkpoints_required`

### 8.3 Example Claim Placement

```markdown
Store selection must be explicit; implicit store selection is undefined.
(claim: claim.store.explicit_store_selection)
```

---

## 9. Decision Rights Matrix (Authority Routing)

This matrix defines which canonical doc owns which type of decision. If you need to change a decision, amend the owner doc (see `specs/AMENDMENTS.md`).

| Decision Type | Owner Doc (Single Source) |
|---------------|---------------------------|
| Authority hierarchy, proof doctrine, contradiction handling | `specs/SYSTEM.md` |
| Change control for binding docs | `specs/AMENDMENTS.md` |
| Methodology contract (how agents should work) | `specs/INTENT.md` |
| Agent persona/interaction constraints | `methodology/SOUL.md` |
| Doc compilation rules, graph semantics, truth labels, claims registration | `interfaces/DOC_RULES.md` |
| Claims registry (what we promise + proof surfaces) | `interfaces/CLAIMS.md` |
| Store semantics and purity model | `interfaces/STORE_MODEL.md` |
| Subsystem existence/status/truth labels registry | `core/PLUGINS.md` |
| Control-plane sequencing patterns | `interfaces/CONTROL_PLANE.md` |
| Deprecation and migration contract | `core/DEPRECATION.md` |
| Loaded-term definitions | `interfaces/GLOSSARY.md` |
| Testing contracts | `interfaces/TESTING.md` |

---

## 10. Compliance Verification

### 10.1 Machine Checks

| Check | What It Validates | Command |
|-------|------------------|---------|
| Doc graph reachability | Every doc reachable from DECAPOD | `decapod validate` |
| Header format | Required fields present | `decapod validate` |
| Truth labels | Labels match proof surfaces | `decapod validate` |
| No contradictions | Binding docs don't conflict | `decapod validate` (planned) |

### 10.2 Human Review Triggers

These require human judgment:
- Whether a claim is appropriately scoped
- Whether a doc correctly classifies as binding/non-binding
- Whether authority routing is correct

### 10.3 Common Violations

| Violation | Fix |
|-----------|-----|
| Missing `## Links` section | Add complete links section |
| Missing header fields | Add required fields |
| Wrong truth label | Update to correct label |
| Subsystem list not in PLUGINS.md | Add to PLUGINS.md, reference from there |
| Duplicate requirement | Remove duplicate, keep authoritative source |

---

## Links

### Core Router
- [core/DECAPOD.md](core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards**

### Authority (Constitution Layer)
- [specs/INTENT.md](specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](specs/SYSTEM.md) - System definition and authority doctrine
- [specs/SECURITY.md](specs/SECURITY.md) - Security contract
- [specs/GIT.md](specs/GIT.md) - Git etiquette contract
- [specs/AMENDMENTS.md](specs/AMENDMENTS.md) - Change control

### Registry (Core Indices)
- [core/PLUGINS.md](core/PLUGINS.md) - Subsystem registry
- [core/INTERFACES.md](core/INTERFACES.md) - Interface contracts index
- [core/METHODOLOGY.md](core/METHODOLOGY.md) - Methodology guides index
- [core/DEPRECATION.md](core/DEPRECATION.md) - Deprecation contract

### Contracts (Interfaces Layer - This Document)
- [interfaces/CLAIMS.md](interfaces/CLAIMS.md) - **Promises ledger**
- [interfaces/STORE_MODEL.md](interfaces/STORE_MODEL.md) - Store semantics
- [interfaces/GLOSSARY.md](interfaces/GLOSSARY.md) - Term definitions
- [interfaces/CONTROL_PLANE.md](interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/TESTING.md](interfaces/TESTING.md) - Testing contract

### Practice (Methodology Layer)
- [methodology/SOUL.md](methodology/SOUL.md) - Agent identity
- [methodology/ARCHITECTURE.md](methodology/ARCHITECTURE.md) - Architecture practice
- [methodology/TESTING.md](methodology/TESTING.md) - Testing practice
- [methodology/KNOWLEDGE.md](methodology/KNOWLEDGE.md) - Knowledge curation
- [methodology/MEMORY.md](methodology/MEMORY.md) - Memory and learning