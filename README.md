<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  The governance kernel for AI coding agents.
</p>

<p align="center">
  The agent defers to Decapod. Your code stays aligned.<br />
  Intent, boundaries, and proof — enforced, not hoped for.
</p>

<p align="center">
  <a href="https://github.com/DecapodLabs/decapod/actions"><img alt="CI" src="https://github.com/DecapodLabs/decapod/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://crates.io/crates/decapod"><img alt="crates.io" src="https://img.shields.io/crates/v/decapod.svg"></a>
  <a href="https://github.com/DecapodLabs/decapod/blob/master/LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <a href="https://ko-fi.com/decapodlabs"><img alt="Ko-fi" src="https://img.shields.io/badge/Support-Ko--fi-ff5f5f?logo=ko-fi&logoColor=white"></a>
</p>

---

## The problem

AI agents are powerful but unchecked:

1. **Intent drift** — Agent starts coding. Human asked for X. Agent delivers Y.
2. **Dependencies skipped** — Agent assumes "just do it" includes all prerequisites.
3. **Context bloat** — Agent dumps the whole repo in the prompt. Tokens burn.
4. **Vibes as proof** — "Looks good" is treated as done.
5. **Unsafe mutation** — Agent touches protected branches. Validation gets skipped.

The agent doesn't know it's drifting. There's no governance layer telling it to stop.

---

## What Decapod is

Decapod is the layer your agent **defers to** — not an alternative, not another tool, just the source of truth:

- The agent asks before it acts: "what's in scope?"
- The agent asks before inference: "what context is relevant?"
- The agent asks before it commits: "is this done?"

**Decapod answers. The agent proceeds.**

### The governance loop

```
         ┌──────────────┐         ┌──────────────┐
         │    User      │◄────────│    Agent     │
         └──────┬───────┘         └──────┬───────┘
                │                      │
                │         ┌─────────▼────────┐
                │         │    Decapod       │◄──── Intent?
                │         │    (govern)      │     Boundary?
                │         │                 │     Context?
                │         │                 │     Proof?
                │         └─────────▲────────┘
                │                   │
         ┌──────▼───────┐         │        ┌──────────────┐
         │    Agent     │─────────┴────────│    Model    │
         └──────────────┘                  └──────────────┘
```

Every non-trivial action:

- **Where are we headed?** → *intent resolved*
- **What territory?** → *context resolved*  
- **Off-limits?** → *boundaries enforced*
- **Arrived?** → *proof verified*

The agent checkpoints with Decapod as many times as needed — iterates on intent, requests clarification, resolves context — until aligned. Then executes.

---

## Why it exists

Your agent writes code. Nobody checks its homework.

Decapod forces agents to answer three questions before touching a single line:

1. **What did the human actually ask for?** — Intent crystallization, not vibes.
2. **What boundaries apply?** — Protected branches, validation gates, policy constraints.
3. **How will we prove it's done?** — Artifact-backed completion, not narrative claims.

The answers become cryptographically verifiable artifacts. The code that lands in your repo is provably what was intended.

### When it clicks

**"The spec was vibes."** Decapod forces intent to crystallize — constraints, boundaries, acceptance criteria — before a single line is generated.

**"Multiple agents, one repo, total chaos."** Decapod coordinates shared state across parallel runs. No silent overwrites. Each agent gets an isolated workspace.

**"It passes CI but is it *done*?"** Decapod gates completion on `VERIFIED` — every proof gate actually passed, not "the agent said so."

---

## Get running

```bash
cargo install decapod
decapod init
```

That's it. Keep using Claude Code, Codex, Gemini CLI, Cursor — whatever you already use. Decapod gets called by your agent automatically.

### What lands in your repo

```text
.decapod/
  config.toml                 # project configuration
  data/                       # durable state
  generated/
    specs/                   # intent, architecture, validation specs
    artifacts/                # proof artifacts, provenance
  sessions/                   # per-session logs
AGENTS.md                    # universal agent contract
```

