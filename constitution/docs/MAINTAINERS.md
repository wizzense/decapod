# Maintainers

## Links

- [core/DECAPOD.md](../../core/DECAPOD.md) - **Router and navigation charter (START HERE)**
- [specs/INTENT.md](../../specs/INTENT.md) - Methodology contract
- [specs/AMENDMENTS.md](../../specs/AMENDMENTS.md) - Change control

## Maintainer Contract

Maintainers MUST enforce:

- daemonless architecture
- repo-native canonical promotion state
- deterministic reducers/envelopes
- explicit schema and proof gates

## PR Acceptance Rules

A PR touching invariants MUST include:

- intent declaration
- invariants affected
- proof/gate added or updated

"No vibes PRs": claims without enforcement are rejectable.

## Versioning Authority

Maintainers MUST apply SemVer discipline:

- schema change => version bump
- CLI/RPC breaking change => major bump
