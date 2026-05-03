<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  Daemonless, repo-native governance kernel for AI coding agents.
</p>

<p align="center">
  Agents call it on demand to converge on human intent, shape context before inference,<br />
  enforce boundaries, and deliver proof-backed completion across concurrent multi-agent work.
</p>

<p align="center">
  <a href="https://github.com/DecapodLabs/decapod/actions"><img alt="CI" src="https://github.com/DecapodLabs/decapod/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://crates.io/crates/decapod"><img alt="crates.io" src="https://img.shields.io/crates/v/decapod.svg"></a>
  <a href="https://github.com/DecapodLabs/decapod/blob/master/LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

---

## Get running

```bash
cargo install decapod
decapod init
```

That's it. Your workflow doesn't change. Your agent calls Decapod before:

- Acting — intent
- Calling the model — context  
- Committing — proof
- Touching protected code — boundaries

Decapod is designed to stay out of the human workflow. The agent checks in. You keep talking to your agent like normal.

> AI agents do not fail because they lack tools. They fail because they lose intent, skip dependencies, mutate context unsafely, and return vibes instead of proof.

### The loop

```
     User
       │
       ▼
    Agent ───────┐
       │         │
       │    ┌────▼────┐
       │    │ Decapod │
       │    │ (check) │
       │    └────┬────┘
       │         │
       ├─────────┤
       │         │
     Model     Agent
       │         │
       └────┬────┘
            ▼
          User
```

---

## What Decapod does

1. **Clarifies intent** — What's the goal?
2. **Bounds context** — Only what's needed. Not the whole repo.
3. **Enforces proof** — VERIFIED means gates passed.
4. **Protects boundaries** — No direct writes to master.

Decapod resolves only what's relevant to the user's intent — no context poisoning. Your agent gets surgical context, not the entire constitution.

---

## Agent workbenches improve the session. Decapod improves the shared substrate.

- **Agents act in private context; Decapod makes their work public to the repo.**
- A task started by one agent provider should be understandable, auditable, or resumable by another. The source of truth lives in `.decapod/`, not in chat history, IDE state, or provider memory.
- The durable parts of agentic work—intent, resolved context, boundaries, todos, specs, validation, proof artifacts—become repo-native operational knowledge.

Decapod absorbs agent deficiencies: ambiguity, context waste, boundary drift, forgotten dependencies, unsafe mutation, and unverifiable "done."

### The shared substrate

```
decapod/
  generated/
    specs/         # INTENT.md, ARCHITECTURE.md, etc.
    context/       # deterministic context capsules
    artifacts/     # proof artifacts, provenance
  governance/      # todos, claims, workunits
  data/            # durable state
```

This is what persists. Not the chat transcript.

### The constitution

Decapod ships with an embedded engineering constitution.

Over 100 industry-grade declarative documents covering architecture, security, performance, testing, knowledge graphs, claims, proof surfaces, interfaces, evaluation criteria, and workflows. Everything an engineering org usually keeps in scattered docs, tribal memory, and review culture becomes executable guidance your agent can consult.

> Recent research has confirmed what Decapod was built around from the start: AI coding agents waste significant context on irrelevant files. — [arXiv:2602.11988](https://arxiv.org/pdf/2602.11988)

Your agent doesn't guess. It reads the constitution. It cites claim IDs. It follows gates. It asks for clarification. It produces proof.

### Your interface

Override the embedded constitution with `.decapod/OVERRIDE.md`. Plain English rules that take precedence:

```text
.decapod/
  OVERRIDE.md    # your rules, overrides embedded defaults
```

Your overrides augment the constitution automatically.

---

## Proof lives in the repo

Every run leaves its operational evidence in `.decapod/`:

- captured intent → `generated/specs/INTENT.md`
- resolved context → `generated/context/`
- todos and dependencies → `governance/todos.jsonl`
- verification results → `generated/artifacts/`
- proof artifacts → `generated/artifacts/provenance/`

That directory is the proof surface. It can be inspected locally, reviewed in pull requests, archived with the codebase, and used by the next agent invocation to re-establish state.

**The repo remembers. Chat history doesn't.**

---

## Agent Workbench Gaps

| What workbenches optimize | What Decapod preserves |
|-------------------------|------------------------|
| The current session | Reusable repo-native knowledge |
| Worker throughput | Shared substrate quality |
| Provider-specific context | Explicit intent, boundaries, proof |
| Session-scoped memory | `.decapod/` durable state |

**Multi-provider continuity**: A task started by Claude Code should be auditable by Codex, resumable by Gemini CLI, and verifiable by Kilo. The source of truth is `.decapod/`, not chat history, IDE state, or provider memory.

---

## Integrations

Decapod works with Claude Code, Codex, Gemini CLI, Cursor, Kilo, and any shell-capable agent through simple entrypoints.

Each entrypoint calls Decapod at key moments: before acting (intent), before inference (context), before committing (proof), before touching protected code (boundaries).

See [AGENTS.md](AGENTS.md) for the universal agent contract.

---

## What you get

- No daemon.
- No SaaS control plane.
- No hidden agent memory.
- Full operational state stored locally in `.decapod/`.
- Proof your team can inspect, diff, review, and commit.

---

## Before / After

### Before

```
User: "build auth"
Agent: [full repo in prompt]
       → generates
       → commits
```

### After

```
User: "build auth"
Agent: [Decapod]
       → intent: auth system
       → context: src/auth/
       → generates
       → [Decapod]
       → proof: verified
       → commits
```

---

## Running

```bash
cargo install decapod
decapod init
```

Use whatever agent you already use: Claude, Codex, Gemini, Cursor.

---

## Guarantees

- **Daemonless** — runs on-demand
- **Repo-native** — state in `.decapod/`
- **Proof-gated** — VERIFIED means gates pass
- **Boundaries enforced** — protected branches locked

---

## Contributing

```bash
git clone https://github.com/DecapodLabs/decapod
cd decapod
cargo build && cargo test
```

---

## Docs

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)

---

## Support

- [Issues](https://github.com/DecapodLabs/decapod/issues)
