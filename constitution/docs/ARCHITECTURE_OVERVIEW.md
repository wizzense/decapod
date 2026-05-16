# Architecture Overview (Canonical)

## 1. Storage Boundary

Decapod has one governed repo-native state root for project operations: `<repo>/.decapod`.

Rules:

- Promotion-relevant state MUST be repo-native.
- Agents MUST use Decapod CLI/RPC for state mutation.
- `.decapod` direct edits are forbidden.

## 2. Execution Posture

Decapod is background infrastructure for agents. It is invoked explicitly, performs a bounded control-plane action, writes auditable state or artifacts when required, and exits.

Architectural constraints:

- No required daemon or hidden remote coordinator.
- No provider-specific coupling to one coding agent.
- No human-facing workflow app as the primary interface.
- CLI/RPC surfaces exist for agents and automation; humans primarily inspect generated artifacts and proof.

## 3. Governance Hierarchy

Recursive improvement and agent self-correction are governed by this authority order:

1. user intent
2. project constitution
3. repo rules
4. task/spec constraints
5. agent role contract
6. proof requirements
7. stop conditions

Agents may propose improvements at higher layers, but promotion requires explicit artifacts and proof. Agent-local execution cannot silently rewrite higher-level intent or bypass proof requirements.

## 3.1 Recursive Improvement Passes

Recursive agent loops are allowed only as constitution-authorized passes over bounded deficiencies. A prompt such as "improve something" must become an explicit recursive improvement pass artifact before execution.

Each pass must answer:

- What deficiency was observed?
- Which parent task or spec owns the deficiency?
- Which constitutional rule authorizes the pass?
- What is allowed to change?
- What is forbidden to change?
- What proof is required?
- What stop condition prevents infinite polishing?
- What risk level applies?
- Does this require user approval?

`decapod validate` fails closed when a recursive pass lacks authority, parent lineage, concrete proof, bounded scope, a stop condition, or when it mutates parent intent, expands scope, weakens governance, or touches forbidden paths.

The artifact path is `governance/recursive_passes/*.json` under the repo state root. This is a validation surface, not a workflow engine.

## 4. Artifact Model

Core artifacts:

- Intent artifacts: `INTENT.md`, `SPEC.md`, ADRs.
- Claims artifacts: interface claims and proof obligations.
- Proof artifacts: validation reports, state-commit records, verification outputs.
- Provenance artifacts: artifact/proof manifests with hashes.
- Acceptance evidence artifacts: scenarios, generated acceptance tests, binding validation reports, test runner output, and mutation reports.
- Recursive improvement artifacts: bounded pass proposals with constitutional authority, scope, stop condition, risk, and proof.

## 5. Context Shaping

Decapod reduces wasted inference as a correctness property:

- clarify intent before spending model context
- assemble bounded context capsules
- avoid irrelevant repo sprawl
- stop for clarification when uncertainty is high
- validate output before completion claims

Token savings are a consequence of scoped governance, not a standalone product goal.

## 6. Validation and Promotion

Validation semantics:

- `decapod validate` is the repository health/proof gate.
- Failure means completion claims are invalid.

Promotion semantics:

- `decapod workspace publish` is the promote path.
- Publish MUST fail when required provenance manifests are missing.

## 7. Acceptance Proof Inputs

Acceptance-pipeline artifacts are evidence, not governing authority. Decapod may ingest or reference Gherkin features, scenario IR, generated tests, step-binding validation, runner output, and mutation reports as proof inputs attached to a task or workunit.

The control-plane authority stays with Decapod:

- intent is captured before acceptance evidence is interpreted
- boundaries decide which files, modules, and commands are in scope
- context shaping decides what the agent reads before inference
- proof plans decide which evidence is required for completion
- generated artifacts preserve what future agents can inspect

Current support is artifact-oriented: acceptance outputs can be captured as verification artifacts and file hashes. First-class acceptance proof gates belong behind a proof adapter that normalizes external reports into Decapod proof results without making Decapod a test runner.

## 8. Concurrent Agent Work

Concurrent work is coordinated through explicit task ownership, isolated worktrees, artifact-backed handoffs, validation, and proof before promotion.

Current architecture supports local-first coordination primitives. It must not claim distributed consensus, Raft, ZooKeeper-style coordination, or global locking semantics unless those mechanisms exist and have proof surfaces.

## 9. Acceptance Pipeline Lineage

Acceptance-pipeline thinking made completion criteria explicit before delivery. Decapod turns that intent into an agent-mediated governance path: pre-inference context shaping, boundary enforcement, artifact-backed coordination, validation, and proof-backed completion.

This complements human review. It does not make every human review obsolete; it makes agent-speed work inspectable before promotion.

Manual acceptance checklists remain useful, but they are not sufficient as the control layer for autonomous development. Decapod generalizes the loop by making acceptance evidence repo-native, agent-callable, replayable where possible, and subordinate to intent and proof policy.

## 10. Deterministic Execution Model

Determinism rules:

- Reducers and store updates are append-only/event-oriented.
- Envelopes are explicit, schemaed JSON.
- Golden vectors are used to detect protocol drift.
- Validation gates are executable and reproducible.
