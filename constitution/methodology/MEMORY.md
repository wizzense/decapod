# MEMORY.md - Agent Memory Practice

**Authority:** guidance (memory hygiene and usage)
**Layer:** Guides
**Binding:** No
**Scope:** how to create, retrieve, and prune memory effectively
**Non-goals:** schema enforcement and machine interface contracts

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [Memory Types](#2-memory-types)
3. [Creation Discipline](#3-creation-discipline)
4. [Retrieval Discipline](#4-retrieval-discipline)
5. [Pruning and Maintenance](#5-pruning-and-maintenance)
6. [Confidence and Uncertainty](#6-confidence-and-uncertainty)
7. [Memory vs. Knowledge Distinction](#7-memory-vs-knowledge-distinction)
8. [Integration with Learning Systems](#8-integration-with-learning-systems)

---

## 1. Purpose

Memory exists to reduce repeated effort and improve decision quality across sessions. The goal is not comprehensive logging but actionable residue — pointers and short-term context that improve future performance.

---

## 2. Memory Types

### 2.1 Short-Term Memory (Context)

Immediate working context from current session.

**What it contains:**
- Current task and its state
- Active files and their content
- Recent commands executed
- Immediate goals and next steps

**Characteristics:**
- High fidelity, high relevance
- Lost at session end
- Should not be treated as durable

**Example:**
```markdown
Current task: Expand core/METHODOLOGY.md to 1500+ lines
Progress: Written initial structure, currently writing §3
Next: Complete §4-§6, then expand Links section
Files: constitution/core/METHODOLOGY.md
```

### 2.2 Medium-Term Memory (Workspace)

Session-persistent knowledge within a project.

**What it contains:**
- Project structure and conventions
- Current work in progress
- TODOs and their state
- Recent decisions and their rationale

**Characteristics:**
- Persists across sessions within project
- Should be distilled to permanent storage
- Can be reconstructed from artifacts

**Example:**
```markdown
Project: Decapod constitution expansion
Active work: Expanding methodology and interface docs
Convention: Each doc needs complete ## Links section
Current priority: METHODOLOGY.md, PLUGINS.md, GAPS.md
```

### 2.3 Long-Term Memory (Durable)

System-wide knowledge that persists indefinitely.

**What it contains:**
- Architectural decisions and their rationale
- Patterns that recur across projects
- Known failure modes and their symptoms
- Learned shortcuts and optimizations

**Characteristics:**
- Highly distilled and validated
- Should be verifiable
- Transferable across projects

**Example:**
```markdown
Pattern: When adding claims to CLAIMS.md, always include proof surface
Failure mode: Claims without proof become technical debt
Shortcut: decapod validate catches most doc structure issues
```

---

## 3. Creation Discipline

### 3.1 When to Create Memory

Create memory entries when:
- Completing significant work that might be relevant later
- Discovering a non-obvious solution to a problem
- Encountering a failure mode worth avoiding
- Making a decision that required significant analysis

**Do not create memory for:**
- Trivial, easily re-derived information
- Session-specific context that won't persist
- Information already captured in documentation
- Transient state that changes frequently

### 3.2 Memory Entry Format

Keep memory entries concise:

```markdown
# Memory Entry

**What:** [What happened or what you learned]
**Context:** [When/why this matters]
**Action:** [What to do with this]
**Confidence:** [High/Medium/Low]
**Expires:** [When to revisit or null for permanent]
```

### 3.3 What to Store

**Store pointers and short residue, not essays.**

Good memory:
- "Use `decapod validate` before committing — catches doc structure issues"
- "PLUGINS.md is the canonical subsystem registry — don't restate lists"
- "Claim-before-work pattern prevents duplicate effort"

Bad memory:
- Full copy of a doc that could be retrieved
- Detailed explanation of something that's documented
- Raw transcript of a conversation

### 3.4 Linking Over Copying

Link to TODO, knowledge, or proof artifacts rather than copying content:

```markdown
# Good
See TODO-123 for the implementation details of this pattern.

# Bad  
The implementation does:
1. Check store selection
2. Validate store purity
3. ...
```

---

## 4. Retrieval Discipline

### 4.1 When to Retrieve

Retrieve memory when:
- Starting a new task in a familiar domain
- Encountering a familiar error or failure
- Making a decision similar to past decisions
- Planning work in an area you've touched before

### 4.2 Retrieval Strategies

1. **Retrieve only what is relevant to the active task**
   - Don't load entire memory on every task
   - Query for specific context
   - Update memory with new context as task evolves

2. **Treat low-confidence memory as a hypothesis**
   - Memory can be wrong or outdated
   - Verify before acting on old memory
   - Update memory when new information contradicts it

3. **Verify before promoting conclusions**
   - Cross-check with documentation
   - Test assumptions before committing
   - Update memory when reality differs

### 4.3 Retrieval Example

```bash
# Retrieve relevant memory for doc expansion task
decapod data context retrieve --query "methodology doc expansion"

# Result shows:
# - Prior work on METHODOLOGY.md
# - Conventions learned during expansion
# - Related TODO items

# Verify memory against current state
decapod validate
# Memory still valid, proceed with task
```

---

## 5. Pruning and Maintenance

### 5.1 When to Prune

Prune memory entries when:
- They contain information that's now in documentation
- They are superseded by newer entries
- They were time-sensitive and the time has passed
- They have low value and high maintenance cost
- Confidence was low and was never validated

### 5.2 Pruning Priorities

**High priority to prune:**
- Outdated technical information
- Duplicates of documentation
- Transient context that changed
- Low-confidence entries never validated

**Low priority to prune:**
- Validated architectural decisions
- Verified failure mode patterns
- Proven shortcuts and conventions

### 5.3 Regular Maintenance

Perform memory hygiene:
- Review memory before starting major tasks
- Consolidate similar entries
- Archive entries no longer relevant
- Verify time-sensitive entries

---

## 6. Confidence and Uncertainty

### 6.1 Confidence Levels

| Level | Meaning | Behavior |
|-------|---------|----------|
| **High** | Verified, well-understood | Act on confidently |
| **Medium** | Likely correct, may be incomplete | Act on with verification |
| **Low** | Uncertain, may be wrong | Verify before acting |

### 6.2 Expressing Uncertainty

When memory is uncertain, be explicit:

```markdown
# Memory with explicit uncertainty

**What:** Connection pool exhaustion might cause checkout timeouts
**Confidence:** Low
**Note:** This is hypothesis from reading logs; not verified
**Action:** Investigate during next incident before assuming
```

### 6.3 Updating Confidence

When uncertainty is resolved:
1. Update memory with correct information
2. Mark confidence level
3. Add provenance of how confidence was verified

---

## 7. Memory vs. Knowledge Distinction

### 7.1 Memory is Personal and Ephemeral

Memory reflects personal experience and context. It can be wrong, outdated, or incomplete.

### 7.2 Knowledge is Shared and Validated

Knowledge is curated for shared use and should be verifiable and maintained.

### 7.3 The Relationship

```
Memory → [distillation/validation] → Knowledge
```

When memory reveals something valuable:
1. Assess if it should be shared (knowledge candidate)
2. If yes, create knowledge entry with provenance
3. Keep memory reference to knowledge

---

## 8. Integration with Learning Systems

### 8.1 Memory and TODO

Memory often reveals work to be done:
- Update TODO with context from memory
- Link memory to TODO for traceability
- Close loop when work is complete

### 8.2 Memory and Knowledge

Memory is the raw material for knowledge:
- Episodic observations → knowledge base
- Verification of memory → knowledge provenance
- Memory patterns → semantic knowledge

### 8.3 Memory and Federation

Federated memory allows sharing memory across agents:
```bash
decapod data federation ingest --source memory --domain context
```

---

## Links

### Core Router
- [core/DECAPOD.md](../../core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](../../core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards**

### Authority (Constitution Layer)
- [specs/INTENT.md](../specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](../specs/SYSTEM.md) - System definition and authority doctrine

### Registry (Core Indices)
- [core/PLUGINS.md](../../core/PLUGINS.md) - Subsystem registry
- [core/METHODOLOGY.md](../../core/METHODOLOGY.md) - Methodology guides index
- [core/INTERFACES.md](../../core/INTERFACES.md) - Interface contracts index

### Contracts (Interfaces Layer)
- [interfaces/MEMORY_SCHEMA.md](../../interfaces/MEMORY_SCHEMA.md) - **Binding memory schema**
- [interfaces/MEMORY_INDEX.md](../../interfaces/MEMORY_INDEX.md) - Memory index
- [interfaces/CONTROL_PLANE.md](../../interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/KNOWLEDGE_STORE.md](../../interfaces/KNOWLEDGE_STORE.md) - Knowledge store semantics

### Practice (Methodology Layer - This Document)
- [methodology/SOUL.md](./SOUL.md) - Agent identity
- [methodology/ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture practice
- [methodology/KNOWLEDGE.md](./KNOWLEDGE.md) - **Knowledge curation**
- [methodology/TESTING.md](./TESTING.md) - Testing practice
- [methodology/CI_CD.md](./CI_CD.md) - CI/CD practice

### Operations (Plugins Layer)
- [plugins/TODO.md](../plugins/TODO.md) - Work tracking
- [plugins/FEDERATION.md](../plugins/FEDERATION.md) - **Federation (governed agent memory)**
- [plugins/APTITUDE.md](../plugins/APTITUDE.md) - Skill management

---

## Project Override Context

**Project memory emphasis:**
- Use layered memory (short-term context + durable workspace knowledge)
- Prefer retrieval strategies that combine lexical and semantic signals
- Trigger compaction/summarization before context pressure causes silent loss
- Keep memory interfaces tool-agnostic so storage backends can evolve
- Memory should be a tool for better performance, not a second specification