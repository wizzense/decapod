<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  Governance kernel for AI coding agents.
</p>

<p align="center">
  Like Docker for containers. Like Rails for web.<br />
  But for agentic code: intent, context, boundary, proof.
</p>

<p align="center">
  <a href="https://github.com/DecapodLabs/decapod/actions"><img alt="CI" src="https://github.com/DecapodLabs/decapod/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://crates.io/crates/decapod"><img alt="crates.io" src="https://img.shields.io/crates/v/decapod.svg"></a>
  <a href="https://github.com/DecapodLabs/decapod/blob/master/LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <a href="https://ko-fi.com/decapodlabs"><img alt="Ko-fi" src="https://img.shields.io/badge/Support-Ko--fi-ff5f5f?logo=ko-fi&logoColor=white"></a>
</p>

---

## The problem

Modern AI agents are ungoverned systems:

1. **Intent drift** — User says X, agent outputs Y.
2. **Context explosion** — Full codebase in prompt. Tokens burn.
3. **Proof-free completion** — "Looks good" = done.
4. **Boundary violation** — Agent writes to protected branches.

No controller. No contract. No proof.

---

## What Decapod is

The control plane your agent **defers to**:

- Before action:   "what's the intent?"
- Before inference: "what's relevant?"
- Before commit:  "did this pass?"

Not another tool. Not a prompt. Not a daemon. Just the layer that holds the agent accountable.

### The governance loop

```
        ┌──────────┐        ┌──────────┐
        │  User   │◄───────│  Agent   │
        └────┬────┘        └────┬────┘
             │                 │
             │      ┌─────────▼────────┐
             │      │    Decapod     │◄── Intent?
             │      │   (govern)   │    Boundary?
             │      │             │    Context?
             │      │             │    Proof?
             │      └─────────▲────────┘
             │              │
      ┌───────▼──────┐     │    ┌──────────┐
      │   Agent     │─────┴────│  Model  │
      └────────────┘          └──────────┘
             │                   │
             └─────────┬─────────┘
                       ▼
                     User
```

Every action: check intent → resolve context → enforce boundary → verify proof.

---

## Why this exists

Every generation has its defining infrastructure:

- **Docker** made containers the unit of deployment.
- **Rails** made web productive.
- **Node** made JavaScript universal.
- **Claude/Claude Code** made agentic coding possible.

But nobody built the **governance layer** for agents. The layer that says:

- "Stop. What's the intent?"
- "Wait. What's in scope?"
- "Hold on. Did you prove it?"

That's Decapod. The kernel your agent checkpoints with.

### How it hijacks your project

Decapod mounts a **constitution** into your repo:

```text
.decapod/
  constitution/              # governance rules
    core/DECAPOD.md         # core contract
    interfaces/             # interfaces
    plugins/                # plugin policies
  OVERRIDE.md               # your project overrides
```

This **overrides** your agent's instruction files:

- `AGENTS.md` → becomes the universal contract
- `CLAUDE.md` → augmented with context
- `CODEX.md`, `GEMINI.md` → same pattern

Your agent doesn't load 50-page specs every session. Decapod resolves only what's relevant, binds it to a session lease, and enforces that the agent use it — not the raw file again.

**Your project, your rules, your agent.**

---

## What you get

- **Daemonless** — runs on-demand, exits immediately.
- **Two commands** — install + init. Done.
- **Agent-agnostic** — Claude, Codex, Gemini, Cursor, any shell-out.
- **Parallel-safe** — multiple agents, one repo, no collisions.
- **Proof-gated** — `VERIFIED` means gates passed.
- **Auditable** — every decision in `.decapod/` as plain files.
- **Constitutional** — your overrides take precedence.

---

## Before / After

### Before

```
User:  "fix the login bug"
Agent: [proceeds]
       → burns tokens on full codebase
       → generates code
       → "looks good"
       → commits
```

### After

```
User:  "fix the login bug"
Agent: [checks with Decapod]
       → intent: clarify login flow?
       → asks user
       → generates fix
Agent: [checks with Decapod]
       → proof: generated
       → commits with artifact
```

The difference: **intent → context → code → proof**.

---

## Running

```bash
cargo install decapod
decapod init
```

Your workflow doesn't change. The agent calls Decapod. You just use your agent as normal.

---

## Guarantees

Decapod enforces:

1. **Daemonless** — no background process. [tests/daemonless_lifecycle.rs](tests/daemonless_lifecycle.rs)
2. **Repo-native** — state in `.decapod/`, not external. [src/core/store.rs](src/core/store.rs)
3. **Proof-gated** — `VERIFIED` only when gates pass. [tests/workunit_publish_gate.rs](tests/workunit_publish_gate.rs)
4. **Workspace isolation** — no direct protected branch writes. [tests/workspace_interlock.rs](tests/workspace_interlock.rs)
5. **Bounded validation** — no infinite hangs. [tests/validate_termination.rs](tests/validate_termination.rs)
6. **Store boundary** — agents use CLI, not direct mutation.

---

## Contributing

```bash
git clone https://github.com/DecapodLabs/decapod
cd decapod
cargo build
cargo test
```

---

## Docs

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)
- [CHANGELOG.md](CHANGELOG.md)

---

## Support

- [Issues](https://github.com/DecapodLabs/decapod/issues)
- [Ko-fi](https://ko-fi.com/decapodlabs)