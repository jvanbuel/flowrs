# Binary Size Reduction Design

## Goal

Reduce the flowrs release binary size from 26MB to approximately 12-16MB (~40-50% reduction) through tokio feature trimming and aggressive release build optimizations.

## Changes

### 1. Tokio Feature Trimming

**Current:**
```toml
tokio = { version = "1.48.0", features = ["full"] }
```

**Proposed:**
```toml
tokio = { version = "1.48.0", features = ["rt-multi-thread", "sync", "macros"] }
```

**Rationale:** The codebase only uses:
- `tokio::sync::mpsc` - channels (requires `sync`)
- `tokio::spawn` - spawning tasks (requires `rt-multi-thread`)
- `tokio::task::JoinSet` - managing concurrent tasks (requires `rt-multi-thread`)
- `#[tokio::main]` - the runtime (requires `rt-multi-thread`, `macros`)
- `#[tokio::test]` - async tests (requires `rt-multi-thread`, `macros`)

Unused features being dropped: `io-util`, `io-std`, `net`, `time`, `fs`, `process`, `signal`.

### 2. Release Profile Optimizations

**Current:**
```toml
[profile.dist]
inherits = "release"
lto = "thin"
```

**Proposed:**
```toml
[profile.release]
opt-level = "z"      # Optimize for size
lto = "fat"          # Full link-time optimization
codegen-units = 1    # Single codegen unit for better optimization
strip = true         # Strip symbols from binary
panic = "abort"      # No unwinding machinery

# [profile.dist] section removed - no longer needed
```

**Setting explanations:**
- `opt-level = "z"` - Compiler prioritizes smallest code over speed
- `lto = "fat"` - Cross-crate optimization, better dead code elimination than "thin"
- `codegen-units = 1` - Better optimization at cost of slower compilation
- `strip = true` - Removes debug symbols and symbol table
- `panic = "abort"` - Removes unwinding machinery

## Trade-offs

- Release builds will take 2-3x longer due to LTO and single codegen unit
- `panic = "abort"` means no stack unwinding on panic (acceptable for TUI apps)
- `opt-level = "z"` may have minor runtime performance impact (negligible for I/O-bound TUI)

## Expected Results

- Baseline: 26MB
- Target: 12-16MB (40-50% reduction)

## Verification

1. Apply changes to `Cargo.toml`
2. Run `cargo clean`
3. Build with `cargo build --release`
4. Compare binary size
5. Run `cargo test` to ensure nothing broke
