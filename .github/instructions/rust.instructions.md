---
applyTo: "**/*.rs"
---

# Rust code guidelines

- Build/test before commit: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- Prefer `&str` over `String`, use `anyhow::Result` for fallible paths, avoid `unwrap`/`expect`
- Use `is_app(cmd, &["tool", "tool.exe"])` for cross-platform detection and handle `\n`/`\r\n` via `.lines()`
- Add focused tests alongside changes (unit in `src` modules or integration in `tests/`) following existing rule test patterns
