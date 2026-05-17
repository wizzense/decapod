# GAPS.md - Gap Analysis & Systemic Improvement Methodology

**Authority:** guidance (systematic gap identification and routing methodology)
**Layer:** Guides
**Binding:** No
**Scope:** how to identify, categorize, and route gaps in Decapod-managed systems
**Non-goals:** replacing TODO system, substituting for proof, or defining authoritative requirements

---

## Table of Contents

1. [What Is a Gap](#1-what-is-a-gap)
2. [Gap Categories](#2-gap-categories)
3. [Gap Identification Protocol](#3-gap-identification-protocol)
4. [Gap Documentation & Routing](#4-gap-documentation--routing)
5. [Gap Lifecycle](#5-gap-lifecycle)
6. [Gap Analysis Integration with Subsystems](#6-gap-analysis-integration-with-subsystems)
7. [Gap Taxonomy Reference](#7-gap-taxonomy-reference)
8. [Common Gap Patterns](#8-common-gap-patterns)
9. [Gap Analysis for Leadership](#9-gap-analysis-for-leadership)
10. [Emergency Gap Protocol](#10-emergency-gap-protocol)
11. [Gap Analysis Checklist](#11-gap-analysis-checklist)
12. [Gap Resolution Verification](#12-gap-resolution-verification)

---

⚠️ **CRITICAL: Gap analysis is continuous intelligence work, not one-time audits.** ⚠️

This document defines the practice of systemic gap identification: finding what's missing, misaligned, or underdeveloped in the system, and routing those findings to the appropriate subsystems for resolution.

The goal is not to catalog every possible improvement — it's to systematically surface the gaps that matter, route them correctly, and verify their resolution.

---

## 1. What Is a Gap

A **gap** is any delta between:
- **Current state** (what exists)
- **Required state** (what must exist for correctness)
- **Desired state** (what should exist for optimal performance)

**Gaps are not bugs.** Bugs are deviations from spec. Gaps are missing or incomplete specifications, implementations, or capabilities.

**Examples that clarify the distinction:**

| Situation | Classification | Why |
|-----------|---------------|-----|
| Spec says X, code does Y | Gap (spec/implementation drift) | The spec exists but isn't being enforced |
| No spec for a feature | Gap (missing spec) | There's nothing to deviate from |
| Code crashes on input Z | Bug (code defect) | Spec exists, code fails to comply |
| No test for feature W | Gap (missing proof) | The capability exists but can't be verified |
| Agent doesn't know how to handle scenario Q | Gap (methodology vacuum) | No guidance exists for this situation |
| Two docs contradict each other | Gap (contradiction) | System is in invalid state |

### 1.1 Gap Severity Levels

| Severity | Description | Action Threshold |
|----------|-------------|-----------------|
| **Critical** | Blocks work, violates security contracts, causes data loss | Immediate escalation; stop all downstream work |
| **High** | Causes significant friction, workarounds required | High-priority TODO within 24 hours |
| **Medium** | Inconvenience, unclear guidance, non-blocking friction | Medium-priority TODO within 1 week |
| **Low** | Nice to have, optimization, cosmetic issues | Backlog entry or knowledge entry |

---

## 2. Gap Categories

Gaps are categorized by which layer of the system they inhabit. Correct categorization is essential for routing.

### 2.1 Interface Gaps (`interfaces/`)

**Definition:** Missing or incomplete binding contracts, schemas, or invariants.

**What qualifies:**
- CLI surface without corresponding schema documentation
- Store semantics that allow contamination
- Proof surface that doesn't actually validate what it claims
- Undefined behavior at subsystem boundaries
- Schema drift (doc says X, code does Y)
- Claims without proof surfaces
- Missing error types for edge cases

**Examples:**
```
# Example: CLI surface without schema
decapod new-command --flag-x accepts any value
# But no schema documents what --flag-x should accept

# Example: Proof surface gap
claim.doc.real_requires_proof states REAL needs proof
but the proof surface doesn't actually run in CI
```

**Detection Methods:**
- Run `decapod validate` and analyze warnings
- Compare subsystem registry (PLUGINS.md) to actual CLI help output
- Check for `STUB` or `SPEC` items without graduation path
- Review error messages for undocumented edge cases
- Search for claims marked `not_enforced` that should be enforced

**Routing Table for Interface Gaps:**

| Gap Type | Route To |
|----------|----------|
| Interface contract issues | `interfaces/INTERFACES.md` or specific interface doc |
| Store model violations | `interfaces/STORE_MODEL.md` |
| Doc compilation errors | `interfaces/DOC_RULES.md` |
| Claims without proof | `interfaces/CLAIMS.md` |
| Undefined terms | `interfaces/GLOSSARY.md` |
| Testing contract gaps | `interfaces/TESTING.md` |
| Control plane sequencing | `interfaces/CONTROL_PLANE.md` |

**See:** `interfaces/INTERFACES.md` for interface contract registry

### 2.2 Methodology Gaps (`methodology/`)

**Definition:** Missing guidance, unclear practices, or incomplete cognitive frameworks.

**What qualifies:**
- Agent doesn't know how to handle a specific scenario
- Architecture practice lacks decision criteria
- Knowledge management has no staleness policy
- Memory system lacks retrieval validation
- Unclear when to use which subsystem
- UI components lack architectural patterns
- Frontend/backend integration undefined
- No guidance for a recurring task

**Detection Methods:**
- Agents asking repetitive clarifying questions
- Inconsistent approaches to similar problems
- Documentation exists but isn't actionable
- Process gaps in multi-agent coordination
- Missing "how to" guidance for common tasks
- UI implementations diverge without pattern
- Workarounds being invented repeatedly

**Routing Table for Methodology Gaps:**

| Gap Type | Route To |
|----------|----------|
| Intent-driven workflow gaps | `specs/INTENT.md` (binding methodology) |
| Architecture practice gaps | `methodology/ARCHITECTURE.md` |
| Agent behavior gaps | `methodology/SOUL.md` |
| Knowledge management gaps | `methodology/KNOWLEDGE.md` |
| Learning/memory gaps | `methodology/MEMORY.md` |
| Testing practice gaps | `methodology/TESTING.md` |
| CI/CD workflow gaps | `methodology/CI_CD.md` |
| UI architecture gaps | `architecture/UI.md` |
| Frontend architecture gaps | `architecture/FRONTEND.md` |

**See:** `core/METHODOLOGY.md` for methodology registry

### 2.3 Plugin/Subsystem Gaps (`plugins/`)

**Definition:** Missing functionality, incomplete implementations, or subsystem boundary issues.

**What qualifies:**
- TODO system lacks classification features
- Health system doesn't track subsystem X
- Missing cron job scheduling granularity
- No knowledge→TODO linking mechanism
- Gap between planned (SPEC) and implemented (REAL)
- Cross-subsystem coordination failures
- Performance bottlenecks at subsystem boundaries
- Missing CLI surfaces for needed operations

**Detection Methods:**
- Compare PLUGINS.md registry to actual capabilities
- User requests for missing features
- Workarounds agents invent for missing functionality
- Cross-subsystem coordination failures
- Performance bottlenecks at subsystem boundaries
- Check SPEC items for implementation timeline

**Routing Table for Plugin Gaps:**

| Gap Type | Route To |
|----------|----------|
| Subsystem status issues | `core/PLUGINS.md` |
| Plugin-specific gaps | Respective `plugins/<NAME>.md` |
| Integration gaps | Relevant subsystem docs + PLUGINS.md |
| Missing proof surface | Subsystem owner doc + CLAIMS.md |

**See:** `core/PLUGINS.md` §2 for subsystem registry and truth labels

### 2.4 Core/Coordination Gaps (`core/`)

**Definition:** Issues in routing, navigation, or system-wide coordination.

**What qualifies:**
- DECAPOD.md doesn't route to a documented subsystem
- Cross-category references are broken
- OVERRIDE.md isn't being respected
- Gap between demands and enforcement
- Missing emergency protocols
- Navigation failures (can't find docs)
- Contradictions between core files

**Detection Methods:**
- `decapod validate` failures in doc graph
- Broken links in constitution
- Navigation failures (can't find docs)
- Override system not functioning
- Contradictions between core files
- Missing `## Links` sections

**Routing Table for Core Gaps:**

| Gap Type | Route To |
|----------|----------|
| Router/navigation gaps | `core/DECAPOD.md` |
| Interface index gaps | `core/INTERFACES.md` |
| Methodology index gaps | `core/METHODOLOGY.md` |
| Subsystem registry gaps | `core/PLUGINS.md` |
| User demand gaps | `core/DEMANDS.md` |
| Deprecation gaps | `core/DEPRECATION.md` |
| Gap analysis methodology | `core/GAPS.md` (this file) |

### 2.5 Specification Gaps (`specs/`)

**Definition:** Missing system-level contracts, security considerations, or amendment processes.

**What qualifies:**
- Security model doesn't cover new threat vector
- Amendment process unclear for specific change types
- System boundaries undefined for new component
- Git contract doesn't cover specific workflow
- Intent contract missing scenario coverage
- Missing error handling doctrine
- Missing data model for new domain

**Detection Methods:**
- Security reviews finding uncovered areas
- Amendment requests without clear process
- Cross-system integration ambiguities
- Authority disputes about who owns what
- Unclear ownership for new capabilities

**Routing Table for Spec Gaps:**

| Gap Type | Route To |
|----------|----------|
| Intent/methodology contract gaps | `specs/INTENT.md` |
| System definition gaps | `specs/SYSTEM.md` |
| Security gaps | `specs/SECURITY.md` |
| Git workflow gaps | `specs/GIT.md` |
| Change control gaps | `specs/AMENDMENTS.md` |
| Evaluation gaps | `specs/evaluations/*.md` |
| Skill governance gaps | `specs/skills/*.md` |

### 2.6 Project-Specific Gaps (`.decapod/OVERRIDE.md`)

**Definition:** Gaps between embedded constitution and project needs.

**What qualifies:**
- Project needs custom priority levels
- Specific subsystem needs different defaults
- Custom validation gates required
- Project-specific methodology additions
- Domain-specific patterns not covered
- Integration with project-specific tooling

**Detection Methods:**
- OVERRIDE.md content doesn't address need
- Project repeatedly working around constitution
- Domain-specific gaps not covered by general docs
- Project tooling conflicts with constitution assumptions

**Routing Table for Project Gaps:**

| Gap Type | Route To |
|----------|----------|
| Project overrides | `.decapod/OVERRIDE.md` |
| Project-specific validation | OVERRIDE.md + plugins/VERIFY.md |
| Project methodology | OVERRIDE.md + relevant methodology |

---

## 3. Gap Identification Protocol

### 3.1 Continuous Scanning

Gap identification is not a one-time audit. It happens continuously:

- **During every agent session**: Every time an agent encounters confusion, uncertainty, or a workaround, a gap may exist
- **When validation fails**: `decapod validate` failures are gap signals
- **When agents ask clarifying questions**: Repetitive questions indicate missing guidance
- **When workarounds emerge**: Agents inventing workarounds signal missing functionality
- **When proof surfaces can't validate**: Proof failures reveal implementation gaps
- **During code review**: Human reviewers spot what automated tools miss
- **During incidents**: Post-mortems reveal systemic gaps
- **During architecture decisions**: Decision documentation reveals missing considerations

### 3.2 Gap Signal Detection

**Strong Signals (definite gaps):**
- `decapod validate` fails with new error
- Two docs contradict each other
- Agent can't determine next step
- Proof surface exists but can't be run
- Schema documented but not implemented
- Required feature missing entirely
- Security model has uncovered threat vector
- Data loss path exists

**Medium Signals (likely gaps):**
- Repeated similar questions from different agents
- Workarounds documented as "temporary" (temporary > 2 weeks is permanent)
- SPEC items without graduation timeline
- Claims marked `not_enforced` that seem important
- TODOs without clear resolution path
- Documentation exists but doesn't match code
- Error messages without documented recovery paths

**Weak Signals (potential gaps):**
- Performance could be better
- Minor UX friction
- Missing "nice to have" features
- Undocumented but working behavior
- Style inconsistencies
- Minor code duplication

### 3.3 Gap Triage Questions

When you identify a potential gap, answer these questions:

1. **What layer?** (interface, methodology, plugin, core, spec, project)
2. **What severity?** (critical, high, medium, low)
3. **Who owns it?** (which document/subsystem has authority)
4. **Is it known?** (check existing TODOs, issues, docs)
5. **What's the proof?** (how would we know when it's fixed)

If you cannot answer these questions, continue investigation before documenting the gap.

### 3.4 Gap Identification Tools

**Automated Tools:**
```bash
# Run validation to find structural gaps
decapod validate

# Check subsystem registry consistency
decapod docs list | grep -E 'STUB|SPEC'

# Verify doc graph reachability
decapod validate --check-links

# Check claims enforcement
decapod validate --check-claims
```

**Manual Review:**
- Read new PRs for workarounds that signal missing functionality
- Monitor agent questions for patterns
- Review post-mortems for systemic issues
- Audit architecture decisions for missing considerations
- Survey team for undocumented practices

---

## 4. Gap Documentation & Routing

### 4.1 Document the Gap

Every identified gap should be documented with:

| Field | Description | Example |
|-------|-------------|---------|
| **Title** | Concise description | "CLI surface `--flag-x` lacks value validation schema" |
| **Category** | Layer and type | "Interface Gap: CLI Schema" |
| **Severity** | Impact level | "High" |
| **Evidence** | How you detected it | "`decapod validate` warning, PR #123 workaround" |
| **Impact** | What work is blocked | "Agents can't validate flag values; invalid inputs accepted" |
| **Owner** | Document/subsystem responsible | "interfaces/DOC_RULES.md + implementing subsystem" |
| **Proof** | How to verify when fixed | "`decapod validate` passes; schema doc updated" |
| **Created** | Date identified | "2026-05-10" |
| **Status** | Current state | "Identified" |

### 4.2 Route to Appropriate Subsystem

Use the **routing table** in §2 to determine where the gap belongs.

**Decision Tree:**
```
Is it a missing/incomplete binding contract?
├── YES → interfaces/
└── NO ↓
Is it unclear how to do something?
├── YES → methodology/
└── NO ↓
Is it missing functionality?
├── YES → plugins/ or core/PLUGINS.md
└── NO ↓
Is it navigation/routing?
├── YES → core/DECAPOD.md
└── NO ↓
Is it system-level contract?
├── YES → specs/
└── NO ↓
Is it project-specific?
├── YES → .decapod/OVERRIDE.md
└── UNKNOWN → Continue investigation
```

### 4.3 Create TODO (If Actionable)

If the gap is actionable:
1. Create TODO via `decapod todo add`
2. Tag with appropriate category
3. Reference this GAPS.md section if gap analysis needed
4. Link to relevant subsystem docs
5. Set priority based on severity

**Example TODO creation:**
```bash
decapod todo add "Fix gap: CLI schema missing for X command" \
  --priority high \
  --tags "interface-gap,cli-schema" \
  --description "Category=Interface, Owner=interfaces/DOC_RULES.md, Evidence=decapod validate warning"
```

### 4.4 Update Relevant Index

If the gap reveals missing coverage in an index file:
- Update `core/INTERFACES.md` if interface gaps
- Update `core/METHODOLOGY.md` if methodology gaps
- Update `core/PLUGINS.md` if plugin gaps
- Update `core/DECAPOD.md` if navigation gaps

---

## 5. Gap Lifecycle

```
┌───────────┐    ┌────────────┐    ┌───────┐    ┌──────────┐
│ Identified │───►│ Categorized │───►│ Routed │───►│ Documented │
└───────────┘    └────────────┘    └───────┘    └──────────┘
                                                │
                    ┌───────────────────────────┘
                    ▼
              ┌──────────┐    ┌────────────┐    ┌─────────┐    ┌──────────┐
              │ Ticketed │───►│ In Progress │───►│ Resolved│───►│ Verified │
              └──────────┘    └────────────┘    └─────────┘    └──────────┘
```

**State Definitions:**

| State | Description | Exit Criteria |
|-------|-------------|---------------|
| **Identified** | Gap spotted, not yet categorized | Category determined |
| **Categorized** | Layer and type determined | Owner identified |
| **Routed** | Owner document/subsystem identified | Gap documented |
| **Documented** | Gap described with evidence | TODO created |
| **Ticketed** | TODO created with priority | Work started |
| **In Progress** | Being addressed | Fix implemented |
| **Resolved** | Fix implemented | Proof surface passes |
| **Verified** | Proof surface confirms resolution | TODO closed |

### 5.1 State Transitions

| From | To | Trigger |
|------|----|---------|
| Identified | Categorized | Layer and type determined |
| Categorized | Routed | Owner identified |
| Routed | Documented | Gap documented in appropriate doc |
| Documented | Ticketed | TODO created |
| Ticketed | In Progress | Work begins |
| In Progress | Resolved | Fix implemented |
| Resolved | Verified | Proof surface confirms |
| Any | Identified | New information changes understanding |

---

## 6. Gap Analysis Integration with Subsystems

### 6.1 Integration with TODO System

Gap findings often become TODOs:
- **High-impact gaps** → high-priority TODOs
- **Systemic gaps** → epics with multiple TODOs
- **Methodology gaps** → documentation TODOs
- **Interface gaps** → implementation + doc TODOs

**Workflow:**
1. Gap identified → Create TODO
2. TODO references GAPS.md category
3. Work addresses gap
4. Proof surface confirms resolution
5. TODO closed with evidence

**See:** `plugins/TODO.md` for work tracking

### 6.2 Integration with Validation

Gap detection is often triggered by validation failures:
- `decapod validate` failures
- Doc graph reachability issues
- Schema mismatches
- Store contamination detection

**When validation reveals a gap:**
1. Document the gap
2. Create TODO if actionable
3. Add validation gate if repeatable
4. Update validate taxonomy
5. Document expected vs. actual behavior

**Gap findings should:**
- Add validation gates where possible
- Update validate taxonomy
- Document expected vs. actual behavior

**See:** `interfaces/CONTROL_PLANE.md` §6 for validate doctrine

### 6.3 Integration with Knowledge Base

Gap analysis produces valuable knowledge:
- Why gaps exist (historical context)
- How gaps were resolved (patterns)
- Gap taxonomy and categorization
- Common gap types by subsystem
- Resolution timelines and approaches

**After resolving a gap:**
1. Document the resolution pattern
2. Add to knowledge base if instructive
3. Note what could have prevented it
4. Update methodology if guidance was missing

**See:** `methodology/KNOWLEDGE.md` for knowledge management

### 6.4 Integration with Memory

Agents should remember:
- Gap patterns (avoid repeated gaps)
- Resolution strategies
- Common routing decisions
- Verification approaches
- Prevention strategies

**Memory entries from gap analysis:**
- Patterns of similar gaps
- Effective resolution strategies
- Common mis-routings to avoid
- Proof surfaces that work for verification

**See:** `methodology/MEMORY.md` for learning patterns

---

## 7. Gap Taxonomy Reference

### 7.1 By Layer

| Layer | Gap Type | Index File | Example |
|-------|----------|------------|---------|
| Interfaces | Missing contracts, schemas, invariants | `core/INTERFACES.md` | "No schema for --flag-x" |
| Methodology | Unclear practices, missing guidance | `core/METHODOLOGY.md` | "No guidance for X scenario" |
| Plugins | Missing functionality, incomplete impl | `core/PLUGINS.md` | "Feature Y not implemented" |
| Core | Routing, navigation, coordination | `core/DECAPOD.md` | "Can't find doc for X" |
| Specs | System contracts, security, process | `specs/` | "Security model missing Z" |
| Project | Project-specific overrides | `.decapod/OVERRIDE.md` | "Need custom priority levels" |

### 7.2 By Severity

| Severity | Description | Action | SLA |
|----------|-------------|--------|-----|
| Critical | Blocks work, violates contracts, causes data loss | Immediate TODO, escalate | Immediate |
| High | Causes friction, workarounds needed | High-priority TODO | 24 hours |
| Medium | Inconvenience, unclear guidance | Medium-priority TODO | 1 week |
| Low | Nice to have, optimization | Backlog or knowledge entry | 1 month |

### 7.3 By Lifecycle Stage

| Stage | Gap Characteristic | Typical Resolution |
|-------|-------------------|-------------------|
| Design | Missing spec for planned feature | Add SPEC docs |
| Implementation | STUB without graduation path | Implement or deprioritize |
| Production | REAL but incomplete | Fix or document limitations |
| Maintenance | Drift from documented behavior | Drift recovery |

### 7.4 By Root Cause

| Root Cause | Description | Prevention |
|------------|-------------|------------|
| Incomplete spec | Feature was never fully specified | Require spec before impl |
| Drift | Implementation diverged from spec | Validation gates |
| Missing proof | No verification mechanism | Proof-first development |
| Evolved requirements | Requirements changed, docs didn't | Regular doc refresh |
| Integration gap | Boundary between subsystems undefined | API-first design |

---

## 8. Common Gap Patterns

### 8.1 "SPEC Forever"

**Pattern:** Feature marked SPEC with no graduation timeline

**Detection:**
```bash
# Check PLUGINS.md for old SPEC items
grep "SPEC" constitution/core/PLUGINS.md | grep -v "Graduation"
```

**Characteristics:**
- SPEC item older than 6 months
- No TODO tracking implementation
- No design doc linked
- No explanation for why it's not implemented

**Resolution:**
1. Implement the feature and promote to STUB
2. Or downgrade to IDEA if design is no longer viable
3. Or create explicit "not doing" rationale with deprecation notice

**What breaks if ignored:**
- Trust in SPEC as a meaningful label
- Work planned around unimplemented features
- Design context lost over time

### 8.2 "Documentation Drift"

**Pattern:** Docs say X, code does Y, neither is "wrong" but they differ

**Detection:**
- Validation warnings about schema drift
- Agent confusion about correct behavior
- Error messages that don't match docs

**Example:**
```
# Doc says: "decapod validate runs all proof surfaces"
# Code does: "validate only runs structural checks"

# Neither is wrong, but they diverge
```

**Resolution:**
1. Run drift detection
2. Determine which is "correct" (usually code is truth)
3. Update doc to match code, or fix code to match doc
4. Add validation gate for this drift

**See:** `specs/AMENDMENTS.md` for drift recovery process

### 8.3 "Proof Gap"

**Pattern:** Claim exists in CLAIMS.md but proof surface doesn't verify it

**Detection:**
- Claim marked `not_enforced`
- Proof surface exists but doesn't actually check the claim
- Claim was added without implementing proof

**Example:**
```
claim.doc.real_requires_proof: "REAL requires proof surface"
Status: not_enforced (no validate gate exists)
```

**Resolution:**
1. Implement proof surface
2. Add to validate taxonomy
3. Change enforcement to `partially_enforced` or `enforced`
4. Test the proof surface

**What breaks if ignored:**
- Claims become meaningless
- Agents make promises that can't be verified
- System integrity erodes

### 8.4 "Missing Index"

**Pattern:** Subsystem exists but not in registry

**Detection:**
- CLI command exists but not in PLUGINS.md
- Doc references subsystem that isn't registered
- Truth label doesn't exist in registry

**Example:**
```
# Agent finds "decapod some-new-command"
# But it's not in PLUGINS.md
# Is it canonical?
```

**Resolution:**
1. Determine if the subsystem should be canonical
2. If yes: add to PLUGINS.md with appropriate truth label
3. If no: doc should not reference it as canonical
4. Create owner doc if needed

### 8.5 "Interface Mismatch"

**Pattern:** Two subsystems expect different interfaces

**Detection:**
- Integration failures at boundaries
- Data format inconsistencies between subsystems
- Agents must transform data between subsystems

**Example:**
```
# Subsystem A outputs: {"id": "123", "name": "test"}
# Subsystem B expects: {"ID": "123", "title": "test"}

# No mapping layer exists
```

**Resolution:**
1. Define canonical interface at boundary
2. Add adapter layer or update both subsystems
3. Document the interface contract
4. Add integration tests

### 8.6 "Methodology Vacuum"

**Pattern:** Common task has no documented approach

**Detection:**
- Agents invent different solutions
- Inconsistent outcomes for same task
- No guidance doc exists for recurring scenario

**Example:**
```
# Task: "How to handle partial failures in multi-step workflow"
# No methodology doc covers this
# Agent A: retry all
# Agent B: fail fast
# Agent C: skip and continue
```

**Resolution:**
1. Identify the gap
2. Create methodology guide or update existing guide
3. Include tradeoffs, examples, failure modes
4. Route from relevant docs

---

## 9. Gap Analysis for Leadership

### 9.1 Strategic Gap Assessment

Principals and Architects should periodically:
- Review gap distribution by layer
- Identify systemic gap patterns
- Assess gap resolution velocity
- Prioritize gap categories
- Allocate resources to high-impact gaps

### 9.2 Gap Metrics

Track these metrics over time:

| Metric | What It Measures | How to Collect |
|--------|-----------------|----------------|
| Gap identification rate | New gaps per week | Count new gap TODOs |
| Gap resolution velocity | Time from identified to resolved | TODO timestamps |
| Gap severity distribution | Mix of critical/high/medium/low | Severity field |
| Gap category trends | Which layers have most gaps | Category field |
| Recurring gap patterns | Same root cause gaps | Group by root cause |
| Proof surface coverage | % of claims enforced | CLAIMS.md enforcement field |

### 9.3 Gap Prevention

Proactive measures to reduce gap creation:

1. **Thorough design before implementation**
   - Require SPEC docs before code
   - Review boundaries before building
   - Document failure modes upfront

2. **Proof surfaces for all REAL claims**
   - No REAL without proof
   - Test proof surfaces in CI
   - Verify proof coverage annually

3. **Clear methodology documentation**
   - Write guides before they're urgently needed
   - Update guides when workarounds emerge
   - Include failure modes, not just happy paths

4. **Regular validation**
   - Run `decapod validate` frequently
   - Fix warnings before they become errors
   - Add new validation gates for repeatable issues

5. **Cross-subsystem integration testing**
   - Test boundaries between subsystems
   - Verify data format compatibility
   - Exercise error paths

---

## 10. Emergency Gap Protocol

### 10.1 Critical Gap Detected

If you find a gap that:
- Violates security contract
- Causes data loss
- Breaks validation completely
- Creates split-brain state
- Exposes confidential data
- Enables unauthorized access

**Immediate actions:**

1. **STOP** — Do not proceed with any downstream work
2. **DOCUMENT** — Record the gap with evidence (commands, outputs, screenshots)
3. **NOTIFY** — Alert relevant channels (security@, on-call, architecture)
4. **CONSULT** — Read `plugins/EMERGENCY_PROTOCOL.md` for escalation procedures
5. **CREATE** — Create critical TODO with gap details
6. **ISOLATE** — If possible, prevent the gap from causing further damage
7. **DO NOT PROCEED** — Wait for resolution before continuing

**What NOT to do:**
- Do not try to "fix it quickly" without understanding the root cause
- Do not ignore it hoping it will go away
- Do not work around it without documenting
- Do not tell users to "just ignore" the warning

### 10.2 Authority Escalation

If gap crosses authority boundaries:
1. Document the ambiguity completely
2. Propose authority assignment
3. Reference `interfaces/DOC_RULES.md` §8 (Decision Rights Matrix)
4. Route to `specs/AMENDMENTS.md` if needed
5. Do not proceed until authority is clarified

---

## 11. Gap Analysis Checklist

**When analyzing system for gaps, verify:**

### Structural Validation
- [ ] Run `decapod validate` and catalog all warnings
- [ ] Check for broken links in doc graph
- [ ] Verify all `STUB`/`SPEC` items have graduation paths
- [ ] Review subsystem registry for stale entries

### Claims and Proof
- [ ] Identify `not_enforced` claims in CLAIMS.md
- [ ] Verify proof surfaces exist for all REAL claims
- [ ] Test proof surfaces actually run and pass
- [ ] Check for claims without owner docs

### Subsystem Health
- [ ] Review PLUGINS.md registry vs. actual subsystems
- [ ] Check for phantom REAL entries
- [ ] Verify deprecation routing is accurate
- [ ] Review SPEC items for implementation timelines

### Methodology Coverage
- [ ] Survey methodology docs for actionable guidance
- [ ] Check for scenarios without guidance
- [ ] Review guides for contradictions
- [ ] Verify guide links are accurate

### Navigation and Routing
- [ ] Verify `core/DECAPOD.md` reaches all canonical docs
- [ ] Check `## Links` sections are complete
- [ ] Verify index files are accurate
- [ ] Review OVERRIDE.md for project-specific gaps

### Emergency Preparedness
- [ ] Review emergency protocols for coverage gaps
- [ ] Verify security model covers all threat vectors
- [ ] Check for missing error handling paths
- [ ] Review data loss prevention measures

---

## 12. Gap Resolution Verification

Every resolved gap needs verification:

**Resolution Checklist:**
- [ ] Proof surface passes
- [ ] Documentation updated
- [ ] Index files current
- [ ] TODO closed with evidence
- [ ] Knowledge entry created (if pattern)
- [ ] No new gaps introduced

**Verification Process:**
```bash
# 1. Run the proof surface
decapod validate

# 2. Verify specific claim/feature
decapod validate --check <specific-check>

# 3. Verify no regression in related areas
decapod validate --full

# 4. Check TODO is closed
decapod todo list --status closed --since <date>
```

**Pre-Resolution Verification (what must pass):**
```bash
# Structural validation must pass
decapod validate

# Specific gap-related checks must pass
decapod validate --check <gap-related-check>

# No new warnings introduced
decapod validate 2>&1 | grep -i warning
```

---

## Links

### Core Router
- [core/DECAPOD.md](core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [core/ENGINEERING_EXCELLENCE.md](core/ENGINEERING_EXCELLENCE.md) - **Oracle for Engineering Standards (CTO->Principal)**
- [core/METHODOLOGY.md](core/METHODOLOGY.md) - Methodology guides index

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
- [interfaces/CONTROL_PLANE.md](interfaces/CONTROL_PLANE.md) - Sequencing patterns and validation doctrine
- [interfaces/DOC_RULES.md](interfaces/DOC_RULES.md) - Doc compilation rules
- [interfaces/CLAIMS.md](interfaces/CLAIMS.md) - Promises ledger
- [interfaces/GLOSSARY.md](interfaces/GLOSSARY.md) - Term definitions
- [interfaces/STORE_MODEL.md](interfaces/STORE_MODEL.md) - Store semantics
- [interfaces/TESTING.md](interfaces/TESTING.md) - Testing contract

### Practice (Methodology Layer)
- [methodology/SOUL.md](methodology/SOUL.md) - Agent identity
- [methodology/ARCHITECTURE.md](methodology/ARCHITECTURE.md) - Architecture practice
- [methodology/KNOWLEDGE.md](methodology/KNOWLEDGE.md) - Knowledge curation
- [methodology/MEMORY.md](methodology/MEMORY.md) - Memory and learning
- [methodology/TESTING.md](methodology/TESTING.md) - Testing practice
- [methodology/CI_CD.md](methodology/CI_CD.md) - CI/CD practice

### Domain Architecture Patterns
- [architecture/UI.md](architecture/UI.md) - UI architecture patterns and component design
- [architecture/FRONTEND.md](architecture/FRONTEND.md) - Frontend architecture patterns
- [architecture/WEB.md](architecture/WEB.md) - Web architecture patterns
- [architecture/DATA.md](architecture/DATA.md) - Data architecture patterns
- [architecture/SECURITY.md](architecture/SECURITY.md) - Security architecture patterns
- [architecture/CLOUD.md](architecture/CLOUD.md) - Cloud deployment patterns

### Operations (Plugins Layer)
- [plugins/TODO.md](plugins/TODO.md) - Work tracking
- [plugins/VERIFY.md](plugins/VERIFY.md) - Validation subsystem
- [plugins/MANIFEST.md](plugins/MANIFEST.md) - Manifest patterns
- [plugins/EMERGENCY_PROTOCOL.md](plugins/EMERGENCY_PROTOCOL.md) - Emergency protocols
- [plugins/KNOWLEDGE.md](plugins/KNOWLEDGE.md) - Knowledge subsystem
- [plugins/FEDERATION.md](plugins/FEDERATION.md) - Federation subsystem

---

## Project Override Context

**Current gap themes:**
- Integration maturity: some domain adapters are still placeholder-level
- Verification depth: broaden end-to-end and backend-parity test coverage
- Runtime ergonomics: improve capability granting, versioning, and visibility of subsystem status
- Interface completeness: close remaining stubs in automation and extension lifecycle workflows

**Completed themes:**
- Stronger sandboxing and tool isolation model
- Better context handling and background maintenance flows
- Improved control plane surfaces for channels, routines, and extension management
- Store purity enforcement between user and repo stores

**Systemic observations:**
- Gap velocity has decreased with improved validation gates
- Proof surface coverage is expanding (now ~65% of claims have proof)
- Methodology gaps are the largest remaining category by count
- Critical gaps have dropped significantly; remaining critical gaps are security-related