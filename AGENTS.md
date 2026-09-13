Please see README.md what the project is.

This project is a Rust 2024 library. Library implementations are in focused modules under `src`. Integration tests are under `tests`. The wolf-sheep-grass model is under `examples/wolf_sheep_grass`.

Comments should be used sparingly and all prose should be in ASD-STE100 Simplified Technical English.

Code should be rigorously tested in accordance with best practices. Tests should generally be written before code.

Before you finish a change, run:
- `cargo fmt --all -- --check`
- `cargo check --all-targets --all-features`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --all-features`

Functions should be readable and appropriately split up. All identifiers should be clear and appropriate.
