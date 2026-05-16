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

1. repository constitution
2. project/spec intent
3. task boundaries and ownership
4. proof requirements
5. generated artifacts
6. agent-local execution

Agents may propose improvements at higher layers, but promotion requires explicit artifacts and proof. Agent-local execution cannot silently rewrite higher-level intent or bypass proof requirements.

## 4. Artifact Model

Core artifacts:

- Intent artifacts: `INTENT.md`, `SPEC.md`, ADRs.
- Claims artifacts: interface claims and proof obligations.
- Proof artifacts: validation reports, state-commit records, verification outputs.
- Provenance artifacts: artifact/proof manifests with hashes.

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

## 7. Concurrent Agent Work

Concurrent work is coordinated through explicit task ownership, isolated worktrees, artifact-backed handoffs, validation, and proof before promotion.

Current architecture supports local-first coordination primitives. It must not claim distributed consensus, Raft, ZooKeeper-style coordination, or global locking semantics unless those mechanisms exist and have proof surfaces.

## 8. Acceptance Pipeline Lineage

Acceptance-pipeline thinking made completion criteria explicit before delivery. Decapod turns that intent into an agent-mediated governance path: pre-inference context shaping, boundary enforcement, artifact-backed coordination, validation, and proof-backed completion.

This complements human review. It does not make every human review obsolete; it makes agent-speed work inspectable before promotion.

## 9. Deterministic Execution Model

Determinism rules:

- Reducers and store updates are append-only/event-oriented.
- Envelopes are explicit, schemaed JSON.
- Golden vectors are used to detect protocol drift.
- Validation gates are executable and reproducible.
