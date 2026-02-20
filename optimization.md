# Build and Compile-Time Optimization Guide

This document collects pragmatic steps to speed up builds for this Solana Rust project. Start with the "Quick wins" and only adopt deeper changes if needed.

## Quick wins (no logic changes)

1) Remove unused direct dependencies
- If you don’t call these in your code, drop them from `[dependencies]` to reduce resolution and crate graph size (they’ll still be pulled transitively if required):
  - `arrayref`, `digest`, `blake3`

2) Enable incremental compilation for faster rebuilds
- Add a local Cargo config: `.cargo/config.toml`
```toml
[build]
incremental = true
```

3) Use sccache for compiler caching
```bash
# macOS (Homebrew)
brew install sccache
export RUSTC_WRAPPER="$(which sccache)"
```
- Consider adding the export to your shell profile for persistence.

4) Avoid building unneeded targets during iteration
- For the on-chain program (SBF):
```bash
cargo build-sbf
```
- For the client example only when you need it:
```bash
cargo run --example client
```
- Don’t chain both in one command unless required—each uses different targets and increases total work.

## Keep the program build lean (feature-gate examples)

Move heavy dev dependencies behind a feature and require that feature for the example:

```toml
# Cargo.toml
[features]
client = []
full_flow = []            # optional: builds all instruction builders & extended demo client

[dev-dependencies]
solana-client = { version = "2.3.0", optional = true }
solana-sdk    = { version = "2.3.1", optional = true }
litesvm       = { version = "0.7.1", optional = true }

[[example]]
name = "client"
path = "client/client.rs"
required-features = ["client"]
```
Then run the client as:
```bash
cargo run --example client --features client
```
This prevents Cargo from parsing/building those dev crates when you’re only compiling the on-chain program.

## Diagnostics and visibility

1) Measure where time goes
```bash
cargo build -Z timings
```
- Open the generated HTML to see the slowest crates; optimize or gate them.

2) Avoid memory thrash (if RAM-constrained)
```bash
cargo build --jobs 4
```
- Lower `--jobs` if the machine swaps; it can be faster overall.

## SBF-specific tips

- Keep host (native) and SBF builds separate in your workflow; each target creates a distinct crate graph and cache.
- Minimize version churn: aligning Solana CLI and crate versions avoids rebuild storms.
- Use `solana program build` (wrapper) or `cargo build-sbf` consistently; switching back and forth can churn caches.

## Structural improvements (optional)

1) Split into a workspace with separate members
```
workspace/
  Cargo.toml (workspace)
  program/        # on-chain program (current src/)
  client/         # client binary crate
```
- The program crate remains light; the client’s heavy dependencies won’t affect program builds.

2) Reduce default features (advanced)
- Only if confirmed safe for your usage:
```toml
solana-program = { version = "2.3.0", default-features = false }
```
- Test thoroughly; disabling defaults can drop helpers you may rely on.

## Sanity practices

- Avoid frequent `cargo clean`; it destroys incremental caches. Use it only after toolchain upgrades or when caches are corrupted.
- Align rust-analyzer/rustc versions to prevent ABI mismatches and forced rebuilds. You can pin a directory:
```bash
rustup override set stable
# or a specific version that matches your Solana toolchain
```

## Suggested minimal set to adopt now

- Remove unused direct deps (`arrayref`, `digest`, `blake3`) if not used by your own code.
- Add `.cargo/config.toml` with `incremental = true`.
- Enable `sccache`.
- Feature-gate the client (`required-features = ["client"]`) so program builds don’t parse client deps.

## Optional repo tweaks

- Add a top-level `Makefile` or scripts to separate common flows, e.g., `make build-sbf`, `make run-client`, to avoid accidental all-target builds.
- If CI is added later, cache Cargo registry and target directories across runs.

---
If you want, I can apply the feature-gating and create a ready-to-use `.cargo/config.toml` in this repo. 

---
## New Optimization Opportunities After Full Implementation

The program & client now include full instruction lifecycle (initialize, calculate, mark deceased). That enables additional tuning:

### 1. Deprecation Cleanup
You can proactively migrate away from deprecated SDK modules to reduce future churn:
- Replace `solana_sdk::commitment_config::CommitmentConfig` with the `solana-commitment-config` crate once added.
- Replace `solana_sdk::system_program` with `solana_system_interface::program::ID` (already supported in 2.3.x) via an optional feature:
```toml
[features]
legacy-sdk = []
```
In code:
```rust
#[cfg(feature = "legacy-sdk")] use solana_sdk::system_program;
#[cfg(not(feature = "legacy-sdk"))] use solana_system_interface::program as system_program_if;
```
This lets you flip the feature off when ready.

