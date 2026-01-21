# MSRV CI Fix Summary

## Problem

The MSRV (Minimum Supported Rust Version) CI job was failing with the following error:

```
failed to parse the `edition` key
this version of Cargo is older than the `2024` edition, and only supports `2015`, `2018`, and `2021` editions.
```

**Root Cause**: The `home` crate v0.5.12 requires Rust Edition 2024, which needs Rust 1.85+. However, our MSRV is set to Rust 1.70.

## Solution

We pinned the `home` crate to version 0.5.11, which is compatible with Rust Edition 2021 and Rust 1.70.

## Changes Made

### 1. `Cargo.toml`

#### Added explicit MSRV declaration (line 5):
```toml
rust-version = "1.70"
```

#### Added dependency patch at the end of file (lines 95-98):
```toml
# Pin home crate to maintain MSRV 1.70 compatibility
# home 0.5.12+ requires Rust Edition 2024 (Rust 1.85+)
[patch.crates-io]
home = "=0.5.11"
```

### 2. `.github/workflows/ci.yml`

Added documentation comment in the MSRV job (lines 60-61):
```yaml
# MSRV is set to 1.70. The home crate is pinned to 0.5.11 in Cargo.toml
# to avoid dependency on Edition 2024 (home 0.5.12+ requires Rust 1.85+)
```

## How It Works

The `[patch.crates-io]` section in Cargo.toml tells Cargo to replace any instance of the `home` crate from crates.io with version 0.5.11, regardless of what version is requested by dependencies. This allows us to:

1. Keep MSRV at Rust 1.70
2. Avoid breaking changes from newer `home` versions
3. Maintain compatibility with all transitive dependencies that use `home`

## Dependency Chain

```
oops
└── dirs 5.0.1
    └── dirs-sys 0.4.1
        └── home 0.5.x  <-- This is where the issue originated
```

## Testing

After applying this fix, the MSRV CI job should pass with Rust 1.70. To verify locally:

```bash
# Install Rust 1.70
rustup install 1.70

# Build with Rust 1.70
cargo +1.70 build

# Run tests with Rust 1.70
cargo +1.70 test
```

## Future Considerations

### When to Update MSRV

Consider updating the MSRV to Rust 1.85+ if:
- You need features from newer Rust versions
- Many dependencies require Edition 2024
- Maintaining the pin becomes cumbersome

### How to Update MSRV

If you decide to update the MSRV in the future:

1. Remove the `[patch.crates-io]` section from `Cargo.toml`
2. Update `rust-version` in `Cargo.toml` to the new MSRV
3. Update the CI workflow to use the new Rust version
4. Run `cargo update` to get the latest compatible dependencies
5. Test thoroughly across all platforms

## Alternative Solutions Considered

### Option A: Update MSRV to Rust 1.85+ ❌
- **Pros**: Simpler, no dependency pinning needed
- **Cons**: Breaks compatibility for users on older Rust versions, requires more recent toolchain

### Option B: Pin home crate to 0.5.11 ✅ (Selected)
- **Pros**: Maintains backward compatibility, minimal disruption
- **Cons**: Requires manual maintenance of the pin

### Option C: Remove MSRV job ❌
- **Pros**: No maintenance burden
- **Cons**: No guarantee of MSRV compatibility, could break for users

## Related Documentation

- [Cargo Patch Documentation](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section)
- [Rust Editions Guide](https://doc.rust-lang.org/edition-guide/)
- [MSRV Best Practices](https://doc.rust-lang.org/cargo/reference/manifest.html#the-rust-version-field)

## Maintenance Notes

- Check periodically if newer versions of `home` support Edition 2021
- When updating to Rust 1.85+, remember to remove the patch
- Keep an eye on `dirs` and `dirs-sys` for updates that might resolve this
