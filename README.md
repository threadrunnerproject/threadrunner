# 🧠 ThreadRunner

## Project Summary

ThreadRunner is a system-level runtime for executing local large language models (LLMs) efficiently and persistently on personal machines. It loads models only when needed, keeps them warm during active use, and provides a unified CLI for seamless interaction.

## Quick Start

Instructions for installing, building, and running ThreadRunner from source. Will include model setup, CLI usage, and daemon behavior overview.

## Architecture

Overview of the core components:
- CLI entrypoint (`threadrunner`)
- Background daemon (`threadrunner-daemon`)
- Model manager, token streaming, and IPC via Unix sockets

## v0.1 Scope

Defines the minimal implementation:
- One-shot prompt via CLI
- Lazy model loading on first request
- Streaming output
- Unix socket-based IPC
- Single-session state

## Future Roadmap

Planned features include:
- Continuous RL from personal data
- Modal/task-based tool switching
- Multi-model support
- Plugin system
- Persistent memory
- Web/GUI integration
- Cross-platform support

## License

ThreadRunner is open source under the MIT license.