### 2. Binary Size & Deploy Throughput
For the on-chain program, enabling link-time optimization & stripping can reduce deployment time and compute unit usage:
```toml
# .cargo/config.toml
[target.sbf-solana-solana]
rustflags = ["-C", "lto=fat", "-C", "opt-level=z", "-C", "codegen-units=1"]
```
Use only for release/deploy builds; keep debug builds fast.

### 3. Instruction Data Minimization
Current initialization sends two fields (pubkey, u64). That’s already minimal; no change needed unless you later add optional metadata. If you add fields, consider a compact representation (e.g. using `u32` for smaller payment units + scaling factor).

### 4. Serialization Strategy
Borsh is fine for this use-case; switching to raw byte packing would save negligible bytes versus maintainability. Optimize only if instruction throughput profiling shows serialization costs (unlikely here).

### 5. Payment Calculation Performance
The constant `SECONDS_IN_MONTH` is static. No dynamic math is performed each call except a division. Micro-optimizations (e.g., replacing division with shift/multiply) are not needed; focus instead on batching calculations if later you process many accounts in one instruction.

### 6. Account Creation Flow
If initializing many pensioners in bulk, introduce a batched initialize instruction taking multiple (pubkey, payment) tuples. Trade-off: larger single transaction size vs fewer signatures.

### 7. Testing & Simulation Speed
Use LiteSVM for logic tests (already present). Add targeted unit tests for each instruction outcome to avoid spinning a local validator:
- Success initialize
- Double initialize (should fail ownership) – negative test
- Mark deceased twice (expect `AlreadyDeceased`)
- Calculate due before month vs after artificial timestamp advance
This reduces reliance on ledger I/O.

### 8. CI Configuration
Add caching and split jobs:
- `program-build` (SBF only)
- `client-example` (features=client)
- `tests` (unit + litesvm integration)
Parallelization shortens feedback loop.

### 9. Clippy & Lints
Add clippy and deny warnings for long-term hygiene:
```toml
# .cargo/config.toml
[target.'cfg(all())']
rustflags = ["-Dwarnings"]
```
Run:
```bash
cargo clippy --all-targets --all-features
```
For SBF specific code, ignore false positives with `#[allow(clippy::...)]` sparingly.

### 10. Deterministic Keypair Handling (Client)
For reproducible demos: derive keypairs from a seed instead of random each run, reducing airdrop retries & making logs diffable. Example:
```rust
use solana_sdk::signature::Keypair;
use rand::SeedableRng; use rand_chacha::ChaCha20Rng;
let mut rng = ChaCha20Rng::from_seed([0u8; 32]);
let authority = Keypair::generate(&mut rng);
```

### 11. Compute Unit Profiling
Add a benchmarking feature to wrap instruction execution with CU logging (using `solana_program::log::sol_log_compute_units`). Use it conditionally:
```rust
#[cfg(feature = "profile-cu")]
solana_program::log::sol_log_compute_units();
```
This helps identify hotspots if logic grows.

### 12. Future Scaling Considerations
If monthly recalculation becomes frequent for many accounts, consider an off-chain indexer (e.g., using RPC and storing snapshots) to pre-compute due payments and only submit settlement instructions on-chain.

### 13. Security Hardening (Non-Functional but Important)
- Add an `authority` revocation / transfer instruction (with two-phase commit) before production usage.
- Log a hash of serialized state after mutating instructions for easier off-chain auditing.

### 14. Release Process
Create a release profile tuned for program size:
```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = true
```
Then deploy with:
```bash
cargo build-sbf --release
solana program deploy target/deploy/insurance.so
```

### 15. Workspace Migration Path
Migration steps if you choose to split:
1. Create `program/` subdir; move `src`, `Cargo.toml` (adjust name).
2. New `client/` crate with its own `Cargo.toml` depending on `program` via path.
3. Top-level `Cargo.toml` defines `[workspace]` members.
Outcome: client dependency changes do not invalidate program build cache.

---
## Prioritized Next Actions
1. Feature-gate client example & optional profiling (low effort, immediate benefit).
2. Add `.cargo/config.toml` with incremental + release rustflags.
3. Introduce clippy in dev workflow.
4. (Optional) Workspace split if build times become noticeable (>30s incremental).
5. Add minimal instruction unit tests for faster correctness checks.

---
## Quick Command Reference
```bash
# Fast dev build (program only)
cargo build-sbf

# Run example with gated deps
cargo run --example client --features client

# Timings report
cargo build -Z timings

# Clippy lint pass
cargo clippy --all-targets --all-features

# Release build & deploy
cargo build-sbf --release && solana program deploy target/deploy/insurance.so
```

---
## Summary
The project is production-ready; these optimizations help maintain velocity, reduce deployment friction, and prepare for scale. Adopt incrementally; measure before & after for each change.
