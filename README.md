<p align="center">🦀</p>

<p align="center">
  <code>cargo install decapod && decapod init</code>
</p>

<p align="center">
  <strong>Decapod</strong><br />
  A local control layer your AI coding agent calls before and after it works.
</p>

<p align="center">
  Before work: Decapod turns intent into context, rules, specs, and todos.<br />
  After work: Decapod checks the result and records proof.
</p>

<p align="center">
  <a href="https://github.com/DecapodLabs/decapod/actions"><img alt="CI" src="https://github.com/DecapodLabs/decapod/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://crates.io/crates/decapod"><img alt="crates.io" src="https://img.shields.io/crates/v/decapod.svg"></a>
  <a href="LICENSE"><img alt="license" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
</p>

---

## Get running

```bash
cargo install decapod
decapod init
```

That's it.

`decapod init` asks about your project and creates `.decapod/`, a local folder your agent uses to remember intent, rules, context, specs, todos, and proof.

Your workflow does not change.

You keep talking to your agent like normal. The agent calls Decapod when it needs project memory, intent, boundaries, specs, todos, or proof.

Decapod runs, returns what the agent needs, writes explicit artifacts when useful, and exits.

---

## The simplest model

```text
You talk to the agent.
The agent talks to Decapod.
Decapod keeps the work bounded, remembered, specified, and provable.
```

Decapod is not another coding agent.

It is the repo-native governance layer agents call so their work can stay aligned with human intent, project constraints, and verifiable completion.

---

## Why Decapod exists

AI coding agents are fast, but speed alone does not make their work shippable.

Agents lose context.  
They infer intent too early.  
They mutate files without durable boundaries.  
They finish work without proving what changed.  
They operate concurrently without a shared control surface.

Decapod exists to give agent work a kernel:

- intent before execution
- context before inference
- boundaries before mutation
- todos before concurrency
- proof before completion

The result is not a smarter prompt.

The result is governed agent work.

---

## What Decapod gives your agent

Decapod gives coding agents a local project control surface.

Agents can use it to:

- understand project intent
- preserve repo-native memory
- generate and refine explicit specifications
- create todos and dependency-aware work plans
- enforce project boundaries
- coordinate concurrent work
- validate output
- record proof of completion

Humans mostly see the benefit.

The agent interacts with Decapod directly. Humans inspect the artifacts Decapod intentionally writes: specs, todos, context summaries, proof records, and other project state under `.decapod/`.

---

## What happens after init

After `decapod init`, your repository gets a local `.decapod/` directory.

That folder becomes the project’s governance substrate.

It can hold things like:

```text
.decapod/
  config/
  generated/
    specs/
  memory/
  proof/
  todos/
```

The exact contents evolve with your project and agent workflow.

The important part is simple:

```text
The proof is in the repo.
```

Decapod keeps agent-facing state close to the code, where it can be inspected, versioned, reviewed, and trusted.

---

## Agent-facing, human-benefiting

Decapod is designed to run in the background.

You should not need to manually operate Decapod during normal development.

The intended loop is:

```text
Human -> Agent -> Decapod -> Agent -> Model -> Agent -> Decapod -> Human
```

In practice:

1. You ask your agent to do work.
2. The agent calls Decapod to shape intent and context.
3. The agent performs the work.
4. The agent calls Decapod again to validate and record proof.
5. You receive a result with a clearer trail of what happened.

Decapod is the layer that turns “the agent said it was done” into “the work has an explicit record.”

---

## Why daemonless

Decapod is intentionally daemonless.

It does not require a long-running server.  
It does not require a hosted control plane.  
It does not require a background service watching your repo.

Agents call it when they need it.

Decapod runs, rehydrates local state, performs the requested operation, emits output, writes artifacts when needed, and exits.

That makes it:

- local-first
- repo-native
- low-overhead
- host-agnostic
- easy to call from different agents
- easy to reason about

It behaves less like a platform you log into and more like a project-native utility your agent can rely on.

---

## Designed for many agents

Decapod is agent-agnostic.

It is meant to support workflows across tools like:

- Claude Code
- Codex
- Kilo
- Cursor
- Aider
- custom agents
- local agents
- CI agents

The point is not to pick the winning agent.

The point is to give agents a shared governance substrate inside the repo.

Different agents can come and go.  
The repo keeps its memory, rules, specs, todos, and proof.

---

## Not a prompt pack

Decapod is not a collection of prompts.

Prompts are soft influence.

Decapod is explicit project state.

It exists to make agent work more durable by giving agents something concrete to read from, write to, and reason against.

A prompt can ask an agent to be careful.

Decapod gives the agent a place to record what careful means for this repository.

---

## Not an agent replacement

Decapod does not replace your coding agent.

It does not try to be the model, IDE, chat interface, or orchestration platform.

It sits underneath agent work as a governance kernel.

Your agent still writes code.  
Your agent still talks to the model.  
Your agent still interacts with you.

Decapod gives that loop memory, boundaries, specs, todos, and proof.

---

## Core thesis

Agent-speed development does not remove the need for governance.

It moves governance to the backhaul.

Instead of relying only on traditional PR friction, social review, and human memory, agentic development needs infrastructure that can shape work before inference and verify work after execution.

Decapod is that layer.

```text
Pre-inference shaping.
Boundary enforcement.
Output validation.
Proof-backed completion.
```

---

## Current status

Decapod is early.

The goal is to keep the kernel small, local, inspectable, and useful.

Anything that erodes auditability, proof, or boundaries is a regression, even if it feels productive.

The system is designed around one principle:

```text
Make agent work shippable by making intent explicit, boundaries explicit, and completion provable.
```

---

## Install

```bash
cargo install decapod
```

Then initialize Decapod inside your repository:

```bash
decapod init
```

Your agent can now use Decapod as part of its project workflow.

---

## Repository principle

Decapod should make the invisible parts of agent work explicit.

Not noisy.  
Not ceremonial.  
Not heavy.

Explicit enough that the repo can answer:

```text
What was the intent?
What context mattered?
What boundaries applied?
What work was planned?
What changed?
How was completion proven?
```

That is the kernel.

---

## License

MIT