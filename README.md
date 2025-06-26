# ThreadRunner

## Project Summary

ThreadRunner is a system-level runtime for executing local large language models (LLMs) efficiently and persistently on personal machines. It loads models only when needed, keeps them warm during active use, and provides a unified CLI for seamless interaction.

## Quick Start

### Fast Testing (Dummy Backend)

Get started immediately with no model files required:

```bash
# Clone and build with dummy backend (default)
git clone <repo-url>
cd threadrunner
cargo build --workspace

# Run with generated test tokens
./target/debug/threadrunner "Hello, how are you?"
# Output: lorem ipsum dolor sit amet... Hello. how. are. you.
```

### Real AI Inference (Llama Backend)

For actual language model inference:

```bash
# 1. Download a GGUF model (example: TinyLlama 1.1B)
mkdir -p ~/.threadrunner/models
cd ~/.threadrunner/models
wget https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf

# 2. Build with llama backend
cd /path/to/threadrunner
cargo build --workspace --no-default-features --features llama

# 3. Run with real AI
export THREADRUNNER_BACKEND=llama
export THREADRUNNER_MODEL_PATH="$HOME/.threadrunner/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
./target/debug/threadrunner "Hello! Please tell me about yourself."
# Output: Real AI-generated response with streaming tokens
```

**Note:** The daemon automatically starts in the background and persists between CLI calls for efficient model reuse.

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