### How to know it's working

1. Ask your agent to make a change. Specs appear in `.decapod/generated/`.
2. Ask your agent to validate. Typed pass/fail gates, not "looks good."
3. Ask "what changed and why?" — Agent cites spec and proof artifacts.
3. Ask the agent *"what did Decapod change about your plan?"* — it should cite spec and proof steps, not vibes.

Agent integration: `AGENTS.md` and tool-specific entrypoints (`CLAUDE.md`, `CODEX.md`, `GEMINI.md`) define the full operational contract your agent follows.

Override any constitution default with plain English in `.decapod/OVERRIDE.md`. Learn more about the embedded [constitution](constitution/core/DECAPOD.md).

---

## Why this exists

Coding agents suck. But it's not their fault.

You can't solve the world inside the agent. Like any serious technology, agents need infrastructure — a way to interface with the host machine (files, repos, terminals, policies) in a way that's intelligent, bounded, and provable.

The Unix philosophy ("do one thing well") breaks down the moment the "one thing" becomes: reason over ambiguous intent, plan work, write code, validate it, manage state, coordinate tools, and ship safely. We expect agents to generate great code. They mostly can. But the gaps aren't something you patch by making the agent fatter. The gaps exist because the agent isn't the right place for control-plane responsibilities.

Right now, agent makers keep stuffing more into the agent: task management, memory, rules, planning, codegen, toolchains, browsers — until it's mediocre at everything. Agents shouldn't be responsible for control-plane work. They shouldn't be your TODO database. They shouldn't be the place you encode a team's behavioral expectations. They shouldn't be the system of record for "what got done" or "what's allowed." That belongs in infrastructure.

Decapod is a repo-native governance kernel that agents call into — like a device driver for agent work. It makes intent explicit, boundaries explicit, and completion provable.

State is local and durable in `.decapod/`. Context, decisions, and traces persist across sessions.

It's not a tool the agent uses — it's the layer the agent checks with.

---

## Inference Governance

Decapod can govern the inference boundary between agent and model. The agent calls Decapod before inference to shape what's admissible, then after to validate what was generated.

### The boundary loop

```
User → Agent → Decapod (infer init) → Model → Decapod (infer validate) → Agent → User
```

**Before inference**, Decapod returns:
- `selected_context` — what's relevant to include
- `excluded_context` — what to leave out  
- `clarification_required` — whether to ask first
- `token_budget` — estimated context size
- `proof_required` — what completion looks like

**After inference**, Decapod validates against intent and proof expectations.

### Why it matters

Token waste is the visible symptom of a deeper governance failure: the agent doesn't know what's legitimately in scope for the task. Decapod draws that boundary before the model sees anything.

This isn't token optimization. It's ruling out irrelevant context *before* it inflates the prompt.

### Example

```bash
# Before calling the model
decapod infer init --intent "fix login bug" --context "src/auth/"
# Returns: {"selected_context": ["src/auth/login.rs"], "excluded_context": ["docs/"], "clarification_required": false}

# After the model responds
decapod infer validate --result "$MODEL_OUTPUT" --intent "fix login bug"  
# Returns: {"intent_match": true, "proof_provided": false, "advisory": "No proof artifact"}
```

## How it works

Every Decapod operation returns some combination of three signals.

- **Advisory** narrows the problem. It helps the agent stop wasting cycles on vague intent, missing context, or weak plans.
- **Interlock** blocks unsafe flow. It is the hard stop that says, "you are about to break policy, violate a boundary, or skip proof."
- **Attestation** is the receipt. It records what actually happened and whether the required criteria were met.

```text
Human Intent
    |
    v
AI Agent(s)  <---->  Decapod  <---->  Repository + Policy
                       |  |  |
                       |  |  +-- Interlock (enforced boundaries)
                       |  +----- Advisory (guided execution)
                       +-------- Attestation (verifiable outcomes)
```

