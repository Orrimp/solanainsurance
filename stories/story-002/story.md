# Story 002 — Full LiteSVM Test Coverage + Missing Instruction Builders

> **Status:** 🔲 TODO
> **Priority:** MVP
> **Source:** Story map Activity 1 › Goal 1.2 · Activity 2–7 (all stateful handlers)
> **Linked epics:** Epic 9 (Client SDK completeness), Epic 10 (Test Coverage)
> **Linked open questions:** —
> **Linked risks:** —
> **Depends on:** Story 001 ✅ DONE

---

## User Stories

**As a `[Dev]` Developer**, I want a comprehensive LiteSVM test suite that exercises every
stateful on-chain instruction handler so that regressions are caught immediately and all
existing processor logic has verified, deterministic behaviour.

**As a `[Dev]` Developer**, I want every instruction to have a corresponding builder function
in `client/instructions.rs` so that client code never assembles account metas or serialises
instruction data by hand.

---

## Background & Context

Story 001 resolved the `todo!()` panics in `InitializePensioner` and added 3 LiteSVM tests
for that handler. All other 10 handlers (11 stateful; 2 read-only query handlers `GetPoints` /
`GetAllPoints` are explicitly out of scope) have **zero test coverage**.

Four instruction builders are also missing from `client/instructions.rs`:

| Missing builder | Processor handler |
|----------------|-------------------|
| `contribute` | `process_contribute` |
| `get_all_points` | `process_get_all_points` |
| `start_payout_period` | `process_start_payout_period` |
| `withdraw_monthly` | `process_withdraw_monthly` |

This story delivers both gaps together: builders are written first, then tests call them,
validating builder correctness as a side-effect.

---

## Scope Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Instructions covered | 11 stateful/mutating handlers | `GetPoints` and `GetAllPoints` are log-only with no state mutation; deferred |
| Error coverage | Happy paths + critical error guards | Full `PensionError` exhaustive coverage is a future story |
| Test fixtures | Shared `setup_initialized_account()` + `setup_active_account()` helpers | Eliminates per-test boilerplate; consistent starting state |
| `Active` account setup | Patch serialised state via `svm.set_account()` | No instruction to transition PrePension→Active yet (R2); test-only approach acceptable |
| Builder completeness | Add `contribute`, `get_all_points`, `start_payout_period`, `withdraw_monthly` | Unlocks SDK completeness; all other builders already exist |
| `InitializePensioner` refactor | Migrate existing 3 tests to use shared fixture + builders | Consistency — all tests use the same helpers |

---

## Changes Required

### 1. `client/instructions.rs` — Add 4 missing builders

#### `contribute`

```rust
/// Build a [`PensionInstruction::Contribute`] instruction.
///
/// # Accounts (ordered)
/// 0. `[writable]` `pension_account` — target pension account
/// 1. `[signer]`   `authority`       — insurance authority (contributor)
/// 2. `[]`         system program
///
/// # Arguments
/// * `lamports` — SOL amount to transfer into the pension account; pass `0` for points-only.
/// * `points`   — Pension points to add for `year`; pass `0` for lamports-only.
/// * `year`     — Contribution year (only used when `points > 0`).
pub fn contribute(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    lamports: u64,
    points: u64,
    year: u16,
) -> Instruction
```

#### `get_all_points`

```rust
/// Build a [`PensionInstruction::GetAllPoints`] instruction.
///
/// # Accounts (ordered)
/// 0. `[readonly]` `pension_account`
pub fn get_all_points(
    program_id: Pubkey,
    pension_account: Pubkey,
) -> Instruction
```

#### `start_payout_period`

```rust
/// Build a [`PensionInstruction::StartPayoutPeriod`] instruction.
///
/// Enables the payout phase without setting a recipient.
///
/// # Accounts (ordered)
/// 0. `[writable]` `pension_account`
/// 1. `[signer]`   `authority`
pub fn start_payout_period(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
) -> Instruction
```

#### `withdraw_monthly`

```rust
/// Build a [`PensionInstruction::WithdrawMonthly`] instruction.
///
/// Requires dual signatures: authority **and** pensioner must sign.
///
/// # Accounts (ordered)
/// 0. `[writable]` `pension_account`
/// 1. `[signer]`   `authority`
/// 2. `[signer]`   `pensioner`  — pensioner's wallet keypair
pub fn withdraw_monthly(
    program_id: Pubkey,
    pension_account: Pubkey,
    authority: Pubkey,
    pensioner: Pubkey,
) -> Instruction
```

