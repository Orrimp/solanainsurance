# Pension Insurance Smart Contract — Technical Solution

> Derived from [prd-requirements.md](prd-requirements.md). This document describes how the product requirements are realised technically.

---

## 1. Overall Architecture

The system is a **single native Solana program** paired with an **off-chain client SDK**. There is no application server, no database, and no off-chain middleware for core logic. All pension state lives in Solana accounts; all business rules execute inside the on-chain program.

```
┌─────────────────────────────────────────────────────────────┐
│  Authority / Pensioner keypairs  (off-chain)                │
│                                                             │
│  ┌────────────────────────────────────────────────────┐    │
│  │  Client SDK  (Rust example binary)                 │    │
│  │  client/instructions.rs  — instruction builders    │    │
│  │  client/sender.rs        — tx assembly & RPC send  │    │
│  │  client/solana_ctx.rs    — RPC client & env        │    │
│  │  client/client.rs        — orchestration / demo    │    │
│  └──────────────────┬─────────────────────────────────┘    │
│                     │  Signed Transactions (JSON-RPC)       │
└─────────────────────┼───────────────────────────────────────┘
                      │
          ┌───────────▼──────────────┐
          │  Solana Validator / RPC  │
          └───────────┬──────────────┘
                      │  BPF dispatch
          ┌───────────▼──────────────────────────────────┐
          │  On-Chain Program  (SBF / sBPF bytecode)     │
          │                                              │
          │  entrypoint.rs   — raw bytes → processor     │
          │  processor.rs    — instruction handlers      │
          │  instructions.rs — PensionInstruction enum   │
          │  state.rs        — PensionAccount structs    │
          │  errors.rs       — PensionError domain errs  │
          └──────────────────────────────────────────────┘
                      │  reads / writes
          ┌───────────▼──────────────┐
          │  PensionAccount accounts │
          │  (Solana AccountInfo)    │
          └──────────────────────────┘
```

**Key architectural properties:**
- Stateless program — all mutable state is stored in accounts passed to each instruction, not inside program code.
- Authority-gated writes — the `authority` public key stored inside each account acts as the access-control principal for every mutating instruction.
- No off-chain intermediaries for validation — business rules (BR-1 through BR-18) are enforced entirely inside `processor.rs`.

---

## 2. Core Components

### 2.1 On-Chain Program (`src/`)

| File | Responsibility | PRD mapping |
|------|---------------|-------------|
| `entrypoint.rs` | Declares the BPF entrypoint macro; routes raw instruction bytes to `processor::process_instruction`. | NFR-4 (immutability) |
| `instructions.rs` | `PensionInstruction` enum — all 13 instruction variants, Borsh-serialized. Acts as the program's public API surface. | F1–F8, FR-1 through FR-6 |
| `state.rs` | `PensionAccount`, `PensionStatus`, `PensionMetaData`, `YearPointsEntry`, `Relations` — Borsh-serialized account layout. Includes `serialized_size()` for rent calculation. | FR-1.2, FR-2, FR-6, BR-3 |
| `errors.rs` | `PensionError` enum mapped to `ProgramError::Custom(n)` via `thiserror`. Exhaustive domain error set. | NFR-1, BR-4 through BR-18 |
| `processor.rs` | One handler per instruction variant. Enforces ownership checks, signature checks, authority match, status guards, and business logic. Emits structured `msg!` logs. | All FR-*, NFR-1, NFR-2, NFR-3 |

### 2.2 Off-Chain Client SDK (`client/`)

| File | Responsibility |
|------|---------------|
| `instructions.rs` | Pure, side-effect-free instruction builder functions. One function per instruction variant. Assembles `AccountMeta` lists and serializes `PensionInstruction` with Borsh. |
| `sender.rs` | `send_instructions()` — builds and signs a `Transaction`, simulates to capture compute units, then submits via `send_and_confirm_transaction`. Returns `TxCost` (signature, fee, CU). |
| `solana_ctx.rs` | `create_client()` RPC factory, `resolve_program_id()` (from env or hardcoded fallback), `airdrop_sol()` with polling loop. |
| `client.rs` | Orchestration demo: creates keypairs, airdrops SOL, calls builders → sender to run the full lifecycle flow. |

