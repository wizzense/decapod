# Operations

## Service Level Objectives
- **Command latency**: 99% of local CLI invocations should execute and exit in less than 500ms.
- **Availability**: Decapod must successfully execute synchronous operations with zero persistent locks on active workspaces.

## Monitoring
- **Local Diagnostics log**: Command status and gate failures are logged in text or JSON format to standard output.
- **Trace logs**: Emitted to standard error under the `tracing` subscriber framework.

## Incident Response
- **Conflict mitigation**: In the event of SQLite lock contention, retry transaction blocks with random jitter backoff.
- **Sandbox cleanup**: Prune corrupted or abandoned workspaces using the `decapod workspace prune` command.
