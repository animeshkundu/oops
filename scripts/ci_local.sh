#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

fmt_toolchain=${RUSTFMT_TOOLCHAIN:-}
clippy_toolchain=${CLIPPY_TOOLCHAIN:-}
msrv_toolchain=${MSRV_TOOLCHAIN:-1.88}

fmt_cmd=(cargo fmt --check)
clippy_cmd=(cargo clippy -- -D warnings)

if [[ -n "$fmt_toolchain" ]]; then
  fmt_cmd=(cargo +"$fmt_toolchain" fmt --check)
fi

if [[ -n "$clippy_toolchain" ]]; then
  clippy_cmd=(cargo +"$clippy_toolchain" clippy -- -D warnings)
fi

echo "==> ${fmt_cmd[*]}"
"${fmt_cmd[@]}" || {
  echo "Formatting check failed. If this is due to rustfmt version differences, run cargo fmt with the CI toolchain." >&2
  exit 1
}

echo "==> ${clippy_cmd[*]}"
"${clippy_cmd[@]}"

echo "==> cargo build --release"
cargo build --release

echo "==> cargo test"
cargo test

echo "==> cargo build (msrv)"
if [[ -n "$msrv_toolchain" ]]; then
  echo "Using MSRV toolchain: ${msrv_toolchain}"
  cargo +"$msrv_toolchain" build
else
  cargo build
fi

echo "==> cargo llvm-cov --lcov --output-path lcov.info"
if command -v cargo-llvm-cov >/dev/null 2>&1; then
  cargo llvm-cov --lcov --output-path lcov.info
else
  echo "cargo-llvm-cov not installed; skipping coverage (install with: cargo install cargo-llvm-cov)"
fi

echo "==> Shell integration tests"
./target/release/oops --version
./target/release/oops --help
TF_SHELL=bash ./target/release/oops --alias | grep -q "eval"
TF_SHELL=zsh ./target/release/oops --alias | grep -q "eval"
TF_SHELL=fish ./target/release/oops --alias | grep -q "function"
TF_SHELL=powershell ./target/release/oops --alias | grep -q "function"
TF_SHELL=tcsh ./target/release/oops --alias | grep -q "alias"

echo "==> cargo audit"
if command -v cargo-audit >/dev/null 2>&1; then
  cargo audit
else
  echo "cargo-audit not installed; skipping audit (install with: cargo install cargo-audit)"
fi
