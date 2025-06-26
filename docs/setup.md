# Setup

WIP

## Model Path Configuration

**Default model directory:** `~/.threadrunner/models/`

**Environment override:** `THREADRUNNER_MODEL_PATH`

**Note:** v0.1 automatically downloads `llama2-7b.Q4_K_M.gguf` if the model is not present in the model directory.

## Selecting a Backend

ThreadRunner supports multiple inference backends through Cargo feature flags. Choose the appropriate backend for your use case:

### Real Model Inference (Production)

To use a real language model with llama.cpp:

```bash
cargo build --workspace --no-default-features --features llama
```

This enables the llama backend which:
- Loads GGUF model files 
- Provides real AI inference
- Requires a model file (see Model Path Configuration above)

### Dummy Backend (Development & CI)

Default compilation uses the dummy backend:

```bash
cargo build --workspace
# or simply: cargo build
```

The dummy backend:
- Generates lorem ipsum-style tokens for testing
- Requires no model files
- Used for development and CI pipelines
- Faster compilation and execution for testing

### Environment Variables

Set these environment variables to control runtime behavior:

- `THREADRUNNER_BACKEND`: Override backend selection (`llama` or `dummy`)
- `THREADRUNNER_MODEL_PATH`: Override model file path (llama backend only)

### Examples

```bash
# Build and run with real llama model
export THREADRUNNER_BACKEND=llama
export THREADRUNNER_MODEL_PATH="$HOME/.threadrunner/models/tinyllama-1.1b-chat-v1.0.Q4_K_M.gguf"
cargo build --workspace --no-default-features --features llama
./target/debug/threadrunner "Hello, how are you?"

# Build and run with dummy backend (for testing)
cargo build --workspace
./target/debug/threadrunner "Hello, how are you?"
``` 