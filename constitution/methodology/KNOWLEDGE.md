# KNOWLEDGE.md - Knowledge Management Practice

**Authority:** guidance (how to curate and use knowledge)
**Layer:** Guides
**Binding:** No
**Scope:** capture discipline, curation workflow, and lifecycle hygiene
**Non-goals:** schema contracts and CLI interface definitions

---

## Table of Contents

1. [Purpose](#1-purpose)
2. [Knowledge Types](#2-knowledge-types)
3. [Capture Discipline](#3-capture-discipline)
4. [Curation Rules](#4-curation-rules)
5. [Lifecycle Management](#5-lifecycle-management)
6. [Provenance and Citation](#6-provenance-and-citation)
7. [Knowledge vs. Contract Boundaries](#7-knowledge-vs-contract-boundaries)
8. [Search and Retrieval](#8-search-and-retrieval)
9. [Knowledge Quality](#9-knowledge-quality)
10. [Integration with Other Systems](#10-integration-with-other-systems)

---

## 1. Purpose

Use knowledge entries to preserve context that improves future execution:
- Rationale behind decisions (why we chose X over Y)
- Reusable investigations (how we debugged issue Z)
- Runbooks and operational guidance
- Patterns that generalize across similar problems
- Failure modes and how to recognize them

**Knowledge is context, not contract.** This distinction is critical.

---

## 2. Knowledge Types

### 2.1 Episodic Knowledge

Individual experiences and observations.

**Examples:**
- "Debugged production outage on 2026-05-10; root cause was connection pool exhaustion"
- "Investigation of slow query: missing index on user_id column"
- "User reported issue with checkout flow; traced to stale cache"

**Characteristics:**
- Timestamp-based
- Context-specific
- Not directly actionable without interpretation

### 2.2 Semantic Knowledge

Generalized patterns extracted from episodic knowledge.

**Examples:**
- "Connection pool exhaustion typically happens when: 1) pool too small, 2) queries block, 3) connections leak"
- "Stale cache issues follow a pattern: symptoms appear intermittently, cache invalidation fixes"
- "Checkout flow failures often trace to: payment provider timeout, cart serialization bug, session expiration"

**Characteristics:**
- Pattern-based
- Context-independent
- Directly actionable
- Extracted from multiple episodic entries

### 2.3 Procedural Knowledge

Step-by-step instructions for specific tasks.

**Examples:**
- "How to diagnose high latency: 1) check metrics dashboard, 2) look for slow queries, 3) check resource utilization"
- "How to rotate credentials: 1) generate new key, 2) update secret manager, 3) restart services, 4) verify"
- "How to run database migrations: 1) backup DB, 2) run migration, 3) verify schema, 4) test application"

**Characteristics:**
- Action-oriented
- Ordered steps
- Repeatable

### 2.4 Structural Knowledge

Knowledge about relationships between concepts.

**Examples:**
- "Component X depends on Y for configuration, Z for data"
- "The order service calls payment service, which calls external provider"
- "User authentication flows through: load balancer → auth service → session store"

**Characteristics:**
- Graph-like
- Shows dependencies
- Useful for impact analysis

---

## 3. Capture Discipline

### 3.1 When to Capture

Capture knowledge when:
- Completing a non-trivial investigation
- Making a decision with non-obvious rationale
- Discovering a pattern that could recur
- Writing runbook for operational task
- Solving a problem that took significant time

**Do not capture:**
- Trivial facts obvious from documentation
- Transient state (put in memory, not knowledge base)
- Opinions without evidence
- Duplicate of existing knowledge

### 3.2 What to Capture

For each knowledge entry, capture:

| Field | Description | Required |
|-------|-------------|----------|
| **Title** | Concise description of what this captures | Yes |
| **Type** | Episodic, semantic, procedural, or structural | Yes |
| **Summary** | 2-3 sentences of the key insight | Yes |
| **Context** | Background, constraints, what led to this | Yes |
| **Evidence** | How we know this is true | Yes |
| **Tags** | For discoverability | Yes |
| **Provenance** | Source of knowledge (commit, PR, doc, transcript) | Yes |
| **Action** | What should someone do with this? | No |
| **Related** | Links to related knowledge entries | No |

### 3.3 Capture Format

```markdown
# Knowledge Entry

**Title:** Connection pool exhaustion pattern in production

**Type:** Semantic

**Summary:** Connection pool exhaustion manifests as timeout errors
during peak traffic and can be caused by slow queries, connection leaks,
or insufficient pool size.

**Context:**
During the 2026-05-10 production incident, we observed connection
timeouts that prevented users from checkout. The service had 100 max
connections but queries were blocking waiting for connections.

**Evidence:**
- APM showing connection wait time spiking to 5s+
- Database showing all connections in use
- Code review showing missing connection close in error path

**Tags:**
- performance
- database
- connection-pool
- production-incident

**Provenance:**
- Incident: INC-2026-0510
- PR: #1234 (connection cleanup fix)

**Actions:**
- Monitor connection pool utilization in dashboards
- Set alerts for connection wait time > 1s
- Review error paths for connection leaks

**Related:**
- KNOWLEDGE-456 (similar pattern in auth service)
- KNOWLEDGE-789 (pool sizing guidelines)
```

---

## 4. Curation Rules

### 4.1 Curation Principles

1. **Prefer concise summaries with links to evidence**
   - Don't reproduce entire investigations
   - Link to commits, PRs, docs that have the details
   - Summary should be 3-5 sentences max

2. **Tag entries for discoverability**
   - Use consistent tags
   - Include domain tags (e.g., `database`, `auth`, `frontend`)
   - Include type tags (e.g., `pattern`, `runbook`, `decision`)

3. **Mark stale or superseded entries quickly**
   - Set expiration when knowledge is time-sensitive
   - Mark superseded when practices change
   - Don't let stale knowledge mislead

4. **Link actionable items to TODO IDs**
   - If knowledge reveals work to be done, create TODO
   - Link TODO in knowledge entry
   - Close TODO when work is complete

### 4.2 Quality Guidelines

**Good knowledge entry:**
- Title is specific and descriptive
- Summary captures the key insight
- Context explains why this matters
- Evidence is verifiable
- Tags enable discovery

**Bad knowledge entry:**
- Title is vague ("Issue with database")
- Summary requires reading entire entry to understand
- No context for when to use this
- Unverifiable claims
- Tags are inconsistent or missing

### 4.3 Conflict Resolution

When knowledge entries conflict:

1. **Evidence wins**: Entry with verifiable evidence takes precedence
2. **Recency matters**: Newer evidence overrides older
3. **Source matters**: Direct observation > inference > hearsay
4. **Document disagreement**: Don't delete conflicting entry, add context

---

## 5. Lifecycle Management

### 5.1 Lifecycle States

```
Draft → Published → Verified → Maintained → Superseded → Archived
  │          │           │            │            │           │
  └──────────┴───────────┴────────────┴────────────┴───────────┘
                          (can move backward if issues found)
```

| State | Description |
|-------|-------------|
| **Draft** | Initial capture, needs review |
| **Published** | Available for retrieval |
| **Verified** | Cross-checked and confirmed |
| **Maintained** | Actively kept current |
| **Superseded** | Replaced by newer knowledge |
| **Archived** | Retained for historical reference |

### 5.2 Lifecycle Operations

**Create:** Record new learnings from non-trivial work
**Curate:** Tighten wording and link related artifacts
**Verify:** Cross-check claims before promoting
**Consolidate:** Merge duplicates and promote durable patterns
**Retire:** Mark stale/superseded entries

### 5.3 Maintenance Policy

| Knowledge Type | Review Frequency | Action When Stale |
|---------------|-----------------|-------------------|
| Episodic | 6 months | Archive or consolidate |
| Semantic | 12 months | Verify pattern still holds |
| Procedural | 3 months | Verify steps still work |
| Structural | 12 months | Verify relationships still valid |

---

## 6. Provenance and Citation

### 6.1 Why Provenance Matters

Knowledge without provenance is opinion. Knowledge with provenance is evidence-based.

**Every procedural memory entry must cite evidence:**
- Commit hash linking to the relevant code
- PR number where decision was made
- Document where policy is defined
- Incident ID for operational learnings
- Transcript for conversation-based knowledge

### 6.2 Provenance Types

| Type | Example | When to Use |
|------|---------|-------------|
| **Commit** | `abc123def` | Code-related knowledge |
| **PR** | `#1234` | Decision records |
| **Doc** | `architecture/DATA.md` | Documented policies |
| **Incident** | `INC-2026-0510` | Operational learnings |
| **External** | `vendor-docs-link` | Third-party knowledge |
| **Transcript** | `session-2026-05-10` | Conversation-based |

### 6.3 Citation Format

```markdown
**Provenance:**
- Decision: PR #1234 (approve_connection_pool_size)
- Evidence: commit abc123def (connection cleanup fix)
- Incident: INC-2026-0510
- External: https://docs.postgresql.org/current/pooling.html
```

---

## 7. Knowledge vs. Contract Boundaries

### 7.1 What Stays in Knowledge

- Context and rationale
- Patterns and observations
- Operational guidance
- Investigation learnings
- "How we do things" that's not formal policy

### 7.2 What Becomes Contract

- Requirements and guarantees
- Interface definitions
- Invariants that must hold
- Process definitions

### 7.3 The Transfer Process

When knowledge should become contract:

1. **Identify the gap**: Knowledge reveals a missing requirement
2. **Draft specification**: Write the formal requirement
3. **Register claim**: Add to `interfaces/CLAIMS.md`
4. **Define proof**: Ensure there is a proof surface
5. **Promote**: Move from knowledge to spec/interfaces

**Example:**
```
Knowledge: "Connection pool exhaustion causes checkout failures"
          ↓
Gap: No requirement for connection monitoring
          ↓
Contract: Add claim to CLAIMS.md about monitoring
          ↓
Proof: Add monitoring check to validate
```

---

## 8. Search and Retrieval

### 8.1 Search Strategies

**By tag:**
```bash
decapod data knowledge search --tag performance
```

**By type:**
```bash
decapod data knowledge search --type semantic
```

**By date range:**
```bash
decapod data knowledge search --since 2026-01-01 --until 2026-05-01
```

**By full-text:**
```bash
decapod data knowledge search --query "connection pool"
```

### 8.2 Retrieval Best Practices

1. **Start broad, narrow down**: Search by domain first, then refine
2. **Use tags, not just text**: Tags provide structured discovery
3. **Check related entries**: Linked knowledge often has what you need
4. **Verify recency**: Check timestamp, verify accuracy

---

## 9. Knowledge Quality

### 9.1 Quality Checklist

Before publishing knowledge:

- [ ] Title is specific and descriptive
- [ ] Summary captures key insight in 3-5 sentences
- [ ] Context explains when this matters
- [ ] Evidence is verifiable
- [ ] Tags are consistent and complete
- [ ] Provenance links to source
- [ ] Action is clear (if applicable)
- [ ] No duplicates of existing entries

### 9.2 Knowledge Debt

Knowledge debt accumulates when:
- Entries are not updated when practices change
- Duplicate entries confuse retrieval
- Provenance is missing or broken
- Tags are inconsistent
- Action items are not tracked

**Treat knowledge debt like technical debt.** Allocate time to address it.

---

## 10. Integration with Other Systems

### 10.1 Knowledge and Memory

Knowledge captures durable insights; memory captures session-specific context.

| Aspect | Knowledge | Memory |
|--------|----------|--------|
| Scope | System-wide | Session-specific |
| Duration | Persistent | Temporary |
| Creation | Intentional curation | Automatic accumulation |
| Use | Cross-session learning | Current task support |

### 10.2 Knowledge and TODO

When knowledge reveals work to be done:
1. Create TODO with reference to knowledge entry
2. Link TODO in knowledge entry
3. Update knowledge when TODO is resolved
4. Close knowledge loop when work is verified

### 10.3 Knowledge and Validation

Knowledge should inform validation:
- Validation failures generate knowledge entries
- Knowledge entries that reveal gaps should add validation

---

## Links

### Core Router
- [core/DECAPOD.md](../../core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](../../core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards**
- [core/GAPS.md](../../core/GAPS.md) - Gap analysis methodology

### Authority (Constitution Layer)
- [specs/INTENT.md](../specs/INTENT.md) - **Methodology contract (READ FIRST)**
- [specs/SYSTEM.md](../specs/SYSTEM.md) - System definition and authority doctrine

### Registry (Core Indices)
- [core/PLUGINS.md](../../core/PLUGINS.md) - Subsystem registry
- [core/METHODOLOGY.md](../../core/METHODOLOGY.md) - Methodology guides index
- [core/INTERFACES.md](../../core/INTERFACES.md) - Interface contracts index

### Contracts (Interfaces Layer)
- [interfaces/KNOWLEDGE_SCHEMA.md](../../interfaces/KNOWLEDGE_SCHEMA.md) - **Binding knowledge schema**
- [interfaces/KNOWLEDGE_STORE.md](../../interfaces/KNOWLEDGE_STORE.md) - Knowledge store semantics
- [interfaces/CONTROL_PLANE.md](../../interfaces/CONTROL_PLANE.md) - Sequencing patterns
- [interfaces/CLAIMS.md](../../interfaces/CLAIMS.md) - Promises ledger
- [interfaces/MEMORY_SCHEMA.md](../../interfaces/MEMORY_SCHEMA.md) - Memory schema

### Practice (Methodology Layer - This Document)
- [methodology/SOUL.md](./SOUL.md) - Agent identity
- [methodology/ARCHITECTURE.md](./ARCHITECTURE.md) - Architecture practice
- [methodology/MEMORY.md](./MEMORY.md) - Memory and learning
- [methodology/TESTING.md](./TESTING.md) - Testing practice

### Operations (Plugins Layer)
- [plugins/TODO.md](../plugins/TODO.md) - Work tracking
- [plugins/KNOWLEDGE.md](../plugins/KNOWLEDGE.md) - **Knowledge subsystem**
- [plugins/FEDERATION.md](../plugins/FEDERATION.md) - Federation subsystem

---

## Project Override Context

**Project knowledge emphasis:**
- Capture patterns that generalize across incidents, not only one-off fixes
- Promote architectural learnings into shared contracts and docs
- Track provenance so claims and decisions can be audited
- Keep knowledge actionable: each entry should inform a concrete next decision
- Verify knowledge before publishing; unverified knowledge is liability