---
layout: default
title: Decapod Book
nav_order: 1
---

# Decapod Documentation

Welcome to the official documentation for **Decapod**—the daemonless, local-first, repo-native governance kernel for AI coding agents.

This documentation serves as the comprehensive guide (the **Decapod Book**) for both humans designing repository boundaries and agents navigating them.

---

## 🚀 Getting Started

*   **[Introduction](book/src/introduction.md)**: What is Decapod? Learn about the governance gap and the core pillars of repo-native agent control.
*   **[Quickstart](book/src/quickstart.md)**: Install, initialize, and run your first agent handshake and validation in under five minutes.
*   **[Mental Model](book/src/mental-model.md)**: Understand how agents, tasks, sessions, and workspaces interact.
*   **[Configuration](book/src/configuration.md)**: Structure your repository config, enable cloud backends, and configure containerization.

---

## 🔄 Governed Workflows

Learn how agents move through the lifecycle of planning, execution, validation, and completion.

*   **[Single-Agent Workflows](book/src/workflows/single-agent.md)**: The lifecycle of an agent claim, ensure, validate, and finish loop.
*   **[Multi-Agent Workflows](book/src/workflows/multi-agent.md)**: Handling concurrent agents, task claiming, and locking database state.
*   **[Workspace Isolation](book/src/workflows/workspace-isolation.md)**: Setting up isolated Git worktrees and Docker containers to run tasks securely.
*   **[External Trackers](book/src/workflows/external-trackers.md)**: Integrating Decapod with Jira, Linear, or GitHub Issues.

---

## 💡 Core Concepts

Deep dive into the architecture and mechanisms that make Decapod unique.

*   **[Agent-First Architecture](book/src/concepts/agent-first.md)**: Why Decapod is designed to be called directly at agent "inference pressure points".
*   **[Explicit Intent](book/src/concepts/intent.md)**: Converting ambiguous prompts into concrete, versioned specifications.
*   **[Workspace Sandboxing](book/src/concepts/workspaces.md)**: How isolated execution layers keep your primary branches clean and safe.
*   **[Proof & Validation](book/src/concepts/proof.md)**: Verifying correctness programmatically through policy evaluation instead of agent self-reporting.
*   **[Repository Constitution](book/src/concepts/constitution.md)**: Setting the global guidelines that steer agent behavior.
*   **[Config Overrides](book/src/concepts/overrides.md)**: Project-specific adjustments to constitutional guidelines.
*   **[Model Context Protocol (MCP)](book/src/concepts/mcp.md)**: Navigating Decapod tools through structured agent protocols.

---

## 📖 Reference Manual

Hard specifications, command lists, configuration schemas, and error codes.

*   **[Config Specification (config.toml)](book/src/reference/config-toml.md)**: Key-value reference for repo-level policy control.
*   **[CLI Reference](book/src/reference/cli.md)**: Detailed breakdown of commands and options (init, validate, session, todo, decide).
*   **[Error Reference](book/src/reference/errors.md)**: Decapod exit codes, validator failures, and self-healing instructions.
*   **[Artifact Reference](book/src/reference/artifacts.md)**: Layout and schema of generated intent, handshake, and validation specs.

---

## 🛠️ Developer Resources

To contribute or integrate Decapod into your platform:
*   Visit the main GitHub Repository: [DecapodLabs/decapod](https://github.com/DecapodLabs/decapod)
