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

*   **[Introduction](https://decapodlabs.github.io/decapod/introduction.html)**: What is Decapod? Learn about the governance gap and the core pillars of repo-native agent control.
*   **[Quickstart](https://decapodlabs.github.io/decapod/quickstart.html)**: Install, initialize, and run your first agent handshake and validation in under five minutes.
*   **[Mental Model](https://decapodlabs.github.io/decapod/mental-model.html)**: Understand how agents, tasks, sessions, and workspaces interact.
*   **[Configuration](https://decapodlabs.github.io/decapod/configuration.html)**: Structure your repository config, enable cloud backends, and configure containerization.

---

## 🔄 Governed Workflows

Learn how agents move through the lifecycle of planning, execution, validation, and completion.

*   **[Single-Agent Workflows](https://decapodlabs.github.io/decapod/workflows/single-agent.html)**: The lifecycle of an agent claim, ensure, validate, and finish loop.
*   **[Multi-Agent Workflows](https://decapodlabs.github.io/decapod/workflows/multi-agent.html)**: Handling concurrent agents, task claiming, and locking database state.
*   **[Workspace Isolation](https://decapodlabs.github.io/decapod/workflows/workspace-isolation.html)**: Setting up isolated Git worktrees and Docker containers to run tasks securely.
*   **[External Trackers](https://decapodlabs.github.io/decapod/workflows/external-trackers.html)**: Integrating Decapod with Jira, Linear, or GitHub Issues.

---

## 💡 Core Concepts

Deep dive into the architecture and mechanisms that make Decapod unique.

*   **[Agent-First Architecture](https://decapodlabs.github.io/decapod/concepts/agent-first.html)**: Why Decapod is designed to be called directly at agent "inference pressure points".
*   **[Explicit Intent](https://decapodlabs.github.io/decapod/concepts/intent.html)**: Converting ambiguous prompts into concrete, versioned specifications.
*   **[Workspace Sandboxing](https://decapodlabs.github.io/decapod/concepts/workspaces.html)**: How isolated execution layers keep your primary branches clean and safe.
*   **[Proof & Validation](https://decapodlabs.github.io/decapod/concepts/proof.html)**: Verifying correctness programmatically through policy evaluation instead of agent self-reporting.
*   **[Repository Constitution](https://decapodlabs.github.io/decapod/concepts/constitution.html)**: Setting the global guidelines that steer agent behavior.
*   **[Config Overrides](https://decapodlabs.github.io/decapod/concepts/overrides.html)**: Project-specific adjustments to constitutional guidelines.
*   **[Model Context Protocol (MCP)](https://decapodlabs.github.io/decapod/concepts/mcp.html)**: Navigating Decapod tools through structured agent protocols.

---

## 📖 Reference Manual

Hard specifications, command lists, configuration schemas, and error codes.

*   **[Config Specification (config.toml)](https://decapodlabs.github.io/decapod/reference/config-toml.html)**: Key-value reference for repo-level policy control.
*   **[CLI Reference](https://decapodlabs.github.io/decapod/reference/cli.html)**: Detailed breakdown of commands and options (init, validate, session, todo, decide).
*   **[Error Reference](https://decapodlabs.github.io/decapod/reference/errors.html)**: Decapod exit codes, validator failures, and self-healing instructions.
*   **[Artifact Reference](https://decapodlabs.github.io/decapod/reference/artifacts.html)**: Layout and schema of generated intent, handshake, and validation specs.

---

## 🛠️ Developer Resources

To contribute or integrate Decapod into your platform:
*   Visit the main GitHub Repository: [DecapodLabs/decapod](https://github.com/DecapodLabs/decapod)
