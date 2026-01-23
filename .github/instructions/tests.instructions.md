---
applyTo: "**/tests/**/*.rs"
---

# Test writing guidelines

- Mirror existing patterns: descriptive test names (`test_<what>_<scenario>`), use real-world outputs, avoid brittle assumptions
- Keep tests isolated and deterministic; no network or filesystem side effects without fixtures
- Prefer table-driven cases when multiple inputs/outputs share logic
- Run relevant subsets first (`cargo test <module>`), then full `cargo test` before merge when feasible