---

### 2. `src/lib.rs` — New test module `full_coverage_tests`

#### Shared fixtures

```rust
/// Returns a ready SVM + program_id with an initialised pension account in `PrePension` state.
/// Signature: (svm, program_id, authority_kp, pension_kp, pensioner_pubkey) 
fn setup_initialized_account() -> (LiteSVM, Pubkey, Keypair, Keypair, Pubkey)

/// Returns the same as setup_initialized_account but with status patched to `Active`.
/// Uses svm.set_account() to overwrite the status byte — test-only approach.
fn setup_active_account() -> (LiteSVM, Pubkey, Keypair, Keypair, Pubkey)
```

#### Test functions per handler (11 total)

| Handler | Test function(s) |
|---------|-----------------|
| `InitializePensioner` | Migrate existing 3 tests to use `setup_svm()` + builders |
| `MarkDeceased` | `test_mark_deceased_happy_path`, `test_mark_deceased_already_deceased`, `test_mark_deceased_invalid_authority` |
| `Contribute` | `test_contribute_lamports_only`, `test_contribute_lamports_and_points`, `test_contribute_duplicate_year_rejected` |
| `AddPoints` | `test_add_points_happy_path`, `test_add_points_duplicate_rejected`, `test_add_points_capacity_exceeded` |
| `RecalculateMonthlyFromPoints` | `test_recalculate_monthly_from_points_happy_path` |
| `StartPayout` | `test_start_payout_happy_path`, `test_start_payout_already_active` |
| `StopPayout` | `test_stop_payout_happy_path`, `test_stop_payout_not_active` |
| `ChangePayoutRecipient` | `test_change_payout_recipient_happy_path`, `test_change_payout_recipient_not_active` |
| `StartPayoutPeriod` | `test_start_payout_period_happy_path`, `test_start_payout_period_already_active` |
| `WithdrawMonthly` | `test_withdraw_monthly_happy_path`, `test_withdraw_monthly_insufficient_balance`, `test_withdraw_monthly_pensioner_not_active`, `test_withdraw_monthly_payout_not_enabled` |
| `CalculateDuePayment` | `test_calculate_due_payment_happy_path`, `test_calculate_due_payment_not_active` |

---

## Acceptance Criteria

### AC-1 — Missing builders added

All four builders exist in `client/instructions.rs` and compile without warnings:

- [ ] `contribute` — correct account metas; `system_program::id()` at index 2
- [ ] `get_all_points` — readonly account meta only; no signer
- [ ] `start_payout_period` — pension account writable; authority signer
- [ ] `withdraw_monthly` — pension account writable; **two signers** (authority at index 1, pensioner at index 2)

### AC-2 — Shared test fixtures

- [ ] `setup_initialized_account()` creates a `PrePension` account and returns `(LiteSVM, program_id, authority_kp, pension_kp, pensioner_pubkey)` without panicking
- [ ] `setup_active_account()` returns an account where `state.status == PensionStatus::Active` as verified by deserialisation
- [ ] Both helpers load the program binary from `target/deploy/insurance.so`

### AC-3 — `MarkDeceased` coverage

- [ ] Happy path: status transitions to `Deceased`; `date_of_death` field is still `0` (not yet set — R2)
- [ ] `AlreadyDeceased` returned on second call to same account
- [ ] `InvalidAuthority` returned when a non-authority signer submits the instruction

### AC-4 — `Contribute` coverage

- [ ] Lamports-only: `total_contributions_lamports` increases by the contributed amount; account lamport balance increases accordingly
- [ ] Lamports + points: both `total_contributions_lamports` and `points_count` increment in one transaction
- [ ] `YearAlreadyExists` returned on duplicate `(year, month)` combination

### AC-5 — `AddPoints` coverage

- [ ] Happy path: `points_count` increments; correct `year`, `month`, `points` stored at the new index
- [ ] `YearAlreadyExists` returned for a duplicate `(year, month)`
- [ ] `PointsCapacityExceeded` returned when 64 entries are already present

### AC-6 — `RecalculateMonthlyFromPoints` coverage

