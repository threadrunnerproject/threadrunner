# Architecture

# ThreadRunner Architecture Overview

## Components

### 1. `threadrunner` (CLI)
- Single command interface for users
- Handles prompt input, daemon detection, fallback to daemon launch if needed
- Sends prompt via Unix domain socket
- Receives streamed token output and displays it live

### 2. `threadrunner-daemon`
- Background process managed manually or via autostart (`systemd`, `launchd`)
- Loads quantized GGUF model into memory on demand
- Manages inference sessions, caches KV memory
- Tracks idle time and unloads model when not in use
- Accepts prompt messages via IPC and returns streamed token output

### 3. `ModelManager` (in `threadrunner-core`)
- Abstract interface for model loading, unloading, and token generation
- Will eventually support model hot-swapping and memory paging
- Wraps llama.cpp or llama-rs via trait-based inference layer

### 4. IPC Layer
- Unix domain sockets at path: `~/.threadrunner.sock`
- Message format: length-prefixed JSON or bincode packets
- One prompt per request; streaming handled via socket writes

## File Paths and Conventions
- Model storage: `~/.threadrunner/models/`
- Config (future): `~/.config/threadrunner/config.toml`
- Log files (optional): `~/.cache/threadrunner/log.txt`

## Memory Strategy
- Lazy load models using `mmap`
- Track time since last prompt
- Unload after N seconds (default: 300) of inactivity