---

## 3. Interfaces & Integrations

### 3.1 Program Interface (Instruction API)

The program exposes a single BPF entrypoint. Every interaction is a Solana transaction containing one or more instructions. Instruction data is **Borsh-encoded** `PensionInstruction` variants.

| Instruction | Accounts | Signers | Effect |
|-------------|----------|---------|--------|
| `InitializePensioner` | pension (writable, signer), authority (signer), system_program | authority + pension keypair | Creates account via CPI to System Program; stores initial state |
| `MarkDeceased` | pension (writable), authority (signer) | authority | Sets `status = Deceased` |
| `CalculateDuePayment` | pension (readonly) | — | Logs due lamports; no mutation |
| `AddPoints` | pension (writable), authority (signer) | authority | Appends `YearPointsEntry` |
| `GetPoints` | pension (readonly) | — | Logs points for year |
| `GetAllPoints` | pension (readonly) | — | Logs full points history |
| `StartPayout` | pension (writable), authority (signer), recipient (readonly) | authority | Sets `payout_enabled=1`, `payout_recipient` |
| `StopPayout` | pension (writable), authority (signer) | authority | Clears payout fields |
| `ChangePayoutRecipient` | pension (writable), authority (signer), new_recipient (readonly) | authority | Updates `payout_recipient` |
| `RecalculateMonthlyFromPoints` | pension (writable), authority (signer) | authority | Recomputes `monthly_payment` |
| `Contribute` | pension (writable), contributor/authority (signer), system_program | authority | CPI transfer + optional point entry |
| `StartPayoutPeriod` | pension (writable), authority (signer) | authority | Sets `payout_enabled=1` |
| `WithdrawMonthly` | pension (writable), authority (signer), pensioner (signer) | authority + pensioner | Deducts `monthly_payment` from contributions |

### 3.2 External Integrations

| Integration | Type | Notes |
|-------------|------|-------|
| **Solana System Program** | On-chain CPI | Used by `InitializePensioner` and `Contribute` to create accounts and transfer SOL. Called via `solana_program::invoke`. |
| **Solana Clock Sysvar** | On-chain read | `Clock::get()?.unix_timestamp` is used for `last_payment_timestamp` and payment calculations (FR-3.1, BR-11). |
| **Solana Rent Sysvar** | On-chain read | `Rent::get()?.minimum_balance(size)` determines the lamports required for rent exemption on account creation. |
| **Solana JSON-RPC** | Off-chain HTTP | Client SDK connects to `http://localhost:8899` (configurable via `PROGRAM_ID` env var and `create_client()`). |

### 3.3 Deployment Interface

```bash
# Build
cargo build-sbf                          # → target/deploy/insurance.so

# Deploy
solana program deploy -u localhost ./target/deploy/insurance.so

# Client
PROGRAM_ID=<deployed_id> cargo run --example client
```

---

## 4. Data Models

### 4.1 PensionStatus (state machine)

```
PrePension ──► Inactive ──► Active ──► Deceased  (terminal)
                                  └──► Moved
```

Only `Deceased` is currently enforced as terminal (BR-4). Other transitions are not yet guarded by a state machine — see Technical Risks.

```rust
enum PensionStatus { PrePension, Inactive, Active, Deceased, Moved }
```

### 4.2 PensionMetaData

| Field | Type | Business Rule |
|-------|------|---------------|
| `min_pension_age` | `u8` | FR-6.1 — configurable per plan |
| `max_pension_age` | `u8` | FR-6.1 |
| `points_per_year` | `u64` | FR-6.1 |
| `base_monthly_payment` | `u64` (lamports) | FR-3.3 — base for recalculation |
| `payment_per_point` | `u64` (lamports) | FR-3.3 — per-point multiplier |

### 4.3 PensionAccount