- [ ] Happy path: `monthly_payment` equals `base_lamports + Σpoints * point_multiplier_lamports` (saturating); old value overwritten

### AC-7 — `StartPayout` / `StopPayout` / `ChangePayoutRecipient` coverage

- [ ] `StartPayout`: `payout_enabled == 1`; `payout_recipient` set to supplied pubkey
- [ ] `StartPayout` twice returns `PayoutAlreadyActive`
- [ ] `StopPayout`: `payout_enabled == 0`; `payout_recipient` reset to `Pubkey::default()`
- [ ] `StopPayout` on idle account returns `PayoutNotActive`
- [ ] `ChangePayoutRecipient`: `payout_recipient` updated while `payout_enabled` remains `1`
- [ ] `ChangePayoutRecipient` on idle account returns `PayoutNotActive`

### AC-8 — `StartPayoutPeriod` coverage

- [ ] Happy path: `payout_enabled == 1` (no recipient set — distinguishes from `StartPayout`)
- [ ] Second call returns `PayoutAlreadyActive`

### AC-9 — `WithdrawMonthly` coverage

- [ ] Happy path (Active + payout enabled + sufficient balance): `total_contributions_lamports` decreases by `monthly_payment`; `last_payment_timestamp` updated
- [ ] Insufficient balance: instruction returns `Ok(())`; contributions stay at balance (graceful path per BR-13)
- [ ] `PensionerNotActive` returned when account is `PrePension`
- [ ] `PayoutNotActive` returned when `payout_enabled == 0`

### AC-10 — `CalculateDuePayment` coverage

- [ ] Happy path (Active): instruction returns `Ok(())` (result is logged, not returned)
- [ ] `PensionerNotActive` returned for a `PrePension` account

### AC-11 — Test suite passes cleanly

- [ ] `cargo test` exits `0` with `0 failed` after `cargo build-sbf`
- [ ] No `#[allow(dead_code)]` or `#[allow(unused)]` suppressions added to pass tests

---

## Domain & Business Notes

### Dual-signature requirement for `WithdrawMonthly` (BR-15)

`WithdrawMonthly` requires **both** the authority keypair and the pensioner's wallet keypair to sign. The test fixture must supply both keypairs in `Transaction::new(&[&authority, &pensioner], ...)`. Omitting either must produce a `MissingRequiredSignature` error — this is tested implicitly via the happy path requiring both.

### No `Active` transition instruction (Risk R-6)

The `PrePension → Active` lifecycle transition is not implemented until Release 2. To test handlers that require `Active` status (`WithdrawMonthly`, `CalculateDuePayment`), `setup_active_account()` **directly overwrites** the serialised account data using `svm.set_account()`. This is a test-only strategy; production code must never bypass the state machine.

### `WithdrawMonthly` does not transfer SOL (Risk R-4)

The current processor only decrements `total_contributions_lamports` in account state — no CPI `system_instruction::transfer` is issued. Tests therefore assert **state field values only**, not actual lamport balance changes on the pension account. Real SOL transfer is deferred to Release 2 (Goal 5.2).

### `Contribute` account balance assertion

Unlike `WithdrawMonthly`, `Contribute` **does** CPI-transfer lamports via `system_instruction::transfer`. The test for the lamports-only path must assert both:
- `state.total_contributions_lamports` increased by the expected amount
- `svm.get_account(pension_kp).lamports` increased by the same amount

### Builder account-meta ordering must mirror processor exactly

Each builder's account list must match the `next_account_info` call order in the corresponding processor function. Any divergence will cause an `IncorrectOwner` or `MissingRequiredSignature` error at runtime rather than a compile-time error. The tests themselves serve as the contract-enforcement mechanism.

---

## Definition of Done

- [ ] All 4 missing builders added and documented in `client/instructions.rs`
- [ ] `full_coverage_tests` module added to `src/lib.rs` with all test functions listed in the table above
- [ ] `cargo build-sbf && cargo test` exits `0` with `0 failed`
- [ ] No new `todo!()` macros introduced
- [ ] Story map updated: Goal 1.2 test coverage row and Epic 10 set to `✅ DONE`
- [ ] This story status updated to `✅ DONE`

---

*Story authored: 2026-03-13*
*Author: stakeholder Q&A → `solana-ci` + `@workspace` agents*
