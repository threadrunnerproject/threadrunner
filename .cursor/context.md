# Context

# Context and Background

ThreadRunner was created to solve a specific gap in local LLM tooling:
- Existing tools like `llama.cpp` or `ollama` either run in an always-on server mode or rely on manual re-execution.
- Most CLI tools re-load models from disk on every request, which is inefficient and memory-wasteful.
- There’s no standard open-source runtime that:
  1. Behaves like a system service
  2. Automatically loads a model when needed
  3. Streams output
  4. Exits or sleeps when idle

The project is intended to be:
- **Composable** — cleanly separated core/runtime logic
- **Efficient** — CPU-only by default, GPU later
- **Extensible** — designed for tools, plugins, agents, and personal reinforcement learning
- **Open** — licensed under MIT, with a public GitHub repo and CI from day one

This is version `v0.1` and should be treated as the stable foundation for a larger AI-native operating layer.