| Field | Type | Description | Business Rule |
|-------|------|-------------|---------------|
| `authority` | `Pubkey` | Managing entity | BR-2, BR-14 |
| `pensioner` | `Pubkey` | Pensioner wallet | FR-1.2 |
| `status` | `PensionStatus` | Lifecycle state | FR-1.3, BR-4–BR-6 |
| `date_of_birth` | `i64` | Unix timestamp | FR-1.2 |
| `metadata` | `PensionMetaData` | Plan config | FR-6.1 |
| `date_of_retirement` | `i64` | Unix timestamp | FR-1.2 |
| `date_of_death` | `i64` | Unix timestamp; 0 = alive | FR-1.2 |
| `monthly_payment` | `u64` | Lamports per month | FR-3.1, FR-3.3, BR-10 |
| `last_payment_timestamp` | `i64` | Basis for due calculation | FR-3.1, BR-11 |
| `payout_enabled` | `u8` | 0/1 flag | FR-5.1–FR-5.4, BR-17–BR-18 |
| `payout_recipient` | `Pubkey` | Active disbursement target | FR-5.1, FR-5.3 |
| `points_count` | `u8` | Populated entries (max 64) | FR-2.3, BR-7 |
| `points` | `[YearPointsEntry; 64]` | Fixed-size contribution history | FR-2.1, BR-7–BR-9 |
| `total_contributions_lamports` | `u64` | Cumulative deposits | FR-4.3, BR-13 |
| `relations` | `Relations` | Spouse + children | FR-6.2 |

Account size is computed using `PensionAccount::serialized_size()` — a Borsh round-trip on a zeroed dummy — to avoid manual byte arithmetic and padding errors (BR-3).

### 4.4 YearPointsEntry

| Field | Type | Notes |
|-------|------|-------|
| `year` | `u16` | Contribution year |
| `month` | `u16` | Contribution month (1–12) |
| `points` | `u64` | Points earned |

Uniqueness key: `(year, month)` per account (BR-8).

### 4.5 Relations

| Field | Type |
|-------|------|
| `pensioner` | `Pubkey` |
| `children` | `Vec<Pubkey>` |
| `spouse` | `Pubkey` |

> Note: `Vec<Pubkey>` in `Relations` means account size is not fully fixed. This is a risk for the current on-chain layout — see Technical Risks.

### 4.6 Payment Calculation

$$\text{months\_due} = \left\lfloor \frac{t_{\text{now}} - t_{\text{last\_payment}}}{2{,}629{,}746} \right\rfloor$$

$$\text{total\_due} = \text{months\_due} \times \text{monthly\_payment} \quad \text{(saturating\_mul)}$$

$$\text{new\_monthly} = \text{base\_lamports} + \sum \text{points} \times \text{point\_multiplier\_lamports} \quad \text{(saturating arithmetic)}$$

Where `2,629,746` seconds = 365.25 days ÷ 12 months × 86,400 seconds/day (BR-11).

---

## 5. Relevant Technologies

| Layer | Technology | Version | Rationale |
|-------|-----------|---------|-----------|
| Blockchain runtime | Solana | Mainnet-compatible | Permissionless, high-throughput, low-fee L1. No off-chain compute needed. |
| Language | Rust (edition 2021) | stable | Memory safety, zero-cost abstractions, `no_std`-compatible for BPF target. |
| Program framework | **Native Solana** (no Anchor) | — | Full control over account layout and CPI; avoids Anchor version churn; matches official documentation patterns. |
| Serialization | Borsh | 1.0 | Deterministic binary format; standard for Solana; both on-chain and client use the same schema. |
| Error handling | thiserror | 1.0 | Derives `std::error::Error` and `Display` on `PensionError`; maps cleanly to `ProgramError::Custom(n)`. |
| On-chain SDK | solana-program | 2.3.0 | Core primitives: `AccountInfo`, `Pubkey`, `Clock`, `Rent`, `invoke`, entrypoint macros. |
| Client SDK | solana-sdk, solana-client | 2.3.x | `RpcClient`, `Transaction`, `Keypair`, `Instruction` for off-chain interaction. |
| Testing | LiteSVM | 0.7.1 | In-process Solana runtime simulation — no local validator needed. Fast, deterministic. |
| System interface | solana-system-interface | 3.0.0 | Forward-compatible replacement for `solana_sdk::system_program` (migration path for deprecation). |

