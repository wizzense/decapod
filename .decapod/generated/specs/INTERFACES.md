# Interfaces

## Inbound Contracts
- **CLI Subcommand Surface**: Commands such as `init`, `todo`, `workspace`, `validate`, `rpc`, and `qa`.
- **JSON-RPC Schema**: Direct invocations mapping operation string and parameter payload (e.g. `todo.claim`, `specs.refresh`).

## Outbound Dependencies
| Dependency | Purpose | Minimum Version | Failure Behavior |
|---|---|---|---|
| Git CLI | Worktree creation, branch switching, status checking | 2.30+ | Block workspace command |
| Docker CLI | Container creation and execution | 20.10+ | Skip container step, log warning |

## Data Ownership
- All backplane data resides under `.decapod/data/`.
- SQLite database (`todo.db`) stores current tasks, claims, and event logs.
- Manifest file (`.manifest.json`) defines ownership hashes and fingerprints for living specs.

## Failure Semantics
Decapod uses typed Rust errors (`DecapodError` enum in `src/core/error.rs`) mapped to CLI status codes:
- `ValidationError`: Input parameters, file layouts, or metadata did not pass checks (exit code 1).
- `ConfigError`: Stale, malformed, or missing `.decapod/config.toml` (exit code 1).
- `NotFound`: The requested todo or file was not found (exit code 1).
- `IoError`: Underlying filesystem or subcommand failures (exit code 1).
