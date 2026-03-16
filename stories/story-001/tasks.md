# Story 001 — Technical Task Breakdown

> Derived from: [story.md](story.md)
> Status: ✅ DONE — verified 2026-03-13

---

## Task Group 1 — Instruction Definition (`src/instructions.rs`)

- [x] **TASK-1.1** Add `date_of_birth: i64`, `date_of_retirement: i64`, `metadata: PensionMetaData`, and `spouse: Pubkey` fields to the `InitializePensioner` variant of `PensionInstruction`.
- [x] **TASK-1.2** Import `PensionMetaData` into `instructions.rs` so the new fields compile.
- [x] **TASK-1.3** Add doc comments to `InitializePensioner` describing each field, whether it is required or optional, and its failure conditions (e.g. `date_of_birth == 0` → `InvalidDateOfBirth`).

---

## Task Group 2 — Domain Error (`src/errors.rs`)

- [x] **TASK-2.1** Add `InvalidDateOfBirth` variant to `PensionError` enum with a descriptive message (e.g. `"date_of_birth must be a positive Unix timestamp"`).

---

## Task Group 3 — Processor (`src/processor.rs`)

- [x] **TASK-3.1** Update the `InitializePensioner` match arm to destructure the four new fields (`date_of_birth`, `date_of_retirement`, `metadata`, `spouse`) from the instruction.
- [x] **TASK-3.2** Add a guard: if `date_of_birth <= 0` return `PensionError::InvalidDateOfBirth` before any account mutation.
- [x] **TASK-3.3** Replace the `date_of_birth: todo!()` macro with the validated parameter value.
- [x] **TASK-3.4** Replace the `date_of_retirement: todo!()` macro with the `date_of_retirement` parameter (0 = unknown).
- [x] **TASK-3.5** Replace the `metadata: todo!()` macro with the `metadata` parameter.
- [x] **TASK-3.6** Replace the `date_of_death: todo!()` macro with the literal `0` (always zero at enrollment).
- [x] **TASK-3.7** Replace the `relations: todo!()` macro with `Relations { spouse, children: vec![] }` (children always empty per scope decision).
- [x] **TASK-3.8** Set initial `status` to `PensionStatus::PrePension`.
- [x] **TASK-3.9** Verify `PensionAccount::serialized_size()` is called before `system_instruction::create_account` to allocate the correct rent-exempt space.

---

## Task Group 4 — Client Instruction Builder (`client/instructions.rs`)

- [x] **TASK-4.1** Add `date_of_birth: i64`, `date_of_retirement: i64`, `metadata: PensionMetaData`, and `spouse: Pubkey` parameters to `initialize_pensioner()`.
- [x] **TASK-4.2** Import `PensionMetaData` into `client/instructions.rs`.
- [x] **TASK-4.3** Update the `PensionInstruction::InitializePensioner { .. }` struct literal inside the builder to include all four new fields.
- [x] **TASK-4.4** Confirm account metas are unchanged: `pension_account` (writable, signer) → `authority` (signer) → `system_program` (readonly).
- [x] **TASK-4.5** Add doc comments to `initialize_pensioner()`: accounts table, all arguments, and failure conditions (`InvalidDateOfBirth`).

---

## Task Group 5 — LiteSVM Tests (`src/lib.rs`)

- [x] **TASK-5.1** Add `initialize_pensioner_tests` module gated with `#[cfg(test)]`.
- [x] **TASK-5.2** Add `setup_svm()` helper that loads `target/deploy/insurance.so` and returns `(LiteSVM, program_id)`.
- [x] **TASK-5.3** Add `build_initialize_ix()` helper inside the test module that mirrors the client builder (validates both paths share the same account-meta contract).
- [x] **TASK-5.4** `test_happy_path` (AC-1, AC-2, AC-7): airdrop ≥ 1 SOL to authority, send `InitializePensioner`, deserialise `PensionAccount`, assert every field in the AC-1 table; assert `account.lamports > 0`.
- [x] **TASK-5.5** `test_duplicate_initialization_rejected` (AC-3): send `InitializePensioner` twice with the same pension account keypair; assert the second transaction returns an error.
- [x] **TASK-5.6** `test_date_of_birth_zero_rejected` (AC-6): send `InitializePensioner` with `date_of_birth = 0`; assert the transaction fails with `PensionError::InvalidDateOfBirth`.

---

## Task Group 6 — Build & CI Verification

- [x] **TASK-6.1** Run `cargo build-sbf` and confirm exit `0` (no `todo!()` panics remaining at compile time).
- [x] **TASK-6.2** Run `cargo test initialize_pensioner` and confirm `3 passed; 0 failed`.
- [x] **TASK-6.3** Run `cargo clippy -- -D warnings` and confirm no new warnings introduced.

---

## Task Group 7 — Documentation

- [x] **TASK-7.1** Update `story-001/story.md` status to `✅ DONE` and add the Verification Report section with test output and AC traceability table.
- [x] **TASK-7.2** Confirm open items are noted: AC-4/AC-5 signer checks (runtime-enforced, no dedicated LiteSVM test) and Risk R-2 (`Vec<Pubkey>` children) tracked for a future story.
