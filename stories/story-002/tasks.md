# Story 002 — Technical Task Breakdown

> Derived from: [story.md](story.md)
> Status: 🔲 TODO

---

## Task Group 1 — Instruction Builders (`client/instructions.rs`)

- [ ] **TASK-1.1** Add `contribute` builder with doc comment, correct account metas (`pension_account` writable, `authority` signer, system program at index 2), and Borsh-serialised `PensionInstruction::Contribute { lamports, points, year }` payload.
- [ ] **TASK-1.2** Add `get_all_points` builder with doc comment, single readonly `pension_account` meta, and Borsh-serialised `PensionInstruction::GetAllPoints` payload.
- [ ] **TASK-1.3** Add `start_payout_period` builder with doc comment, `pension_account` writable + `authority` signer account metas, and Borsh-serialised `PensionInstruction::StartPayoutPeriod` payload.
- [ ] **TASK-1.4** Add `withdraw_monthly` builder with doc comment, `pension_account` writable + `authority` signer at index 1 + `pensioner` signer at index 2, and Borsh-serialised `PensionInstruction::WithdrawMonthly` payload.
- [ ] **TASK-1.5** Verify `cargo check` passes with zero warnings after adding all four builders.

---

## Task Group 2 — Shared Test Fixtures (`src/lib.rs`)

- [ ] **TASK-2.1** Add `setup_initialized_account()` helper inside the new `full_coverage_tests` module. It must: airdrop lamports to `authority`, send `InitializePensioner`, and return `(LiteSVM, program_id, authority_kp, pension_kp, pensioner_pubkey)`.
- [ ] **TASK-2.2** Add `setup_active_account()` helper that calls `setup_initialized_account()`, reads the serialised bytes from SVM, patches the `status` field to `PensionStatus::Active`, and overwrites the account via `svm.set_account()`. Return type identical to `setup_initialized_account()`.
- [ ] **TASK-2.3** Verify both helpers compile and return correct state by adding a single smoke-test that calls each and asserts the returned `status` value.

---

## Task Group 3 — Migrate `InitializePensioner` Tests

- [ ] **TASK-3.1** Move the existing 3 tests from `initialize_pensioner_tests` into `full_coverage_tests`, replacing inline setup code with `setup_initialized_account()` and replacing inline instruction builders with the `client::instructions::initialize_pensioner` builder.
- [ ] **TASK-3.2** Delete or empty `initialize_pensioner_tests` module once migration is complete, keeping just the `test` module for the empty-instruction smoke test.

---

## Task Group 4 — `MarkDeceased` Tests

- [ ] **TASK-4.1** `test_mark_deceased_happy_path`: call `mark_deceased` builder, send transaction, deserialise account, assert `status == PensionStatus::Deceased` and `date_of_death == 0`.
- [ ] **TASK-4.2** `test_mark_deceased_already_deceased`: call `mark_deceased` twice; assert the second result is `Err` containing `PensionError::AlreadyDeceased`.
- [ ] **TASK-4.3** `test_mark_deceased_invalid_authority`: send `MarkDeceased` signed by a fresh keypair (not the authority); assert result is `Err` containing `PensionError::InvalidAuthority`.

---

## Task Group 5 — `Contribute` Tests

- [ ] **TASK-5.1** `test_contribute_lamports_only`: send `Contribute { lamports: 500_000_000, points: 0, year: 0 }`, deserialise account, assert `total_contributions_lamports == 500_000_000` and `svm.get_account(pension_kp).lamports` increased by `500_000_000`.
- [ ] **TASK-5.2** `test_contribute_lamports_and_points`: send `Contribute { lamports: 100_000_000, points: 10, year: 2024 }`, assert both `total_contributions_lamports` and `points_count` incremented correctly.
- [ ] **TASK-5.3** `test_contribute_duplicate_year_rejected`: send a second `Contribute` with the same `year`/`month` combination; assert result is `Err` containing `PensionError::YearAlreadyExists`.

---

## Task Group 6 — `AddPoints` Tests

- [ ] **TASK-6.1** `test_add_points_happy_path`: send `AddPoints { year: 2023, month: 1, points: 50 }`, deserialise, assert `points_count == 1` and the stored entry has correct `year`, `month`, `points`.
- [ ] **TASK-6.2** `test_add_points_duplicate_rejected`: send `AddPoints` twice with identical `(year, month)`; assert second call returns `Err` containing `PensionError::YearAlreadyExists`.
- [ ] **TASK-6.3** `test_add_points_capacity_exceeded`: loop to insert 64 entries, assert the 65th returns `Err` containing `PensionError::PointsCapacityExceeded`.

---

## Task Group 7 — `RecalculateMonthlyFromPoints` Tests

