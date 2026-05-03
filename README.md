<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  Governance kernel for AI coding agents.
</p>

<p align="center">
  Install. Init. Use your agent as normal.<br />
  That's the entire prescription.
</p>

<p align="center">
  <a href="https://github.com/DecapodLabs/decapod/actions"><img alt="CI" src="https://github.com/DecapodLabs/decapod/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://crates.io/crates/decapod"><img alt="crates.io" src="https://img.shields.io/crates/v/decapod.svg"></a>
  <a href="https://github.com/DecapodLabs/decapod/blob/master/LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

---

## The prescription

```bash
cargo install decapod
decapod init
```

That's it. Walk away. Use your agent exactly as before.

Decapod runs silently. Your agent checks in before:

- Acting — intent
- Calling the model — context  
- Committing — proof
- Touching protected code — boundaries

You never see it. Your agent does all the work.

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
       ├────────┤
       │         │
    Model    Agent
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

Decapoid resolves only what's relevant to the user's intent — no context poisoning. Your agent gets surgical context, not the entire codebase.

### The constitution (embedded)

Your agent carries the entire software industry in its head.

94 documents. Architecture. Security. Performance. Testing. Knowledge graphs. Claims. Proof surfaces. Interfaces. Evaluation criteria. Workflows. Everything you've spent decades building into your engineering org — now code-executable.

> Decapod was built months before ETH Zurich proved that AI coding agents waste context on irrelevant files. We've been doing this since before it was a research paper. — [arXiv:2602.11988](https://arxiv.org/pdf/2602.11988)

Your agent doesn't guess. It reads the constitution. It cites claim IDs. It follows enforced gates. It produces proof.

You just talk to your agent.

---

## What you get

- No config. No daemon. No workflow.
- Install. Init. Done.
- Production-grade code.
- Full state in `.decapod/`.

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

Use whatever agent. Claude. Codex. Gemini. Cursor.

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