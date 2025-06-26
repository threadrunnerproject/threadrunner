# Project Description

# ThreadRunner: A System-Level Runtime for Local Language Model Execution

ThreadRunner is an open-source, system-level runtime designed to execute and manage local large language models (LLMs) efficiently. It acts as a long-running background service that loads quantized models on demand, keeps them warm while in use, and releases memory when idle. Users interact with the system via a unified CLI (`threadrunner`), which routes prompts to a daemon process via Unix domain sockets.

The daemon retains memory between prompts during active sessions, providing fast, context-aware inference without requiring cloud APIs. This project prioritizes local-first execution, privacy, extensibility, and composable architecture. Future versions will support multi-model runtimes, RL from personal data, tool/plugin systems, and agentic control over local resources.

This repo contains a Rust workspace split into:
- `crates/core`: shared abstractions and interfaces
- `crates/daemon`: background process that manages the runtime and model lifecycle
- `crates/cli`: the user-facing CLI frontend

This is version `v0.1` of the project, focused solely on building a daemon-aware, lazy-loading, streaming LLM runtime with CLI access.