### Build toolchain

```toml
# Cargo.toml (current)
[dependencies]
solana-program          = "2.3.0"
solana-system-interface = "3.0.0"
borsh                   = "1.0"
thiserror               = "1.0"

[dev-dependencies]
litesvm       = "0.7.1"
solana-client = "2.3.0"
solana-sdk    = "2.3.1"
anyhow        = "1.0"

[features]
debug = []          # enables sol_log_params and sol_log_compute_units tracing
```

> `arrayref`, `digest`, `blake3` are currently listed as direct deps but are not used by program code. They should be removed to reduce the crate graph (optimization.md).

---

## 6. Assumptions

| # | Assumption | Impact if wrong |
|---|-----------|----------------|
| A-1 | A single `authority` keypair is sufficient for governance. No multi-sig or DAO is required in scope. | Would require replacing the authority model with a multi-sig PDA or Squads integration. |
| A-2 | Pension account addresses can be random keypairs for now (PDAs deferred per OQ-2). The caller is responsible for tracking account addresses. | Account discoverability breaks for external integrators without an off-chain index. |
| A-3 | `WithdrawMonthly` simulates payout by decrementing an internal counter; actual SOL transfer to the pensioner's external wallet is deferred (OQ-4). | Current implementation is not production-safe for real fund disbursement. |
| A-4 | `Relations.children` is a `Vec<Pubkey>` with a variable number of children. The account size at initialization must be large enough to accommodate the expected maximum. | Variable-length fields break the fixed-size account assumption and could cause deserialization panics if the account was not allocated with sufficient space. |
| A-5 | The Solana clock sysvar provides sufficient timestamp precision (1-second resolution) for monthly payment granularity. | Sub-second precision is not needed; monthly buckets tolerate small clock drift. |
| A-6 | The program will be deployed using the **upgradeable BPF loader** during development, allowing re-deployment. Final production posture (freeze vs. keep upgradeable) is TBD (OQ-9). | Immutable deployment prevents bug fixes; upgradeable deployment requires governance over the upgrade authority. |
| A-7 | All monetary values fit in `u64` lamports (max ~18.4 × 10¹⁸ lamports). Monthly payments and contributions are well within this range. | Saturating arithmetic protects against overflow but silently caps values; extreme inputs would yield incorrect results. |
| A-8 | The `debug` Cargo feature is only enabled in development builds. Production deploys use `cargo build-sbf` without the feature. | Enabling debug in production increases compute unit usage per instruction. |

---

## 7. Technical Risks

