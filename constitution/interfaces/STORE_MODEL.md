# STORE_MODEL.md - Store Purity and Threat Model

**Authority:** interface (store semantics + safety model)
**Layer:** Interfaces
**Binding:** Yes

---

## Table of Contents

1. [Purpose and Scope](#1-purpose-and-scope)
2. [Stores Defined](#2-stores-defined)
3. [Assets (What We Protect)](#3-assets-what-we-protect)
4. [Threats (How Systems Die)](#4-threats-how-systems-die)
5. [Guarantees (Contract)](#5-guarantees-contract)
6. [Red Lines (Unacceptable Behavior)](#6-red-lines-unacceptable-behavior)
7. [Store Selection Semantics](#7-store-selection-semantics)
8. [Contamination Scenarios](#8-contamination-scenarios)
9. [Recovery Procedures](#9-recovery-procedures)

---

This document defines store selection semantics and the safety model for preventing cross-store contamination.

---

## 1. Purpose and Scope

The Store Model exists to:
1. Define what stores are and how they differ
2. Establish guarantees about store isolation
3. Prevent cross-store contamination (repo → user)
4. Define acceptable store access patterns

**This is a safety model.** It defines what MUST NOT happen, not just what SHOULD happen.

---

## 2. Stores Defined

### 2.1 User Store

**Path:** `~/.decapod` (home directory)

**Purpose:** Personal agent state, private to the user

**Characteristics:**
- Private to the user
- Never shared between projects
- Blank slate on first use
- User has full control

### 2.2 Repo Store

**Path:** `<repo>/.decapod/project`

**Purpose:** Project-specific state, shared between agents working on the project

**Characteristics:**
- Shared state (with appropriate access controls)
- Project-specific configuration
- Can be committed to version control (parts)
- Dogfooding surface for Decapod itself

### 2.3 Store Comparison

| Aspect | User Store | Repo Store |
|--------|------------|------------|
| **Path** | `~/.decapod` | `<repo>/.decapod/project` |
| **Scope** | Per-user, per-machine | Per-repo |
| **Sharing** | Not shared | Shared between project members |
| **Privacy** | Private | May be visible to team |
| **Blank slate** | Default (empty) | Configured by project |
| **Typical contents** | Personal TODOs, preferences | Project TODOs, configs |

---

## 3. Assets (What We Protect)

### 3.1 User Store Privacy

**Asset:** A user starts blank and should not inherit repo ideology or backlog

**Why it matters:**
- User privacy
- Prevent project contamination of personal space
- Maintain clean slate semantics

**Threat:** Repo dogfood tasks appearing in user store

### 3.2 Repo Store Reproducibility

**Asset:** Repo state should be deterministically rebuildable from repo-tracked artifacts where declared

**Why it matters:**
- Reproducibility
- Auditability
- Team collaboration

### 3.3 Derived State Integrity

**Asset:** Derived artifacts should never be treated as source-of-truth

**Why it matters:**
- Prevent mutation of derived state
- Maintain clear provenance
- Enable reliable rebuild

### 3.4 Provenance

**Asset:** Every mutation should be attributable to an actor and a store context

**Why it matters:**
- Audit trail
- Accountability
- Debugging

---

## 4. Threats (How Systems Die)

### 4.1 Accidental Contamination

**Threat:** Repo dogfood tasks appearing in user store

**How it happens:**
- Implicit store selection defaults to wrong store
- Agent accidentally writes to user store when intending repo
- No validation of store selection

**Impact:**
- User sees project-specific items
- Personal productivity reduced
- Trust in store separation eroded

### 4.2 Ghost State

**Threat:** Agent writes to a store without intending to (wrong root, implicit defaults)

**How it happens:**
- Default store is user, but agent thought it was repo
- `--root` flag used incorrectly
- Missing explicit store specification

**Impact:**
- State appears in wrong location
- Hard to find/remove
- Can cause confusion for other agents

### 4.3 Split Brain

**Threat:** Multiple "canonical" stores or parallel tooling

**How it happens:**
- Agents using different stores for same purpose
- Local overrides not synchronized
- Ad-hoc tooling bypassing Decapod

**Impact:**
- Inconsistent state
- Conflicting changes
- Loss of audit trail

### 4.4 Provenance Loss

**Threat:** Mutations without a record of who/when/why

**How it happens:**
- Direct file manipulation
- Bypass of Decapod surfaces
- Missing audit logging

**Impact:**
- Cannot trace changes
- Cannot debug issues
- Cannot verify compliance

---

## 5. Guarantees (Contract)

All guarantees here are registered in `interfaces/CLAIMS.md`.

### 5.1 Blank Slate (claim: `claim.store.blank_slate`)

**Guarantee:** A fresh user store contains no TODOs unless the user adds them

**Proof:** `decapod validate --store user`

**What this means:**
- User store starts empty
- No pre-populated items from Decapod
- No sample/demo content

### 5.2 No Auto-Seeding (claim: `claim.store.no_auto_seeding`)

**Guarantee:** Repo store content must never appear in the user store automatically

**Proof:** `decapod validate --store user`

**What this means:**
- No automatic copying of repo TODOs to user
- No sync of project state to personal
- Clear boundary between stores

### 5.3 Explicit Store Selection (claim: `claim.store.explicit_store_selection`)

**Guarantee:** Mutating commands must be treated as undefined unless store context is explicit; `--store` is preferred and `--root` is dangerous

**Proof:** `decapod validate` (store invariants)

**What this means:**
- Commands require explicit store specification
- Implicit default is user store
- `--root` is escape hatch with danger warning

### 5.4 CLI-Only Access (claim: `claim.store.decapod_cli_only`)

**Guarantee:** Agents must not read/write `<repo>/.decapod/*` files directly; access must go through `decapod` CLI surfaces

**Proof:** `decapod validate` (Four Invariants Gate marker checks)

**What this means:**
- No direct file manipulation
- All access via Decapod commands
- Prevents jailbreak-style state tampering

---

## 6. Red Lines (Unacceptable Behavior)

These behaviors are explicitly forbidden:

### 6.1 Writing Repo Backlog into User Store

**What:** Automatically creating TODOs in user store based on repo content

**Why forbidden:** Violates blank slate guarantee

**Example of what NOT to do:**
```bash
# WRONG
decapod todo import --from repo --to user

# This would seed user store with repo content
```

### 6.2 Silently Switching Stores Mid-Session

**What:** Changing store context without explicit command or warning

**Why forbidden:** Causes ghost state

### 6.3 Creating Alternate State Roots Outside `.decapod`

**What:** Creating state in non-standard locations

**Why forbidden:** Breaks audit trail, enables split brain

**Example of what NOT to do:**
```bash
# WRONG
decapod todo --root /tmp/my-todos list
```

### 6.4 Direct Read/Write of `<repo>/.decapod/*` Files

**What:** Manipulating Decapod state files directly

**Why forbidden:** Violates CLI-only access, breaks provenance

**Example of what NOT to do:**
```bash
# WRONG
vim .decapod/project/todos.json
```

### 6.5 Claiming Compliance Without Running Proof

**What:** Saying store is clean without running validation

**Why forbidden:** Proof is the currency of trust

---

## 7. Store Selection Semantics

### 7.1 Default Store

**Default:** User store (`~/.decapod`)

This means:
- `decapod todo list` operates on user store by default
- Agents must explicitly opt into repo store

### 7.2 Explicit Selection

```bash
# Explicit user store (redundant but clear)
decapod todo list --store user

# Explicit repo store
decapod todo list --store repo
```

### 7.3 Root Override (Dangerous)

```bash
# Escape hatch for special cases
decapod todo list --root /custom/path

# WARNING: Bypasses normal store semantics
# Use only when absolutely necessary
```

---

## 8. Contamination Scenarios

### 8.1 Scenario: Accidental Repo → User Seeding

**Situation:** User sees project TODOs in their personal view

**Root cause:** Auto-seeding bug or misconfigured command

**Detection:**
```bash
decapod validate --store user
# Should report: 0 items (fresh store)
```

**Fix:**
1. Identify the contamination source
2. Clear user store of repo items
3. Fix the bug that caused seeding
4. Verify with validation

### 8.2 Scenario: Wrong Store Selection

**Situation:** Agent creates TODO expecting it to be private, but it's in repo store

**Root cause:** Missing `--store user` flag

**Detection:**
```bash
# Check repo store for personal items
decapod todo list --store repo | grep personal

# Check user store is clean
decapod todo list --store user | wc -l
```

**Fix:**
1. Move TODO to correct store
2. Document store selection requirement
3. Add validation for sensitive operations

### 8.3 Scenario: Split State

**Situation:** Two different tools showing different TODOs

**Root cause:** Different stores in use

**Detection:**
```bash
decapod todo list --store user | head -5
decapod todo list --store repo | head -5
# Compare outputs
```

**Fix:**
1. Determine which store is authoritative
2. Migrate if necessary
3. Standardize on one store

---

## 9. Recovery Procedures

### 9.1 Contamination Recovery

If user store is contaminated:

```bash
# 1. Verify contamination
decapod validate --store user
# Should show contamination

# 2. Export any legitimate user items
decapod todo list --store user > user-items-backup.json

# 3. Reset user store (if supported)
decapod store reset --store user

# 4. Restore legitimate items
# (manually, to avoid re-contamination)

# 5. Verify clean
decapod validate --store user
```

### 9.2 Provenance Recovery

If provenance is broken:

```bash
# 1. Check audit log
decapod audit log --store user | head -20

# 2. Identify gap
# 3. Restore from backup if available
# 4. Add missing provenance for future changes
```

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - **Oracle for Engineering Standards**

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security contract
- `specs/AMENDMENTS.md` - Change control

### Registry (Core Indices)
- `core/PLUGINS.md` - Subsystem registry
- `core/INTERFACES.md` - Interface contracts index
- `core/METHODOLOGY.md` - Methodology guides index

### Contracts (Interfaces Layer - This Document)
- `interfaces/CONTROL_PLANE.md` - Sequencing patterns
- `interfaces/DOC_RULES.md` - Doc compilation rules
- `interfaces/CLAIMS.md` - **Promises ledger**
- `interfaces/GLOSSARY.md` - Term definitions
- `interfaces/TESTING.md` - Testing contract

### Practice (Methodology Layer)
- `methodology/SOUL.md` - Agent identity
- `methodology/ARCHITECTURE.md` - Architecture practice
- `methodology/TESTING.md` - Testing practice
- `methodology/KNOWLEDGE.md` - Knowledge curation
- `methodology/MEMORY.md` - Memory and learning

### Operations (Plugins Layer)
- `plugins/TODO.md` - Work tracking
- `plugins/VERIFY.md` - Validation subsystem
- `plugins/EMERGENCY_PROTOCOL.md` - Emergency protocols