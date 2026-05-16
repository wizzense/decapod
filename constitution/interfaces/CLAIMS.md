# CLAIMS.md - Claims Ledger (Promises and Proof Surfaces)

**Authority:** interface (registry of guarantees and their proof surfaces)
**Layer:** Interfaces
**Binding:** Yes
**Scope:** table-driven ledger of explicit guarantees/invariants and where they are proven/enforced
**Non-goals:** replacing specs; this is an index of promises, not the full spec text

---

## Table of Contents

1. [Purpose and Scope](#1-purpose-and-scope)
2. [Table Schema](#2-table-schema)
3. [Claims Registry](#3-claims-registry)
4. [Workflow: Registering/Updating a Claim](#4-workflow-registeringupdating-a-claim)
5. [Enforcement Levels Explained](#5-enforcement-levels-explained)
6. [Claim Lifecycle](#6-claim-lifecycle)
7. [Proof Surface Reference](#7-proof-surface-reference)

---

This ledger exists to prevent "forgotten invariants" and accidental promise drift.

**Rule**: If a canonical doc makes a guarantee/invariant, it MUST be registered here with a claim-id.

---

## 1. Purpose and Scope

The Claims Ledger serves three functions:

1. **Completeness check**: Prevents guarantees from being made and forgotten
2. **Enforcement visibility**: Shows which promises have proof surfaces and which don't
3. **Change impact analysis**: When changing a guarantee, which tests need updating?

**What belongs in the ledger:**
- Behavioral guarantees (what the system will/won't do)
- Invariants (conditions that must always hold)
- Performance requirements (latency, throughput, availability)
- Security properties (authentication, authorization, confidentiality)
- Data integrity constraints

**What doesn't belong:**
- Implementation details
- Documentation structure requirements (those go in DOC_RULES)
- Methodology guidance (those are non-binding)

---

## 2. Table Schema

Each claim row contains:

| Column | Description | Required |
|--------|-------------|----------|
| **Claim ID** | Stable identifier in format `claim.<domain>.<name>` | Yes |
| **Claim (normative)** | The promise, phrased as a single sentence | Yes |
| **Owner Doc** | Where the claim is fully specified | Yes |
| **Enforcement** | `enforced` \| `partially_enforced` \| `not_enforced` | Yes |
| **Proof Surface** | Named, runnable surface(s) that detect drift | Yes for enforced |
| **Notes** | Brief context, limitations, or migration pointers | No |

### 2.1 Claim ID Format

Format: `claim.<domain>.<name>`

| Domain | Use For |
|--------|---------|
| `doc` | Documentation and doc compilation |
| `store` | Store model and state semantics |
| `foundation` | Core system properties |
| `agent` | Agent behavior and control plane |
| `proof` | Proof and validation doctrine |
| `concurrency` | Concurrent operation guarantees |
| `federation` | Federated data properties |
| `risk_policy` | Risk management and policy |
| `review` | Code review and approval |
| `context` | Context management |
| `lcm` | Lifecycle management |
| `map` | Mapping and delegation |
| `todo` | Work tracking |
| `git` | Git workflow and workspace |
| `session` | Session management |
| `validate` | Validation behavior |
| `architecture` | Architecture requirements |
| `knowledge` | Knowledge management |
| `workunit` | Work unit tracking |
| `eval` | Evaluation and judgment |
| `skill` | Skill management |
| `harness` | Test harness and evidence |

### 2.2 Enforcement Levels

| Level | Meaning | What it requires |
|-------|---------|------------------|
| `enforced` | Proof surface exists and runs in CI | Proof surface MUST pass for promotion |
| `partially_enforced` | Proof exists but doesn't cover all cases | Known gaps are documented |
| `not_enforced` | No proof surface | Claim exists but cannot be automatically verified |

---

## 3. Claims Registry

### 3.1 Documentation Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.doc.decapod_is_router_only` | `core/DECAPOD.md` routes and prioritizes canonical docs but does not define or override behavioral rules. | `core/DECAPOD.md` | `partially_enforced` | `decapod validate` (doc graph + canon headers) | Social + doc-layer boundary; code enforcement is limited. |
| `claim.doc.no_shadow_policy` | If a rule is not declared in canonical docs, it is not enforceable. | `interfaces/DOC_RULES.md` | `partially_enforced` | `decapod validate` (doc graph) | Enforcement of "shadow policy" is largely procedural. |
| `claim.doc.real_requires_proof` | Any `REAL` interface claim requires a named proof surface; otherwise it must be `STUB` or `SPEC`. | `interfaces/DOC_RULES.md` | `not_enforced` | planned: validate checks for proof surface annotations | Current enforcement is doc-level; future validate gate can check. |
| `claim.doc.decapod_reaches_all_canonical` | `core/DECAPOD.md` reaches every canonical doc via the `## Links` graph. | `interfaces/DOC_RULES.md` | `enforced` | `decapod validate` (doc graph gate) | Prevents buried canonical law and unreachable contracts. |
| `claim.doc.no_duplicate_authority` | No requirement may be defined in multiple canonical docs; duplicates must defer to the owner doc. | `interfaces/DOC_RULES.md` | `not_enforced` | planned: validate checks for duplicated requirements | Procedural today; becomes enforceable only with additional tooling. |
| `claim.doc.no_contradicting_canon` | If two canonical binding docs appear to disagree, the system is invalid; resolution is amendment, not interpretation. | `specs/AMENDMENTS.md` | `not_enforced` | `decapod validate` (planned: contradiction checks) | Humans must treat contradictions as a stop condition. |
| `claim.doc.readme_human_only` | README is human-facing product documentation; agent-operational rules must live in entrypoint and constitution surfaces. | `core/DECAPOD.md` | `not_enforced` | planned: docs-surface partition gate | Prevents README from becoming implicit agent policy. |

### 3.2 Store Model Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.store.blank_slate` | A fresh user store contains no TODOs unless the user adds them. | `interfaces/STORE_MODEL.md` | `enforced` | `decapod validate --store user` | Protects user-store privacy and blank slate semantics. |
| `claim.store.no_auto_seeding` | Repo store content must never appear in the user store automatically. | `interfaces/STORE_MODEL.md` | `enforced` | `decapod validate --store user` | Prevents cross-store contamination. |
| `claim.store.explicit_store_selection` | Mutating commands must be treated as undefined unless store context is explicit; `--store` is preferred and `--root` is dangerous. | `interfaces/STORE_MODEL.md` | `partially_enforced` | `decapod validate` (store invariants) | CLI behavior may still allow footguns; treated as a red-line constraint. |
| `claim.store.decapod_cli_only` | Agents must not read/write `<repo>/.decapod/*` files directly; access must go through `decapod` CLI surfaces. | `interfaces/STORE_MODEL.md` | `enforced` | `decapod validate` (Four Invariants Gate marker checks) | Prevents jailbreak-style state tampering and out-of-band mutation. |

### 3.3 Foundation Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.foundation.intent_state_proof_primitives` | Decapod governance is anchored on explicit intent, explicit state boundaries, and executable proof surfaces. | `core/DECAPOD.md` | `partially_enforced` | `decapod validate` + canonical doc graph gates | Foundation doctrine is explicit; full semantic enforcement remains incremental. |
| `claim.foundation.daemonless_repo_native_canonicality` | Decapod remains daemonless and repo-native for promotion-relevant state and evidence. | `specs/SYSTEM.md` | `partially_enforced` | `decapod validate` + repo-native manifest/provenance gates | Operationally enforced in current control plane; hardening continues through gate expansion. |
| `claim.foundation.proof_gated_promotion` | Promotion-relevant outcomes are invalid without executable proof and machine-verifiable artifacts. | `specs/SYSTEM.md` | `partially_enforced` | `decapod validate` + workspace publish proof gates | Publish paths enforce this today; broader policy coupling is still evolving. |

### 3.4 Internalization Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.internalize.explicit_attach_lease` | Internalized context may affect inference only through an explicit session-scoped attach lease; ambient reuse is forbidden. | `interfaces/INTERNALIZATION_SCHEMA.md` | `partially_enforced` | `decapod internalize attach` + `decapod internalize detach` + `decapod validate` internalization gate | Lease files and provenance logs are enforced; downstream inference callers must honor the contract. |
| `claim.internalize.best_effort_not_replayable` | Best-effort internalizer profiles must never claim replayability and must record binary/runtime fingerprints. | `interfaces/INTERNALIZATION_SCHEMA.md` | `enforced` | `decapod internalize create` + `decapod internalize inspect` + `decapod validate` internalization gate | Prevents fake reproducibility claims for non-deterministic profiles. |

### 3.5 Agent Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.agent.invocation_checkpoints_required` | Agents must call Decapod before plan commitment, before mutation, and after mutation for proof. | `interfaces/CONTROL_PLANE.md` | `partially_enforced` | `decapod todo` ownership records + `decapod validate` + required tests | Enforcement is partly procedural until explicit checkpoint trace gate exists. |
| `claim.agent.no_capability_hallucination` | Agents must not claim capabilities absent from the Decapod command surface. | `interfaces/CONTROL_PLANE.md` | `not_enforced` | planned: capability-claim consistency gate | Missing surfaces must be reported as gaps, not fabricated behavior. |
| `claim.agent.intent_refinement_required` | Agents MUST ask clarifying questions and refine requirements with the user BEFORE burning tokens on inference/implementation. | `core/INTERFACES.md` | `not_enforced` | planned: intent-refinement gate | SPEC pending: agent must produce a refined design doc before code generation. |

### 3.6 Proof and Concurrency Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.proof.executable_check` | A "proof" is an executable check that can fail loudly (tests, linters, validators, etc). No new DSL. | `core/PLUGINS.md` | `enforced` | `decapod validate` | Definition is normative; proof registry (Epoch 1) will formalize. |
| `claim.concurrency.no_git_solve` | Decapod does not "solve" Git merge conflicts; it reduces collisions via work partitioning and proof gates. | `core/PLUGINS.md` | `partially_enforced` | `decapod validate` (workspace/protected-branch gates) | Prevents over-claiming on concurrency; residual merge semantics remain Git-native. |

### 3.7 Plugin/Subsystem Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.broker.is_spec` | DB Broker (serialized writes, audit) is SPEC, not REAL. Do not claim it is implemented. | `core/PLUGINS.md` | `enforced` | `decapod validate` (truth label check) | Will graduate to REAL in Epoch 4. |
| `claim.test.mandatory` | Every code change must have corresponding tests. No exceptions. | `methodology/ARCHITECTURE.md` | `enforced` | `cargo test` + CI | Tests gate merge; untested code is rejected. |

### 3.8 Federation Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.federation.store_scoped` | Federation data exists only under the selected store root. | `plugins/FEDERATION.md` | `enforced` | `decapod validate` (federation.store_purity gate) | Prevents cross-store contamination. |
| `claim.federation.provenance_required_for_critical` | Critical federation nodes must have ≥1 valid provenance source with scheme prefix. | `plugins/FEDERATION.md` | `enforced` | `decapod validate` (federation.provenance gate) | Prevents hallucination anchors. |
| `claim.federation.append_only_critical` | Critical types (decision, commitment) cannot be edited in place; must be superseded. | `plugins/FEDERATION.md` | `enforced` | `decapod validate` (federation.write_safety gate) | Write-safety for operational truth. |
| `claim.federation.lifecycle_dag_no_cycles` | The supersedes edge graph contains no cycles. | `plugins/FEDERATION.md` | `enforced` | `decapod validate` (federation.lifecycle_dag gate) | Prevents infinite supersession loops. |

### 3.9 Risk Policy Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.risk_policy.single_contract_source` | Risk tiers, required checks, docs drift, and evidence requirements are defined in one machine-readable contract source. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: `risk-policy-gate` + `decapod validate` contract-shape checks | SPEC until runtime gate consumes contract as source of truth. |
| `claim.risk_policy.preflight_before_fanout` | Risk-policy preflight must complete successfully before expensive CI fanout starts. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: `risk-policy-gate` | SPEC pending CI orchestration enforcement. |
| `claim.review.sha_freshness_required` | Review-agent state is valid only when tied to current PR head SHA. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: review check-run head SHA verifier | SPEC pending implementation. |
| `claim.review.single_rerun_writer` | Exactly one canonical rerun writer may request review reruns, deduped by marker plus head SHA. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: rerun-writer dedupe gate | SPEC pending enforcement surface. |
| `claim.review.remediation_loop_reenters_policy` | Automated remediation must push to the same PR branch and re-enter policy gates; bypass is forbidden. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: remediation workflow policy gate | SPEC pending deterministic remediation implementation. |
| `claim.evidence.manifest_required_for_ui` | UI and critical flow changes require machine-verifiable evidence manifests and verifier checks. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: `browser-evidence-verify` + `decapod validate` marker checks | SPEC until artifact verifier is mandatory. |
| `claim.harness.incident_to_case_loop` | Production regressions must map to harness-gap cases and tracked follow-up. | `interfaces/RISK_POLICY_GATE.md` | `not_enforced` | planned: harness-gap lifecycle checks | SPEC pending workflow linkage automation. |

### 3.10 Context Pack Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.context_pack.canonical_layout` | Agent context pack uses canonical `.decapod/context` and `.decapod/memory` layout, not root file sprawl. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: `decapod validate` context-pack layout gate | SPEC pending directory/shape enforcement. |
| `claim.context_pack.deterministic_load_order` | Context pack load order is deterministic across runners. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: load-order validation gate | SPEC pending loader checks. |
| `claim.context_pack.mutation_authority_rules` | High-authority context files require human-owned or explicit approval updates. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: mutation-policy enforcement gate | SPEC pending policy engine integration. |
| `claim.memory.append_only_logs` | Operational memory logs are append-first and cannot be silently erased in place. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: append-only validation checks | SPEC pending log write-policy enforcement. |
| `claim.memory.distill_proof_required` | `memory.md` must be produced by deterministic distillation with a named proof surface. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: deterministic distill proof check | SPEC pending distill command/proof surface. |
| `claim.context_pack.security_scoped_loading` | Sensitive context-pack memory is scope-gated and not auto-loaded into broad shared contexts. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: scoped-load policy checks | SPEC pending runtime loader policy enforcement. |
| `claim.context_pack.correction_loop_governed` | Corrections must be persisted through control-plane artifacts and proofed, not mental notes. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: correction-to-proof audit gate | SPEC pending end-to-end trace enforcement. |
| `claim.context.capsule.deterministic` | Context capsule query output is deterministic for identical inputs and canonical source set. | `interfaces/AGENT_CONTEXT_PACK.md` | `not_enforced` | planned: deterministic capsule serialization test + validate gate | Prevents non-reproducible context packs from becoming promotion inputs. |
| `claim.context.capsule.policy_enforced` | Context capsule issuance is policy-bound by risk tier and fails closed on scope/tier/revision violations. | `interfaces/AGENT_CONTEXT_PACK.md` | `partially_enforced` | `govern capsule query` policy checks + `decapod validate` context-capsule-policy gate | Broker/mutation/promotion coupling is staged; issuance boundary is enforced in v1. |

### 3.11 Project Specs Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.project_specs.canonical_set_enforced` | Local project specs use a fixed canonical `specs/*.md` set that Decapod scaffolds, validates, and resolves into context. | `interfaces/PROJECT_SPECS.md` | `partially_enforced` | `decapod init` + `decapod validate` (project specs gate) + `context.resolve` local spec payload | Prevents drift between repo-local specs and constitution-governed runtime behavior. |

### 3.12 LCM Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.lcm.append_only_ledger` | LCM events are stored in append-only JSONL ledger (`lcm.events.jsonl`) and never mutated or deleted. | `interfaces/LCM.md` | `enforced` | `decapod validate` (LCM Immutability Gate) | Enforced via validate_lcm_immutability gate. |
| `claim.lcm.content_hash_deterministic` | Content hash is SHA256 of raw content bytes — deterministic across runs. | `interfaces/LCM.md` | `enforced` | `decapod validate` (LCM Immutability Gate) | Enforced via validate_lcm_immutability gate. |
| `claim.lcm.index_rebuildable` | LCM SQLite index (`lcm.db`) is always rebuildable from `lcm.events.jsonl`. | `interfaces/LCM.md` | `enforced` | `decapod lcm rebuild --validate` + `decapod validate` (LCM Rebuild Gate) | Enforced via validate_lcm_rebuild_gate. |
| `claim.lcm.summary_deterministic` | Same originals in timestamp order produce the same summary hash across runs. | `interfaces/LCM.md` | `enforced` | `decapod lcm summarize` produces stable hash | Deterministic by construction. |
| `claim.map.scope_reduction_invariant` | Agentic map delegation MUST declare retained scope; empty retain is rejected. | `interfaces/LCM.md` | `enforced` | `decapod map agentic --retain` required | Enforced in CLI argument parsing. |

### 3.13 Git/Workspace Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.git.container_workspace_required` | Git-tracked implementation work must execute in Docker-isolated git workspaces rooted at `.decapod/workspaces/*`, not by directly editing the host repository working tree. Inside containers, `validate` only verifies build correctness (compile, test, lint) - git workspace gates are skipped. Host-side Git operations (commit, push, PR) happen after exiting the container. | `specs/GIT.md` | `enforced` | `decapod validate` (Git Workspace Context Gate, skipped in container) | Container validate is build-only; git ops happen on host. |
| `claim.git.no_direct_main_push` | Direct commits/pushes to protected branches (master/main/production/stable/release/*) are forbidden; work must happen in working branches. | `specs/GIT.md` | `enforced` | `decapod validate` (Git Protected Branch Gate) | Enforced via validate gate checking current branch and unpushed commits. |
| `claim.git.container_runtime_preflight_required` | Container workspace runs must pass runtime-access preflight and fail loudly with elevated-permission remediation when access is denied. | `specs/GIT.md` | `partially_enforced` | `container.run` runtime `info` preflight + permission-aware error diagnostics | Enforced in container runtime preflight; broader policy-level enforcement remains future work. |

### 3.14 Session/Security Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.session.agent_password_required` | Session access requires agent identity plus an ephemeral per-session password stored in process-local OnceLock (not env vars); expired sessions trigger cleanup and assignment eviction. | `specs/SECURITY.md` | `enforced` | `session.acquire` credential issuance + `ensure_session_valid` password check + stale-session cleanup hook | Enforced via process-local password storage - no longer exposed in environment. |

### 3.15 Validation Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.validate.bounded_termination` | `decapod validate` MUST terminate in bounded time and return a typed failure under DB lock contention. | `interfaces/TESTING.md` | `enforced` | `tests/validate_termination.rs` + `DECAPOD_VALIDATE_TIMEOUT_SECS` timeout path | Prevents proof-gate hangs from becoming cultural bypass. |
| `claim.validate.no_cross_turn_lock_residency` | No single agent session may hold validation-related datastore locks across multiple turns/commands. | `interfaces/CONTROL_PLANE.md` | `partially_enforced` | `tests/validate_termination.rs` + contention integration tests | Locking discipline is implemented in command-scoped paths; broader contention coverage remains in progress. |

### 3.16 Architecture Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.architecture.artifact_required_for_governed_execution` | Governed execution architecture directives MUST be defined in constitution interfaces, not mutable runtime artifact stores. | `interfaces/ARCHITECTURE_FOUNDATIONS.md` | `not_enforced` | planned: architecture directive gate | Keeps architecture policy repo-native and constitutional. |
| `claim.architecture.intent_to_design_traceability` | Architecture directives MUST require traceability from intent to system design, invariants, tradeoffs, verification, and rollout operations. | `interfaces/ARCHITECTURE_FOUNDATIONS.md` | `not_enforced` | planned: intent-to-architecture traceability gate | Ensures user intent is translated into senior-level architecture reasoning before promotion. |

### 3.17 Knowledge Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.knowledge.provenance_required` | Every procedural memory entry must cite evidence (commit, PR, doc, test, or transcript). | `interfaces/KNOWLEDGE_STORE.md` | `enforced` | `decapod validate` (Knowledge Integrity Gate) | Enforced via validate_knowledge_integrity gate. |
| `claim.knowledge.directional_flow` | Episodic observations cannot flow directly into procedural/semantic memory. Must use explicit promotion artifact + human approval. | `interfaces/KNOWLEDGE_STORE.md` | `not_enforced` | planned: gate in knowledge promote | Blocks direct friction→procedural writes. |
| `claim.knowledge.promotion.firewall` | Promotion-relevant procedural knowledge must pass explicit promotion firewall event requirements (evidence + approval + append-only ledger). | `interfaces/KNOWLEDGE_STORE.md` | `not_enforced` | planned: knowledge promotion firewall gate + ledger schema checks | Prevents advisory memory from silently becoming promotion authority. |
| `claim.knowledge.versioned_schema` | Knowledge store uses versioned schemas. No breaking changes without migration path. | `interfaces/KNOWLEDGE_STORE.md` | `not_enforced` | planned: schema migration validation | Readers never break on writes. |

### 3.18 Workunit Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.workunit.manifest.schema_deterministic` | Work unit manifests use a deterministic schema and transition contract for intent/spec/state/proof lineage. | `interfaces/PLAN_GOVERNED_EXECUTION.md` | `not_enforced` | planned: work unit schema determinism tests + validate gate | Pins promotion readiness to reproducible task-scoped artifacts. |
| `claim.workunit.capsule_policy_lineage_required` | VERIFIED workunits and publish gating require a deterministic context capsule with non-empty policy lineage bound to the same task id. | `interfaces/PLAN_GOVERNED_EXECUTION.md` | `partially_enforced` | `decapod validate` workunit gate + `workspace publish` workunit gate + `tests/workunit_publish_gate.rs` | Enforced at workunit/publish boundary; broader promotion lineage joins remain staged. |

### 3.19 Evaluation Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.eval.variance.repeatable_settings` | Promotion-relevant variance evals MUST capture reproducible settings in EVAL_PLAN and compare under matched lineage. | `specs/evaluations/VARIANCE_EVALS.md` | `partially_enforced` | `decapod eval plan` + `decapod eval aggregate` settings/hash checks | Cross-plan mismatch is blocked unless explicitly acknowledged. |
| `claim.eval.judge.json_contract` | Judge verdicts MUST conform to strict JSON contract and bounded-time execution. | `specs/evaluations/JUDGE_CONTRACT.md` | `partially_enforced` | `decapod eval judge` (typed errors: `EVAL_JUDGE_JSON_CONTRACT_ERROR`, `EVAL_JUDGE_TIMEOUT`) | Malformed or timed-out judgments are promotion blockers. |
| `claim.eval.bootstrap_ci` | Non-deterministic promotion decisions MUST use repeated runs with bootstrap confidence intervals. | `specs/evaluations/VARIANCE_EVALS.md` | `partially_enforced` | `decapod eval aggregate` + deterministic CI tests | Prevents one-shot variance blindness. |
| `claim.eval.no_silent_regressions` | Promotion MUST fail on statistical regression or insufficient run count when eval gate is required. | `specs/engineering/FRONTEND_BACKEND_E2E.md` | `partially_enforced` | `decapod eval gate` + `decapod validate` + publish eval gate check | Enforced when eval gate requirement artifact is present. |

### 3.20 Skill Claims

| Claim ID | Claim (normative) | Owner Doc | Enforcement | Proof Surface | Notes |
|----------|-------------------|----------|-------------|--------------|-------|
| `claim.skill.card.deterministic` | Imported SKILL.md content MUST produce deterministic SKILL_CARD hashes for identical source content. | `specs/skills/SKILL_GOVERNANCE.md` | `partially_enforced` | `decapod data aptitude skill import --write-card` + `decapod validate` skill-card gate | Hash ignores timestamp fields to preserve reproducibility. |
| `claim.skill.resolve.deterministic` | Skill resolution for identical query + identical skill-store state MUST produce deterministic resolution hash. | `specs/skills/SKILL_GOVERNANCE.md` | `partially_enforced` | `decapod data aptitude skill resolve` + deterministic test vectors | Prevents non-repeatable skill selection in multi-agent runs. |
| `claim.skill.no_unverified_authority` | Skill prose is non-authoritative unless translated into Decapod artifacts/store entries. | `specs/skills/SKILL_GOVERNANCE.md` | `partially_enforced` | `decapod validate` skill artifact gates + aptitude skill store | Blocks promotion dependence on external unmanaged skill text. |

---

## 4. Workflow: Registering/Updating a Claim

### 4.1 Adding a New Claim

When adding a new guarantee:

1. **Identify the claim ID**: Use `claim.<domain>.<name>` format
2. **Write the claim**: Phrase as a single sentence promise
3. **Find the owner doc**: Which document has full details?
4. **Assess enforcement**: Is there a proof surface?
5. **Add to this ledger**: Add the row with all columns
6. **Update owner doc**: Add claim-id reference near the guarantee
7. **Create proof if needed**: If `enforced` is claimed, proof must exist

### 4.2 Updating a Claim

When changing a guarantee:

1. **Update the claim text**: Edit the claim column
2. **Update owner doc**: Ensure alignment
3. **Update enforcement if needed**: If proof surface changes
4. **Add migration note**: In Notes column if behavior changes
5. **Follow deprecation if changing meaning**: See `core/DEPRECATION.md`

### 4.3 Deprecating a Claim

When a claim is no longer true:

1. **Mark as not_enforced** if proof no longer runs
2. **Or remove from ledger** if the guarantee no longer exists
3. **Add deprecation note** in Notes column
4. **Update owner doc** to reflect change

---

## 5. Enforcement Levels Explained

### 5.1 `enforced`

The proof surface exists and runs automatically. This claim blocks promotion if the proof fails.

**Requirements:**
- Proof surface must be named and runnable
- Proof must run in CI or validation
- Failure must block promotion

### 5.2 `partially_enforced`

Proof exists but doesn't cover all cases. Known gaps are documented in Notes.

**What this means:**
- Some scenarios are verified
- Other scenarios are documented as unverified
- Progress on coverage is tracked

### 5.3 `not_enforced`

No automated verification exists. The claim represents an intention or expectation, not a guarantee.

**What this means:**
- Humans must verify compliance
- No automated blocking if violated
- Should be prioritized for proof surface creation

---

## 6. Claim Lifecycle

```
Proposed → Accepted → [Enforced | Partially Enforced | Not Enforced]
    ↓
Deprecated → Removed
```

**Proposed**: Claim drafted, seeking feedback
**Accepted**: Finalized and registered
**Enforced**: Proof surface exists and runs
**Partially Enforced**: Some coverage, some gaps
**Not Enforced**: Intent only, no proof
**Deprecated**: Being phased out
**Removed**: No longer relevant

---

## 7. Proof Surface Reference

### 7.1 Core Proof Surfaces

| Proof Surface | What It Checks |
|---------------|----------------|
| `decapod validate` | Structural validity, doc graph, store invariants |
| `decapod validate --store user` | User store blank slate, no seeding |
| `decapod validate --check claims` | Claims enforcement status |
| `cargo test` | Unit and integration tests |
| `cargo clippy` | Linting and code quality |
| `cargo audit` | Security vulnerabilities |

### 7.2 Subsystem Proof Surfaces

See `core/PLUGINS.md` for subsystem-specific proof surfaces.

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
- `interfaces/STORE_MODEL.md` - Store semantics
- `interfaces/CONTROL_PLANE.md` - Sequencing patterns
- `interfaces/GLOSSARY.md` - Term definitions
- `interfaces/TESTING.md` - Testing contract
- `interfaces/LCM.md` - Lifecycle management
- `interfaces/PROJECT_SPECS.md` - Project specs schema
- `interfaces/RISK_POLICY_GATE.md` - Risk policy gate
- `interfaces/AGENT_CONTEXT_PACK.md` - Agent context pack