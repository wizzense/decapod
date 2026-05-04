<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  Daemonless, repo-native governance kernel for AI coding agents.
</p>

<p align="center">
  Agents call Decapod on demand to turn intent into context, then context into explicit specifications before inference,<br />
  and finished work into proof.
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

That's it. `decapod init` asks about your project, scaffolds `.decapod/`, and gives your agent a repo-native control surface for governed work.

Your workflow does not change. You keep talking to your agent like normal. The agent checks in with Decapod before:

- **Acting** — clarify intent
- **Calling the model** — resolve context
- **Touching protected code** — enforce boundaries
- **Committing** — produce proof

See the canonical router in [constitution/core/DECAPOD.md](constitution/core/DECAPOD.md).

> AI agents do not fail because they lack tools. They fail because they lose intent, skip dependencies, mutate context unsafely, and return vibes instead of proof.

---

## The loop

```text
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

Decapod is not the agent. Decapod is the governance kernel the agent calls when intent, context, boundaries, dependencies, or proof need to become explicit.

---

## What Decapod does

1. **Clarifies intent** — what is the goal?
2. **Bounds context** — what does the agent actually need?
3. **Generates specs** — what should be built, changed, or preserved?
4. **Tracks dependencies** — what must happen first?
5. **Enforces boundaries** — what must not be touched?
6. **Requires proof** — what makes the work complete?

Decapod resolves only what is relevant to the user's intent. Your agent gets surgical context, not the whole repo and not the entire constitution.

---

## Why it exists

Agent workbenches improve the session.

Decapod improves the shared substrate.

Agents act in private context. Decapod makes the durable parts of their work public to the repo: intent, resolved context, boundaries, todos, specs, validation, and proof artifacts.

A task started by Claude Code should be auditable by Codex, resumable by Gemini CLI, and verifiable by Kilo. The source of truth lives in `.decapod/`, not chat history, IDE state, or provider memory.

Decapod absorbs agent deficiencies:

- ambiguity
- context waste
- boundary drift
- forgotten dependencies
- unsafe mutation
- unverifiable "done"

---

## The shared substrate

```text
.decapod/
  generated/
    specs/         # INTENT.md, ARCHITECTURE.md, etc.
    context/       # deterministic context capsules
    artifacts/     # verification output, proof, provenance
  governance/      # todos, claims, workunits
  data/            # durable repo-native state
  config.toml      # project control plane config
  OVERRIDE.md      # local rules that override embedded defaults
```

This is what persists.

Not the chat transcript.

---

## The constitution

Decapod ships with an embedded engineering constitution: over 100 declarative documents covering architecture, security, performance, testing, knowledge graphs, claims, proof surfaces, interfaces, evaluation criteria, and workflows.

Everything an engineering org usually keeps in scattered docs, tribal memory, and review culture becomes executable guidance your agent can consult.

> Recent research has confirmed what Decapod was built around from the start: AI coding agents waste significant context on irrelevant files. — [arXiv:2602.11988](https://arxiv.org/pdf/2602.11988)

Your agent does not guess. It reads the constitution, cites claim IDs, follows gates, asks for clarification, and produces proof.

---

## Your interface

Use `.decapod/OVERRIDE.md` for plain-English rules that override the embedded constitution:

```text
.decapod/
  OVERRIDE.md    # your rules, overriding embedded defaults
```

Use `.decapod/config.toml` for project-level configuration:

```toml
name = "my-project"
summary = "What this repo does"
primary_languages = ["rust", "typescript"]
architecture = "cli"
```

`config.toml` captures project context and setup preferences:

- project name and summary
- primary language or languages
- architecture type
- generated agent entrypoints such as `CLAUDE.md`, `GEMINI.md`, and others

Keep durable rules in `OVERRIDE.md`. Keep project shape in `config.toml`.

---

## Proof lives in the repo

Every run leaves operational evidence in `.decapod/`:

- captured intent → `generated/specs/INTENT.md`
- resolved context → `generated/context/`
- todos and dependencies → `governance/todos.jsonl`
- verification results → `generated/artifacts/`
- proof artifacts → `generated/artifacts/provenance/`

That directory is the proof surface. It can be inspected locally, reviewed in pull requests, archived with the codebase, and used by the next agent invocation to re-establish state.

**The repo remembers. Chat history does not.**

---

## Agent workbench gaps

| Workbenches optimize | Decapod preserves |
|---|---|
| The current session | Reusable repo-native knowledge |
| Worker throughput | Shared substrate quality |
| Provider-specific context | Explicit intent, boundaries, and proof |
| Session-scoped memory | Durable state in `.decapod/` |

Use whatever agent you already use: Claude, Codex, Gemini, Cursor, Kilo.

---

## Before / after

### Before

```text
User: "build auth"
Agent: [full repo in prompt]
       → generates
       → commits
```

### After

```text
User: "build auth"
Agent: [Decapod]
       → intent: auth system
       → context: src/auth/
       → specs generated
       → work completed
       → [Decapod]
       → proof: verified
       → commits
```

---

## Guarantees

- **Daemonless** — runs on demand
- **Repo-native** — state lives in `.decapod/`
- **Local-first** — no SaaS control plane required
- **Provider-agnostic** — works across agent workbenches
- **Proof-gated** — VERIFIED means gates passed
- **Boundary-aware** — protected paths and branches are enforced

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
