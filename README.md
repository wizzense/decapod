<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  Daemonless, local-first governance kernel behind AI coding agents.
</p>

<p align="center">
  Decapod is the daemonless, local-first governance kernel behind AI coding agents. Agents call it on demand to converge on human intent, shape context before inference, enforce boundaries, and deliver proof-backed completion across concurrent multi-agent work.
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

`decapod init` creates `.decapod/`, a local folder your agent uses to remember intent, rules, context, specs, and proof.

Your **conversational** workflow does not change. You keep talking to your agent normally; behind the scenes, the agent calls Decapod to ensure work happens in isolated environments and adheres to your project's constitution.

---

## How it works

AI coding agents often lose the plot: they forget intent, pull too much context, skip dependencies, and touch protected files. Decapod provides a repo-native checkpoint system to absorb these deficiencies.

### The Loop

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

Decapod is not the agent. It is the **governance kernel** called before:
- **Acting** — clarify intent and generate specs
- **Inference** — resolve surgical context capsules
- **Touching Code** — enforce boundaries and protected paths
- **Completing** — produce verification and proof

---

## Capabilities

1. **Clarifies intent** — Converts vague requests into explicit, versioned specifications.
2. **Bounds context** — Resolves only the minimal relevant code/docs for the task.
3. **Enforces boundaries** — Safeguards protected branches and sensitive modules.
4. **Governs adaptation** — Manages feedback-driven instruction changes through explicit review.
5. **Requires proof** — Gates completion on deterministic verification artifacts.

---

## The substrate

Decapod preserves what agent workbenches lose: reusable, repo-native knowledge that survives the session.

```text
.decapod/
  generated/
    specs/         # Human-visible intent and architecture specs
    context/       # Deterministic context capsules
    artifacts/     # Verification output and proof provenance
  data/            # Durable repo-native state (DBs, events, todos)
  config.toml      # Project shape and agent-facing configuration
  OVERRIDE.md      # Local rules that override embedded defaults
```

Every run leaves operational evidence. The generated files are the human-visible proof surface—inspect them locally, review them in PRs, and use them to re-establish state across different agents (Claude, Codex, Gemini, Cursor).

---

## The constitution

Decapod ships with an embedded engineering constitution: over 100 declarative documents covering architecture, security, performance, and testing.

Everything an engineering org usually keeps in tribal memory or review culture becomes executable guidance. Your agent does not guess; it reads the constitution, cites claim IDs, follows gates, and produces proof.

---

## Guarantees

- **Daemonless** — Runs on demand like `git` or `grep`.
- **Repo-native** — All state lives in your repository.
- **Provider-agnostic** — Works across all agent workbenches.
- **Proof-gated** — Completion requires passed verification gates.
- **Boundary-aware** — Enforces protected paths and branch isolation.

---

## Contributing

```bash
git clone https://github.com/DecapodLabs/decapod
cd decapod
cargo build && cargo test
```

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)
- [Issues](https://github.com/DecapodLabs/decapod/issues)
