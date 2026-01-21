# Development Notes for CI/MSRV

## MSRV (Minimum Supported Rust Version)

This project maintains compatibility with **Rust 1.70**.

### Why Rust 1.70?

Rust 1.70 was released in June 2023 and provides a good balance between:
- Modern language features
- Wide compatibility with older systems
- Stable ecosystem support

### Home Crate Pin

⚠️ **Important**: The `home` crate is pinned to version 0.5.11 in `Cargo.toml`:

```toml
[patch.crates-io]
home = "=0.5.11"
```

**Reason**: Version 0.5.12+ requires Rust Edition 2024 (Rust 1.85+), which would break our MSRV.

This pin ensures that all dependencies (including transitive ones like `dirs` → `dirs-sys` → `home`) use a compatible version.

### Testing MSRV Locally

```bash
# Option 1: Use the verification script
chmod +x scripts/verify-msrv.sh
./scripts/verify-msrv.sh

# Option 2: Manual testing
rustup install 1.70
cargo +1.70 build
cargo +1.70 test
```

### CI Configuration

The MSRV is tested in CI via the `msrv` job in `.github/workflows/ci.yml`:

```yaml
msrv:
  name: MSRV (Minimum Supported Rust Version)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Rust 1.70
      uses: dtolnay/rust-toolchain@1.70
    - name: Build
      run: cargo build
```

### Updating MSRV

If you need to update the MSRV in the future:

1. **Update `Cargo.toml`**:
   ```toml
   [package]
   rust-version = "1.XX"  # Update to new MSRV
   ```

2. **Update CI workflow**:
   ```yaml
   - name: Install Rust 1.XX
     uses: dtolnay/rust-toolchain@1.XX
   ```

3. **Consider removing the home crate pin** if updating to Rust 1.85+:
   ```toml
   # Remove this section if MSRV >= 1.85
   [patch.crates-io]
   home = "=0.5.11"
   ```

4. **Run full test suite**:
   ```bash
   cargo +1.XX test --all-features
   cargo +1.XX build --release
   ```

5. **Update documentation** (README.md, CHANGELOG.md)

### Troubleshooting

#### "edition `2024` is unstable" error
This means a dependency requires Edition 2024 (Rust 1.85+). Check:
1. If the `home` crate pin is in place
2. Run `cargo tree` to find which dependency needs a newer version
3. Consider pinning that dependency or updating MSRV

#### Build fails with older Rust
1. Check that the `rust-version` field matches your installed version
2. Verify all dependencies support the MSRV
3. Look for features that require newer Rust versions

#### MSRV CI job fails
1. Verify the toolchain version in CI matches `rust-version` in Cargo.toml
2. Check if any new dependencies require a newer Rust version
3. Review recent Cargo.lock changes

### Dependency Management

When adding new dependencies:

1. **Check their MSRV** before adding
2. **Test with Rust 1.70** after adding
3. **Consider pinning** if they require newer Rust
4. **Document** any special requirements

Example check:
```bash
# Check a crate's MSRV
cargo info <crate-name> | grep -i rust

# Test after adding
cargo +1.70 check
```

### Related Files

- `Cargo.toml` - Contains `rust-version` and `[patch.crates-io]`
- `.github/workflows/ci.yml` - MSRV CI job configuration
- `scripts/verify-msrv.sh` - Local MSRV verification script
- `MSRV_FIX_SUMMARY.md` - Detailed explanation of the home crate issue

### References

- [Cargo rust-version field](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)
- [Cargo patch section](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section)
- [Rust Editions](https://doc.rust-lang.org/edition-guide/)