## What you get

- **Daemonless.** No background process. The binary starts, does its job, exits.
- **Two-command install.** Install and init. Done.
- **Agent-agnostic.** Works with Claude, Codex, Gemini, Cursor, and anything else that can shell out.
- **Parallel-safe.** Multiple agents, one repo, no collisions.
- **Proof-gated completion.** `VERIFIED` requires passing proof-plan results, not narrative.
- **Fully auditable.** Every decision, trace, and proof artifact lives in `.decapod/` as plain files.
- **Context internalization.** Turn long documents into mountable, verifiable context adapters with explicit source hashes, determinism labels, session-scoped attach leases, and explicit detach so agents stop re-ingesting the same 50-page spec every session.

The deep surface area — interfaces, capsules, eval kernel, knowledge promotions, obligation graphs — lives in the embedded constitution.

---

## Before / After

### Before Decapod

```
User: "fix the login bug"
Agent: [proceeds without clarification]
  → burns tokens on entire codebase
  → generates code based on vibes
  → "looks good"
  → commits
```

### After Decapod

```
User: "fix the login bug"
Agent: decapod infer init --intent "fix login bug" --context src/auth/
  → Decapod: {"selected_context": ["src/auth/login.rs"], "clarification_required": true}
Agent: asks user: "which login flow — password reset or OAuth?"
User: "password reset"
Agent: [generates fix]
Agent: decapod infer validate --result "$CODE" --intent "fix login bug"
  → Decapod: {"intent_match": true, "proof_provided": true}
  → commits with proof artifact
```

The difference: **intent** → **context** → **code** → **proof**.

---

## What Decapod Guarantees

These are the things Decapod **actually enforces** — break any of these and `decapod validate` will fail:

Decapod stays daemonless. There is no background service to keep alive. It runs on demand and exits when the call is done. That behavior is enforced by [tests/daemonless_lifecycle.rs](tests/daemonless_lifecycle.rs).

Decapod stays repo-native. State lives in `.decapod/` as plain files and local data, rather than an external control-plane service. The storage model is implemented in [src/core/store.rs](src/core/store.rs).

Completion is proof-gated. `VERIFIED` is not a vibe-based status; it only exists when the proof plan passes. That path is enforced by the WorkUnit status machinery and [tests/workunit_publish_gate.rs](tests/workunit_publish_gate.rs).

Workspace isolation is mandatory. Agents cannot mutate protected branches directly and are expected to work in isolated worktrees. That is enforced by the workspace interlock and [tests/workspace_interlock.rs](tests/workspace_interlock.rs).

Validation is bounded. `decapod validate` must terminate in finite time instead of hanging indefinitely. That guarantee is enforced by [tests/validate_termination.rs](tests/validate_termination.rs) and the timeout logic around validation gates.

The store boundary is real. Agents are expected to use Decapod command surfaces instead of mutating `.decapod/*` directly. That is enforced by validation gates and the broker layer.

Mutations require a session. State-changing operations need active credentials, and session checks are part of the mutation path.

These are **aspirational** (we're working on them):

- Parallel-safe multi-agent coordination (partially enforced via workspace isolation)
- Context capsule deterministic output (partially enforced)

See `.decapod/contracts/README_CONTRACTS.json` for the full contract map and enforcement links.

---

## Contributing

```bash
git clone https://github.com/DecapodLabs/decapod
cd decapod
cargo build
cargo test
```

## Docs

- [CONTRIBUTING.md](CONTRIBUTING.md) — development guide
- [SECURITY.md](SECURITY.md) — security policy
- [CHANGELOG.md](CHANGELOG.md) — release history

## Support

- [Issues](https://github.com/DecapodLabs/decapod/issues)
- [Ko-fi](https://ko-fi.com/decapodlabs)

## License

MIT. See [LICENSE](LICENSE).