| # | Risk | Severity | Mitigation |
|---|------|----------|------------|
| R-1 | **`todo!()` macros in `InitializePensioner`** — `date_of_birth`, `metadata`, `date_of_retirement`, `date_of_death`, and `relations` are unresolved. The program will panic at runtime when this instruction is called. | Critical | Highest-priority fix (story-map Epic 7). Extend the instruction args and populate all fields before any testing or deployment. |
| R-2 | **Variable-length `Relations.children` (Vec)** — Borsh serializes `Vec` with a length prefix. If an account is initialized without enough space to hold future children, reallocation is not supported on Solana without `realloc`. | High | Either cap children to a fixed-size array (e.g. `[Pubkey; 8]`) or use `AccountInfo::realloc` with `zero_init=true`. Resolve OQ-6. |
| R-3 | **No PDA — accounts are not discoverable** — Using random keypairs means off-chain clients must track account addresses externally. There is no on-chain way to derive a pensioner's account address from their pubkey. | High | Migrate to PDAs (`["pension", pensioner_pubkey]`). Resolves OQ-2 and enables idempotent creation. |
| R-4 | **`WithdrawMonthly` does not transfer SOL** — The current implementation decrements `total_contributions_lamports` but does not move lamports to the pensioner's wallet. | High | Implement actual CPI transfer via `system_instruction::transfer` from the pension account (using `invoke_signed` once PDAs are adopted). Resolves OQ-4. |
| R-5 | **No authority transfer instruction** — The `authority` field is immutable after initialization. Lost or compromised authority keypair = permanently locked account. | High | Implement two-phase authority transfer (propose → accept). Resolves OQ-1. |
| R-6 | **Status state machine not enforced beyond `Deceased`** — Transitions between `PrePension`, `Inactive`, `Active`, `Moved` have no guards. Any status can be written arbitrarily. | Medium | Define and enforce a state machine in `processor.rs`. Resolves OQ-7. |
| R-7 | **Points capacity (64 entries = ~5.3 years at monthly granularity)** — An enrolled pensioner will exhaust the array within their first 6 years of monthly contributions. | Medium | Increase capacity or introduce a linked overflow account. Resolves OQ-5. |
| R-8 | **No payment cap — misconfiguration risk** — `monthly_payment` and `RecalculateMonthlyFromPoints` can set arbitrarily large values. A configuration error could drain the contribution balance in a single withdrawal. | Medium | Add a configurable `max_monthly_payment` guard in the processor. Resolves OQ-8. |
| R-9 | **Compute unit budget / no per-handler visibility** — All 13 handlers share the program's 200,000 CU budget. There is no systematic measurement of how many CUs each handler consumes. `sol_log_compute_units()` is currently called only at the **end** of each handler (behind `debug`), giving remaining CUs but not consumed delta. `process_get_all_points` with 64 entries and one `msg!` per entry is the likely hotspot. Without per-handler CU tracking, regressions are silent. | Medium | Implement three-layer CU profiling — see §8.5 Compute Unit Profiling. |
| R-10 | **Solana SDK deprecations** — `solana_sdk::commitment_config` and `solana_sdk::system_program` are flagged as deprecated in 2.3.x. Breaking changes in future SDK versions could block builds. | Low | Migrate to `solana-commitment-config` crate and `solana_system_interface::program::ID`. Gate behind `legacy-sdk` feature during transition. |

---

## 8. Build, Test & Deploy

### Build

```bash
cargo build-sbf                          # on-chain .so (SBF target)
cargo build                              # host build for tests & client
```

### Test

```bash
cargo test                               # unit + LiteSVM program tests (no validator)
cargo clippy --all-targets --all-features
```

### Deploy (local)

```bash
solana-test-validator --reset            # terminal 1
solana config set --url localhost
solana program deploy -u localhost ./target/deploy/insurance.so
PROGRAM_ID=<id> cargo run --example client
```

### Release build optimisations

```toml
# Cargo.toml
[profile.release]
codegen-units = 1
lto           = true
opt-level     = "z"
panic         = "abort"
strip         = true
```

```toml
# .cargo/config.toml
[build]
incremental = true

[target.sbf-solana-solana]
rustflags = ["-C", "lto=fat", "-C", "opt-level=z", "-C", "codegen-units=1"]
```

### 8.5 Compute Unit Profiling

#### Why it matters

Solana programs run within a per-transaction CU budget (default 200,000; max 1,400,000 with `ComputeBudgetInstruction::set_compute_unit_limit`). Exceeding the budget causes the transaction to fail with `ComputationalBudgetExceeded`. Without per-handler measurement, a new loop, an extra `msg!`, or a larger deserialized struct can silently push a hot instruction over budget in production.

The `consumed 273 of 200,000` line seen in program logs is the runtime's own final accounting line. It reflects the **total** CUs used by the entire program invocation for one instruction.

#### Layer 1 — On-chain bracketing (`profile-cu` feature)

Each handler is bracketed with a pair of `sol_log_compute_units()` calls that emit the **remaining** CU count at entry and exit. The diff gives the handler's own cost:

```rust
// processor.rs — pattern applied to every handler
pub fn process_contribute(...) -> ProgramResult {
    #[cfg(feature = "profile-cu")]
    { msg!("[cu:Contribute:enter]"); sol_log_compute_units(); }

    // ... handler body ...

    #[cfg(feature = "profile-cu")]
    { msg!("[cu:Contribute:exit]"); sol_log_compute_units(); }
    Ok(())
}
```

