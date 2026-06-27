# Security

## Threat Model
```mermaid
flowchart TD
    Agent[Agent Client] --> |Session Token / Password| Decapod[Decapod CLI]
    Decapod --> |Local File Access| DB[(SQLite Database)]
    Decapod --> |Subprocess execution| Sandbox[Container Sandbox]
    Sandbox --> |Access Restricted| Workspace[Git Worktree]
```

## Authentication
- Access validation requires a valid session token corresponding to `DECAPOD_SESSION_PASSWORD`.

## Authorization
- Commands checking or mutating workspaces or todos check the active session to authorize execution.

## Data Classification
| Data Class | Example | Protection |
|---|---|---|
| Credentials | Session passwords | Stored in memory / secure OS variables |
| Backlog State | Todos, presence logs | SQLite database files in `.decapod/` |
| Working Files | Workspace directories | Git worktree directories, branch namespaces |
