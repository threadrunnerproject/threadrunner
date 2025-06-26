# Coding Instructions

# Coding Guidelines and Preferences

- Use idiomatic Rust 2021+.
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) when designing public interfaces in `threadrunner-core`.
- Use `tokio` for async execution; prefer structured concurrency and `tokio::select!` patterns when coordinating tasks.
- Code should be modular and testable — no tightly coupled binaries.
- CLI tools should use `clap` or `argh` for argument parsing.
- Daemon logic should be resilient: graceful shutdown, log errors, no panics on bad input.
- Prefer memory-mapped file loading (`mmap`) for large GGUF model files.
- Stream tokens incrementally to the CLI using a channel or async reader interface.
- Use Unix domain sockets for IPC. Wrap all socket communication in versioned message structs (`enum`-based protocol).
- Keep dependencies minimal. Avoid using large LLM or ML libraries directly — all inference must go through llama-rs or llama.cpp backends.
- Always write `README.md`, `docs/architecture.md`, and `docs/setup.md` before implementing features.

Formatting: run `cargo fmt` before every commit.  
Lints: use `cargo clippy` and prefer warnings as errors once stable.