Build and run a targeted test with profiling enabled:

```bash
cargo test test_contribute_lamports_and_points --features profile-cu -- --nocapture 2>&1 | grep '\[cu:\|consumed'
```

The output will look like:
```
Program log: [cu:Contribute:enter]
Program consumption: 198_432 units remaining
Program log: [cu:Contribute:exit]
Program consumption: 192_811 units remaining
# → Contribute consumed ~5,621 CUs
```

#### Layer 2 — LiteSVM test assertions (`compute_units_consumed`)

Every `svm.send_transaction()` call in `src/lib.rs` returns a `TransactionResult`. The `meta.compute_units_consumed` field (a `u64`) holds the exact CUs charged for that transaction. Since all tests send single-instruction transactions, this equals the handler's cost.

Each test should assert an upper bound to catch regressions:

```rust
let result = svm.send_transaction(tx).expect("must succeed");
assert!(
    result.meta.compute_units_consumed <= 10_000,
    "Contribute consumed {} CUs — exceeds budget guard",
    result.meta.compute_units_consumed,
);
```

A dedicated `cu_budget_tests` module should run all 13 instructions on fixture accounts and print a CU table, establishing the baseline:

| Instruction | CU budget guard (TBD after first measurement) |
|-------------|-----------------------------------------------|
| `InitializePensioner` | TBD |
| `MarkDeceased` | TBD |
| `CalculateDuePayment` | TBD |
| `AddPoints` | TBD |
| `GetPoints` | TBD |
| `GetAllPoints` (64 entries) | TBD — hotspot |
| `StartPayout` | TBD |
| `StopPayout` | TBD |
| `ChangePayoutRecipient` | TBD |
| `RecalculateMonthlyFromPoints` | TBD |
| `Contribute` | TBD |
| `StartPayoutPeriod` | TBD |
| `WithdrawMonthly` | TBD |

Budget guards are set to **2× the measured baseline** to allow headroom for future field additions without hitting production failures.

#### Layer 3 — Off-chain simulation (`sender.rs`)

`sender.rs` already calls `simulate_transaction` before submitting. The simulation response includes a `units_consumed` field. This should be surfaced in the existing `TxCost` struct so every client-sent transaction reports its CU cost:

```rust
// client/sender.rs — extend TxCost
pub struct TxCost {
    pub signature: Signature,
    pub fee_lamports: u64,
    pub compute_units_consumed: u64,  // ← add this field
}
```

This makes CU cost visible at the orchestration layer without enabling any feature flags, giving production-representative numbers against a real validator.

#### Implementation checklist

- [ ] Add `profile-cu` to `[features]` in `Cargo.toml`
- [ ] Add `#[cfg(feature = "profile-cu")]` entry + exit brackets to all 13 handlers in `processor.rs`
- [ ] Add `compute_units_consumed` assertions to all 28 existing tests in `src/lib.rs`
- [ ] Add `cu_budget_tests` module with a CU report table and fill in TBD values
- [ ] Extend `TxCost` in `client/sender.rs` with `compute_units_consumed`
- [ ] Set CI to fail if any handler exceeds its CU budget guard

---

### Feature flags

| Feature | Purpose |
|---------|---------|
| `debug` | Enables `sol_log_params` and `sol_log_compute_units` tracing per instruction; logs entry/exit of each handler via structured `msg!` |
| `profile-cu` | Activates CU bracketing in every handler: emits `[cu:{HandlerName}:enter]` + `sol_log_compute_units()` at the top and `[cu:{HandlerName}:exit]` + `sol_log_compute_units()` at the bottom. Diff of the two remaining-CU values = CUs consumed by that handler. **Must not be enabled in production builds** (adds ~200 CU overhead per checkpoint call). |
| `legacy-sdk` | Keeps deprecated SDK import paths during migration to new crates |
| `client` | Gates client example deps; use `--features client` to build the example |