- [ ] **TASK-7.1** `test_recalculate_monthly_from_points_happy_path`: pre-populate some points via `AddPoints`, send `RecalculateMonthlyFromPoints { base_lamports, point_multiplier_lamports }`, deserialise, assert `monthly_payment == base_lamports + Σpoints * point_multiplier_lamports` (saturating).

---

## Task Group 8 — `StartPayout` / `StopPayout` / `ChangePayoutRecipient` Tests

- [ ] **TASK-8.1** `test_start_payout_happy_path`: send `StartPayout { recipient }`, assert `payout_enabled == 1` and `payout_recipient == recipient`.
- [ ] **TASK-8.2** `test_start_payout_already_active`: call `StartPayout` twice; assert second returns `Err` containing `PensionError::PayoutAlreadyActive`.
- [ ] **TASK-8.3** `test_stop_payout_happy_path`: enable payout first, then send `StopPayout`, assert `payout_enabled == 0` and `payout_recipient == Pubkey::default()`.
- [ ] **TASK-8.4** `test_stop_payout_not_active`: send `StopPayout` on an account where payout was never started; assert `Err` containing `PensionError::PayoutNotActive`.
- [ ] **TASK-8.5** `test_change_payout_recipient_happy_path`: enable payout, send `ChangePayoutRecipient { new_recipient }`, assert `payout_recipient == new_recipient` and `payout_enabled` remains `1`.
- [ ] **TASK-8.6** `test_change_payout_recipient_not_active`: send `ChangePayoutRecipient` while payout is disabled; assert `Err` containing `PensionError::PayoutNotActive`.

---

## Task Group 9 — `StartPayoutPeriod` Tests

- [ ] **TASK-9.1** `test_start_payout_period_happy_path`: use `start_payout_period` builder, assert `payout_enabled == 1` and `payout_recipient == Pubkey::default()` (no recipient — distinguishes from `StartPayout`).
- [ ] **TASK-9.2** `test_start_payout_period_already_active`: call twice; assert second returns `Err` containing `PensionError::PayoutAlreadyActive`.

---

## Task Group 10 — `WithdrawMonthly` Tests (requires `Active` account)

- [ ] **TASK-10.1** `test_withdraw_monthly_happy_path`: use `setup_active_account()`, enable payout, set `total_contributions_lamports` above `monthly_payment` via `set_account`, send `WithdrawMonthly` signed by **both** `authority` and `pensioner`, assert `total_contributions_lamports` decreased by `monthly_payment` and `last_payment_timestamp` updated.
- [ ] **TASK-10.2** `test_withdraw_monthly_insufficient_balance`: set `total_contributions_lamports < monthly_payment`, send `WithdrawMonthly`, assert result is `Ok(())` and balance clamped (graceful path per BR-13).
- [ ] **TASK-10.3** `test_withdraw_monthly_pensioner_not_active`: use `setup_initialized_account()` (`PrePension`), enable payout via `set_account`, send `WithdrawMonthly`; assert `Err` containing `PensionError::PensionerNotActive`.
- [ ] **TASK-10.4** `test_withdraw_monthly_payout_not_enabled`: use `setup_active_account()` with payout disabled, send `WithdrawMonthly`; assert `Err` containing `PensionError::PayoutNotActive`.

---

## Task Group 11 — `CalculateDuePayment` Tests

- [ ] **TASK-11.1** `test_calculate_due_payment_happy_path`: use `setup_active_account()`, send `CalculateDuePayment`, assert result is `Ok(())`.
- [ ] **TASK-11.2** `test_calculate_due_payment_not_active`: use `setup_initialized_account()` (`PrePension`), send `CalculateDuePayment`, assert `Err` containing `PensionError::PensionerNotActive`.

---

## Task Group 12 — Configuration & Build Verification

- [ ] **TASK-12.1** Run `cargo build-sbf` and confirm exit `0` (required before LiteSVM tests can load the `.so`).
- [ ] **TASK-12.2** Run `cargo test` and confirm all tests pass with `0 failed`. No `#[allow(dead_code)]` or `#[allow(unused)]` attributes introduced to suppress warnings.
- [ ] **TASK-12.3** Run `cargo clippy -- -D warnings` and fix any new warnings introduced by this story.

---

## Task Group 13 — Documentation Updates

- [ ] **TASK-13.1** Add full doc comments to all 4 new builders in `client/instructions.rs` (accounts table, arguments, failure conditions) following the style of `initialize_pensioner`.
- [ ] **TASK-13.2** Add a module-level doc comment to `full_coverage_tests` describing what it covers and the LiteSVM dependency.
- [ ] **TASK-13.3** Update `story-002/story.md` status from `🔲 TODO` to `✅ DONE` once all ACs are met.
- [ ] **TASK-13.4** Update `story-map.md`: set Goal 1.2 test-coverage row and Epic 10 to `✅ DONE`.
