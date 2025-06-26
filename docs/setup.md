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
- `RUST_LOG`: Control logging verbosity (`info`, `debug`, `trace`, etc.)

#### Logging

ThreadRunner uses the `tracing` crate for structured logging. You can control the log level using the `RUST_LOG` environment variable:

```bash
# Show informational messages
export RUST_LOG=info
./target/debug/threadrunner "Hello, how are you?"

# Show debug messages for more detailed output
export RUST_LOG=debug
./target/debug/threadrunner "Hello, how are you?"

# Show trace messages for maximum verbosity
export RUST_LOG=trace
./target/debug/threadrunner "Hello, how are you?"
```

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

## Exit Codes

ThreadRunner CLI returns specific exit codes to indicate the result of the operation:

| Exit Code | Name       | Description                                    |
|-----------|------------|------------------------------------------------|
| 0         | Ok         | Command completed successfully                 |
| 1         | Unknown    | Unknown or unexpected error occurred          |
| 2         | Connection | Failed to connect to daemon or network error  |
| 3         | Model      | Model loading or inference error               |
| 4         | Timeout    | Operation timed out                            |

These exit codes can be used in scripts to handle different failure scenarios:

```bash
#!/bin/bash
./target/debug/threadrunner "Hello, how are you?"
exit_code=$?

case $exit_code in
  0) echo "Success!" ;;
  2) echo "Connection failed - is the daemon running?" ;;
  3) echo "Model error - check model path and file" ;;
  4) echo "Request timed out" ;;
  *) echo "Unknown error occurred" ;;
esac
```

## Logging Configuration

### Setting RUST_LOG Environment Variable

ThreadRunner uses the standard Rust logging infrastructure. Control log verbosity by setting the `RUST_LOG` environment variable:

```bash
# Show only errors and warnings (default)
export RUST_LOG=warn

# Show informational messages
export RUST_LOG=info

# Show debug messages for detailed troubleshooting
export RUST_LOG=debug

# Show trace messages for maximum verbosity
export RUST_LOG=trace

# Target specific modules (useful for debugging)
export RUST_LOG=threadrunner_daemon=debug,threadrunner_core=info

# Run with the chosen log level
./target/debug/threadrunner "Hello, how are you?"
```

### Daemon Log File Location

The ThreadRunner daemon automatically logs to a daily rotating file:

**Default location:** `~/.cache/threadrunner-daemon-YYYY-MM-DD.log`

Examples of log file paths:
- `~/.cache/threadrunner-daemon-2024-01-15.log`
- `~/.cache/threadrunner-daemon-2024-01-16.log`

The daemon creates a new log file each day and automatically handles log rotation. To view recent daemon activity:

```bash
# View today's daemon log
tail -f ~/.cache/threadrunner-daemon-$(date +%Y-%m-%d).log

# View all recent daemon logs
ls -la ~/.cache/threadrunner-daemon-*.log

# Search for specific events
grep "unloaded model" ~/.cache/threadrunner-daemon-*.log
``` 