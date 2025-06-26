# 🧵 ThreadRunner

### *A Modern System-Level Runtime for Local LLMs*

[![Build Status](https://img.shields.io/github/workflow/status/username/threadrunner/CI)](https://github.com/username/threadrunner/actions)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](Cargo.toml)

*Efficient • Persistent • Streaming • Fast*

---

## 🎯 **What is ThreadRunner?**

ThreadRunner is a **high-performance system daemon** that makes local large language model (LLM) inference feel as natural as using any command-line tool. It intelligently manages model loading, provides persistent background execution, and delivers real-time streaming responses—all optimized for personal machines.

### ✨ **Key Features**

🚀 **Lazy Loading** - Models load only when needed, not on system startup  
🔥 **Hot Persistence** - Keep models warm in memory between requests  
⚡ **Streaming Output** - Real-time token generation for responsive UX  
🔌 **Unix Socket IPC** - Lightning-fast inter-process communication  
🎛️ **Multi-Backend** - Support for both real AI and testing backends  
🛡️ **Resource Management** - Automatic cleanup and memory optimization  

---

## 🏗️ **Architecture Overview**

ThreadRunner follows a **client-daemon architecture** with clean separation between user interface and model execution:

graph TD
    A["🖥️ CLI User"] --> B["`**threadrunner CLI**<br/>Entry Point`"]
    B --> C{"`🔍 Check Daemon<br/>Running?`"}
    C -->|No| D["`🚀 Spawn Daemon<br/>Process`"]
    C -->|Yes| E["`🔌 Connect via<br/>Unix Socket`"]
    D --> E
    E --> F["`📡 Send JSON Request<br/>{prompt, stream: true}`"]
    F --> G["`🧠 Daemon Process<br/>threadrunner-daemon`"]
    
    G --> H{"`🎯 Model<br/>Loaded?`"}
    H -->|No| I["`⚡ Load Model<br/>Backend`"]
    H -->|Yes| J["`🔄 Process<br/>Prompt`"]
    I --> J
    
    J --> K["`🔥 Generate Tokens<br/>Streaming`"]
    K --> L["`📤 Send Token<br/>Response`"]
    L --> M["`📥 CLI Receives<br/>& Displays`"]
    M --> N{"`🏁 End of<br/>Stream?`"}
    N -->|No| K
    N -->|Yes| O["`✅ Complete<br/>& Exit`"]
    
    subgraph "💾 Backend Options"
        P["`🦙 Llama Backend<br/>Real AI Inference`"]
        Q["`🎭 Dummy Backend<br/>Lorem Ipsum Test`"]
    end
    
    I --> P
    I --> Q
    
    style A fill:#e1f5fe
    style B fill:#f3e5f5
    style G fill:#f3e5f5
    style P fill:#e8f5e8
    style Q fill:#fff3e0
    style O fill:#e8f5e8

### 🧱 **Component Structure**

graph LR
    subgraph "🏗️ Workspace Structure"
        A["`**crates/**<br/>📦 Rust Crates`"]
        A --> B["`**cli/**<br/>🖥️ Command Line Interface`"]
        A --> C["`**daemon/**<br/>⚙️ Background Service`"]
        A --> D["`**core/**<br/>🎯 Shared Components`"]
    end
    
    subgraph "🎯 Core Components"
        D --> E["`**model.rs**<br/>🧠 ModelBackend Trait`"]
        D --> F["`**ipc.rs**<br/>📡 JSON Protocol`"]
        D --> G["`**error.rs**<br/>⚠️ Error Handling`"]
        D --> H["`**llama_backend.rs**<br/>🦙 Llama.cpp Integration`"]
    end
    
    subgraph "🖥️ CLI Features"
        B --> I["`**client.rs**<br/>🔌 Socket Connection`"]
        B --> J["`**frame.rs**<br/>📦 Message Framing`"]
        B --> K["`**config.rs**<br/>⚙️ Configuration`"]
    end
    
    subgraph "⚙️ Daemon Features"
        C --> L["`**daemon.rs**<br/>🔄 Main Loop`"]
        C --> M["`**state.rs**<br/>💾 Model State`"]
        C --> N["`**frame.rs**<br/>📦 Message Framing`"]
    end
    
    style A fill:#e3f2fd
    style D fill:#f3e5f5
    style B fill:#e8f5e8
    style C fill:#fff3e0

---

## 🚀 **Quick Start Guide**

### 🎭 **Option 1: Fast Testing (Dummy Backend)**

Get started immediately with **zero setup** required:

```bash
# Clone the repository
git clone https://github.com/username/threadrunner.git
cd threadrunner

# Build with dummy backend (default)
cargo build --workspace

# Run with generated test tokens
./target/debug/threadrunner "Hello, how are you?"
```

**Expected Output:**
```
lorem ipsum dolor sit amet Hello. how. are. you.
```

> 💡 **Perfect for**: Development, CI/CD, testing the architecture without model files

### 🦙 **Option 2: Real AI Inference (Llama Backend)**

For actual language model inference:

```bash
# 1. 📁 Create model directory
mkdir -p ~/.threadrunner/models
cd ~/.threadrunner/models

# 2. ⬇️ Download a GGUF model (example: TinyLlama 1.1B)
wget https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf

# 3. 🔨 Build with llama backend
cd /path/to/threadrunner
cargo build --workspace --no-default-features --features llama

# 4. 🎯 Configure environment
export THREADRUNNER_BACKEND=llama
export THREADRUNNER_MODEL_PATH="$HOME/.threadrunner/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"

# 5. 🚀 Run with real AI
./target/debug/threadrunner "Hello! Please tell me about yourself."
```

**Expected Output:**
```
Hello! I'm TinyLlama, a conversational AI assistant. I'm designed to be helpful, harmless, and honest. I can assist you with various tasks like answering questions, helping with writing, explaining concepts, and having friendly conversations. How can I help you today?
```

> 🔥 **The daemon automatically starts in the background and persists between CLI calls for efficient model reuse!**

---

## 🔧 **Build Configurations**

ThreadRunner supports multiple backends through Cargo feature flags:

graph TB
    subgraph "🔧 Build Configurations"
        A["`**Default Build**<br/>cargo build`"] --> B["`🎭 Dummy Backend<br/>Lorem Ipsum Testing`"]
        C["`**Llama Build**<br/>--features llama`"] --> D["`🦙 Llama Backend<br/>Real AI Inference`"]
    end
    
    subgraph "🎭 Dummy Backend Features"
        B --> E["`✅ No Model Files Needed`"]
        B --> F["`⚡ Fast Development`"]
        B --> G["`🔄 CI/CD Testing`"]
        B --> H["`📝 Lorem Ipsum Output`"]
    end
    
    subgraph "🦙 Llama Backend Features"
        D --> I["`🧠 Real AI Inference`"]
        D --> J["`📁 GGUF Model Support`"]
        D --> K["`🔥 llama.cpp Integration`"]
        D --> L["`💾 Memory Mapped Loading`"]
    end
    
    subgraph "⚙️ Runtime Configuration"
        M["`**Environment Variables**`"] --> N["`THREADRUNNER_BACKEND`"]
        M --> O["`THREADRUNNER_MODEL_PATH`"]
        M --> P["`RUST_LOG`"]
    end
    
    style A fill:#e3f2fd
    style C fill:#e3f2fd
    style B fill:#fff3e0
    style D fill:#e8f5e8
    style M fill:#f3e5f5

### 🛠️ **Build Commands**

| Backend | Command | Use Case |
|---------|---------|----------|
| **Dummy** | `cargo build --workspace` | Development, testing, CI |
| **Llama** | `cargo build --workspace --no-default-features --features llama` | Production AI inference |
| **All Features** | `cargo build --workspace --all-features` | Development with all backends |

---

## 📡 **Communication Protocol**

ThreadRunner uses a **JSON-over-Unix-socket** protocol for high-performance IPC:

sequenceDiagram
    participant User
    participant CLI as "threadrunner CLI"
    participant Daemon as "threadrunner-daemon"
    participant Model as "Model Backend"
    
    User->>CLI: threadrunner "Hello, how are you?"
    CLI->>CLI: Parse arguments & config
    CLI->>Daemon: Check if daemon running
    
    alt Daemon not running
        CLI->>Daemon: Spawn daemon process
        Daemon->>Daemon: Bind Unix socket
        Daemon->>CLI: Ready signal
    end
    
    CLI->>Daemon: Connect via Unix socket
    CLI->>Daemon: Send PromptRequest JSON
    note over CLI,Daemon: {"v": 1, "prompt": "Hello...", "stream": true}
    
    Daemon->>Daemon: Parse request
    
    alt Model not loaded
        Daemon->>Model: Load model from disk
        Model->>Daemon: Model ready
    end
    
    Daemon->>Model: Process prompt
    Model->>Model: Generate tokens
    
    loop For each token
        Model->>Daemon: Return token
        Daemon->>CLI: Send TokenResponse
        note over Daemon,CLI: {"token": "Hello", "eos": false}
        CLI->>User: Display token (streaming)
    end
    
    Model->>Daemon: Generation complete
    Daemon->>CLI: Send final response
    note over Daemon,CLI: {"token": null, "eos": true}
    CLI->>User: Complete output & exit
    
    note over Daemon: Daemon stays alive for next request
    note over Daemon: Auto-unload model after idle timeout

### 📋 **Protocol Specification (v1)**

**Request Format:**
```json
{
  "v": 1,
  "prompt": "Your prompt text here",
  "stream": true
}
```

**Streaming Response Format:**
```json
{
  "token": "Generated",
  "eos": false
}
```

**Final Response Format:**
```json
{
  "token": null,
  "eos": true
}
```

---

## ⚙️ **Configuration**

### 🌍 **Environment Variables**

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `THREADRUNNER_BACKEND` | Backend selection | `llama` or `dummy` | `llama` |
| `THREADRUNNER_MODEL_PATH` | Path to GGUF model file | `~/.threadrunner/models/*.gguf` | `/path/to/model.gguf` |
| `RUST_LOG` | Logging verbosity | `warn` | `debug`, `info`, `trace` |

### 📁 **Directory Structure**

```
~/.threadrunner/
├── models/                 # Model files directory
│   ├── tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf
│   └── other-models.gguf
└── cache/                  # Daemon logs and cache
    └── threadrunner-daemon-2024-01-15.log
```

### 🔧 **Example Configurations**

**Development Setup:**
```bash
export RUST_LOG=debug
cargo build --workspace
./target/debug/threadrunner "Test prompt"
```

**Production Setup:**
```bash
export THREADRUNNER_BACKEND=llama
export THREADRUNNER_MODEL_PATH="$HOME/.threadrunner/models/your-model.gguf"
export RUST_LOG=info
cargo build --workspace --no-default-features --features llama
./target/debug/threadrunner "Your prompt here"
```

---

## 📊 **Exit Codes & Error Handling**

ThreadRunner provides **detailed exit codes** for robust scripting and automation:

| Exit Code | Name | Description | Script Usage |
|-----------|------|-------------|--------------|
| `0` | **Success** | Command completed successfully | Continue execution |
| `1` | **Unknown** | Unknown or unexpected error | Generic error handling |
| `2` | **Connection** | Failed to connect to daemon | Retry or check daemon |
| `3` | **Model** | Model loading or inference error | Check model path/file |
| `4` | **Timeout** | Operation timed out | Increase timeout or retry |

### 🛡️ **Error Handling Example**

```bash
#!/bin/bash
./target/debug/threadrunner "Hello, how are you?"
exit_code=$?

case $exit_code in
  0) echo "✅ Success!" ;;
  2) echo "❌ Connection failed - is the daemon running?" ;;
  3) echo "❌ Model error - check model path and file" ;;
  4) echo "⏱️ Request timed out" ;;
  *) echo "❓ Unknown error occurred" ;;
esac
```

---

## 🔬 **Advanced Usage**

### 📝 **Logging & Debugging**

**Detailed Logging:**
```bash
# Show all debug information
export RUST_LOG=debug
./target/debug/threadrunner "Debug this prompt"

# Target specific modules
export RUST_LOG=threadrunner_daemon=debug,threadrunner_core=info
./target/debug/threadrunner "Targeted logging"

# View daemon logs
tail -f ~/.cache/threadrunner-daemon-$(date +%Y-%m-%d).log
```

### 🔄 **Daemon Management**

The daemon automatically:
- 🚀 **Spawns** when first CLI request arrives
- 🔥 **Persists** between requests for model reuse
- ⏰ **Auto-unloads** models after idle timeout
- 📝 **Logs** all activity to daily rotating files

**Manual Daemon Control:**
```bash
# Check if daemon is running
ps aux | grep threadrunner-daemon

# Kill daemon (will auto-restart on next request)
pkill threadrunner-daemon

# Monitor daemon in real-time
tail -f ~/.cache/threadrunner-daemon-*.log
```

### 🧪 **Testing & Development**

**Run Tests:**
```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p threadrunner-core
cargo test -p threadrunner-daemon
cargo test -p threadrunner-cli

# Run with dummy backend for CI
cargo test --workspace --features dummy
```

**Development Workflow:**
```bash
# Fast iteration with dummy backend
cargo build --workspace
cargo test --workspace
./target/debug/threadrunner "Test prompt"

# Switch to real model testing
cargo build --workspace --no-default-features --features llama
export THREADRUNNER_BACKEND=llama
./target/debug/threadrunner "Real AI test"
```

---

## 🗺️ **Roadmap & Future Vision**

### 🎯 **v0.1 Scope** *(Current)*

✅ One-shot prompt via CLI  
✅ Lazy model loading on first request  
✅ Streaming token output  
✅ Unix socket-based IPC  
✅ Single-session state management  
✅ Dummy backend for testing  
✅ Llama.cpp integration  

### 🚀 **v0.2 Planned Features**

🔄 **Multi-Session Support** - Persistent conversation contexts  
🎯 **Model Switching** - Hot-swap between different models  
⚡ **Performance Optimizations** - Memory pool, async optimizations  
🔌 **Plugin Architecture** - Extensible backend system  

### 🌟 **v1.0+ Vision**

🧠 **Continuous RL** - Learning from personal usage patterns  
🛠️ **Modal Tools** - Task-specific model switching (coding, writing, chat)  
📚 **Multi-Model Support** - Ensemble inference, specialized models  
🌐 **Web/GUI Integration** - Browser extension, desktop app  
🖥️ **Cross-Platform** - Windows, macOS, Linux support  
💾 **Persistent Memory** - Long-term conversation and context memory  

---

## 🏗️ **Project Structure**

```
threadrunner/
├── 📁 crates/
│   ├── 🖥️ cli/                    # Command-line interface
│   │   ├── src/
│   │   │   ├── main.rs            # CLI entry point
│   │   │   ├── client.rs          # Socket client logic
│   │   │   ├── frame.rs           # Message framing
│   │   │   └── config.rs          # CLI configuration
│   │   └── tests/                 # CLI integration tests
│   │
│   ├── ⚙️ daemon/                  # Background daemon service
│   │   ├── src/
│   │   │   ├── main.rs            # Daemon entry point
│   │   │   ├── daemon.rs          # Main daemon loop
│   │   │   ├── state.rs           # Model state management
│   │   │   └── frame.rs           # Message framing
│   │   └── tests/                 # Daemon tests
│   │
│   └── 🎯 core/                    # Shared components
│       ├── src/
│       │   ├── lib.rs             # Core library
│       │   ├── model.rs           # ModelBackend trait
│       │   ├── ipc.rs             # IPC protocol definitions
│       │   ├── error.rs           # Error types
│       │   └── llama_backend.rs   # Llama.cpp integration
│       └── tests/                 # Core tests
│
├── 📚 docs/                        # Documentation
│   ├── architecture.md            # System architecture
│   ├── setup.md                   # Setup instructions
│   └── README.md                  # Documentation index
│
├── 🛠️ scripts/                     # Build and utility scripts
├── 📦 packaging/                   # Distribution packaging
├── 🎯 models/                      # Default model directory
└── 📋 Justfile                     # Task runner commands
```

---

## 🤝 **Contributing**

We welcome contributions! Here's how to get started:

### 🛠️ **Development Setup**

```bash
# Clone and set up development environment
git clone https://github.com/username/threadrunner.git
cd threadrunner

# Install Rust toolchain (see rust-toolchain.toml)
rustup component add clippy rustfmt

# Run tests to verify setup
cargo test --workspace

# Build all features
cargo build --workspace --all-features
```

### 📋 **Development Guidelines**

- 🧪 **Testing**: All new features must include tests
- 📝 **Documentation**: Update docs for user-facing changes  
- 🎨 **Code Style**: Use `cargo fmt` and `cargo clippy`
- 🔧 **Features**: Use feature flags for optional dependencies
- 📊 **Performance**: Profile and benchmark critical paths

### 🎯 **Areas for Contribution**

- 🚀 **Performance Optimizations** - Memory, CPU, I/O improvements
- 🧪 **Backend Implementations** - New model backends (OpenAI, Anthropic, etc.)
- 🛠️ **Tooling** - Build scripts, packaging, CI/CD improvements
- 📚 **Documentation** - Examples, tutorials, API documentation
- 🐛 **Bug Fixes** - Issues, edge cases, error handling

---

## 📄 **License**

ThreadRunner is open source under the **MIT License**.

```
MIT License

Copyright (c) 2024 ThreadRunner Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

<div align="center">

**🧵 ThreadRunner** - *Making local LLMs feel native*

[![⭐ Star this repo](https://img.shields.io/badge/⭐-Star%20this%20repo-yellow)](https://github.com/vivienhenz24/threadrunner)
[![🐛 Report Issues](https://img.shields.io/badge/🐛-Report%20Issues-red)](https://github.com/vivienhenz24/threadrunner/issues)
[![💬 Discussions](https://img.shields.io/badge/💬-Discussions-blue)](https://github.com/vivienhenz24/threadrunner/discussions)

*Built with ❤️ in Rust*

</div>
