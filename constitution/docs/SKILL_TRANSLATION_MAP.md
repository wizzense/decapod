# SKILL_TRANSLATION_MAP.md

## Links

- [core/DECAPOD.md](../../core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [specs/skills/SKILL_GOVERNANCE.md](../../specs/skills/SKILL_GOVERNANCE.md) - Skill governance contract
- [plugins/APTITUDE.md](../../plugins/APTITUDE.md) - Aptitude subsystem

## Decapod Translation Map (Skills)

- Skill package (`SKILL.md` + scripts) -> `SKILL_CARD` artifact at `<repo>/.decapod/governance/skills/*` with source digest + normalized workflow outline.
- Agent choosing a skill ad hoc -> `SKILL_RESOLUTION` artifact at `<repo>/.decapod/generated/skills/*` with deterministic ranking and hash.
- Marketplace metadata -> non-authoritative input only; canonical authority stays repo-native.
- Human preference for workflows -> aptitude skill/preference entries in Decapod store.
- Skill drift -> `decapod validate` artifact-hash mismatch failure.

## Why this is kernel-viable

- Stateless CLI invocation
- Deterministic serialization + hashing
- Multi-agent shared substrate
- No provider coupling
- No long-running coordinator
