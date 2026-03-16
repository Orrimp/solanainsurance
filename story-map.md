# Pension Insurance Smart Contract — Story Map

> Derived from [prd-requirements.md](prd-requirements.md) and [technical-solution.md](technical-solution.md).
> Structured as a User Story Map: activities form the backbone, goals sit beneath each activity, and stories are arranged into horizontal priority slices (releases).

---

## How to Read This Map

```
ACTIVITY 1          ACTIVITY 2          ACTIVITY 3  ...     ← Backbone (what users do)
  └─ Goal 1.1         └─ Goal 2.1         └─ Goal 3.1
       Story               Story               Story         ← Release 1 (MVP) slice
       Story               Story               Story         ← Release 2 slice
       Story               Story               Story         ← Release 3 / Future slice
```

**User roles referenced:**  `[Auth]` = Pension Authority  ·  `[Pen]` = Pensioner  ·  `[Dev]` = Developer/Integrator  ·  `[Aud]` = Auditor/Regulator

---

## Implementation Status Legend

| Tag | Meaning |
|-----|---------|
| ✅ DONE | Implemented and functional |
| ⚠️ PARTIAL | Code exists but incomplete (e.g. `todo!()` macros) |
| 🔲 TODO | Not yet started |

---

## Backbone — User Activities (End-to-End Workflow)

```
┌──────────────────┐  ┌──────────────────────┐  ┌────────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐  ┌──────────────────────┐
│  1. Set Up       │  │  2. Enrol Pensioner   │  │  3. Record             │  │  4. Manage          │  │  5. Disburse        │  │  6. Monitor &       │  │  7. Govern           │
│     the Program  │  │                       │  │     Contributions      │  │     Payments        │  │     Pension         │  │     Audit           │  │     the Program      │
└──────────────────┘  └──────────────────────┘  └────────────────────────┘  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘  └──────────────────────┘
```

---

## Activity 1 — Set Up the Program

**Goal:** Deploy a working on-chain program that the authority can immediately use to manage pension accounts.

### Goal 1.1 — Build and deploy the program artifact

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Dev]`, I can build the program with `cargo build-sbf` so that a deployable `.so` artifact is produced. | Dev | ✅ DONE | `crate-type = ["cdylib", "lib"]` in Cargo.toml |
| MVP | As `[Dev]`, I can deploy the program to a local validator so that end-to-end testing is possible. | Dev | ✅ DONE | `solana program deploy -u localhost` |
| MVP | As `[Dev]`, I can set `PROGRAM_ID` via environment variable so that the client SDK resolves the correct deployed address. | Dev | ✅ DONE | `solana_ctx::resolve_program_id()` |
| R2 | As `[Dev]`, I can build the program with a release profile (LTO, strip, opt-level=z) so that the deploy artifact is minimised for lower transaction fees. | Dev | 🔲 TODO | Release profile not yet added to Cargo.toml |
| R2 | As `[Dev]`, I can feature-gate client dependencies so that `cargo build-sbf` does not pull in `solana-client` and `solana-sdk`. | Dev | 🔲 TODO | `required-features = ["client"]` pattern |
| Future | As `[Dev]`, I can run a CI pipeline that builds the program, runs tests, and exercises the client example in separate parallel jobs. | Dev | 🔲 TODO | |

### Goal 1.2 — Establish a testable baseline

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Dev]`, I can run `cargo test` without a live validator using LiteSVM so that feedback is fast and deterministic. | Dev | ✅ DONE | LiteSVM wired in `lib.rs` test module |
| MVP | As `[Dev]`, I can run `cargo clippy --all-targets --all-features` without errors so that code quality gates are met. | Dev | 🔲 TODO | Clippy not yet configured with `-Dwarnings` |
| R2 | As `[Dev]`, I can enable the `debug` feature to get per-instruction `sol_log_params` and `sol_log_compute_units` output so that I can trace instruction execution during development. | Dev | ✅ DONE | `#[cfg(feature = "debug")]` guards in processor |

---

## Activity 2 — Enrol a Pensioner

**Goal:** The authority creates an on-chain pension account that fully represents a real pensioner, including personal data, plan configuration, and family relations.

### Goal 2.1 — Create a new pension account

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can call `InitializePensioner` with a pensioner pubkey and monthly payment so that a new on-chain account is created and funded for rent exemption. | Auth | ⚠️ PARTIAL | CPI account creation works; `date_of_birth`, `metadata`, `date_of_retirement`, `date_of_death`, `relations` fields have `todo!()` — program panics at runtime (Risk R-1) |
| MVP | As `[Auth]`, I can supply `date_of_birth`, `date_of_retirement`, and plan metadata during initialization so that the account captures the complete pensioner profile. | Auth | 🔲 TODO | Requires extending instruction args and processor |
| MVP | As `[Auth]`, I can supply the pensioner's family relations (spouse, children) at initialization so that future survivor-benefit routing is possible. | Auth | 🔲 TODO | `Relations` struct exists; `Vec<Pubkey>` needs fixed-size or realloc approach (Risk R-2) |
| MVP | As `[Auth]`, I receive a clear error if I try to initialize the same account address twice so that double-enrollment is prevented. | Auth | ✅ DONE | System Program rejects duplicate account creation |
| R2 | As `[Auth]`, I can derive the pension account address deterministically from the pensioner's pubkey (PDA) so that I don't need to track random keypairs externally. | Auth | 🔲 TODO | Risk R-3 / OQ-2; requires `invoke_signed` and seed convention |
| R2 | As `[Dev]`, I can call an instruction-builder function to construct `InitializePensioner` with all fields so that transaction assembly is centralised and type-safe. | Dev | ⚠️ PARTIAL | Builder exists but missing new fields (7.3) |

### Goal 2.2 — Configure the pension plan

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can set the initial monthly payment in lamports during enrollment so that the pensioner's base entitlement is recorded on-chain. | Auth | ✅ DONE | `monthly_payment` field in init instruction |
| R2 | As `[Auth]`, I can set `min_pension_age`, `max_pension_age`, `points_per_year`, and `payment_per_point` during enrollment so that the plan's rules are embedded in the account. | Auth | 🔲 TODO | `PensionMetaData` struct exists; not yet passed to processor |

---

## Activity 3 — Record Contributions

**Goal:** The authority records both financial contributions (SOL deposits) and pension points earned per contribution period, building up an auditable history that determines payment entitlements.

### Goal 3.1 — Deposit funds into a pension account

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can call `Contribute` with a lamport amount so that SOL is transferred on-chain to the pension account and the contribution balance increases. | Auth | ✅ DONE | CPI to System Program; `total_contributions_lamports` updated |
| MVP | As `[Auth]`, I can see the updated `total_contributions_lamports` after contributing so that I have an auditable cumulative deposit record. | Auth | ✅ DONE | Field serialized in account state |
| R2 | As `[Dev]`, I can call an instruction-builder for `Contribute` so that the client SDK assembles the correct account metas. | Dev | 🔲 TODO | Builder not yet in `client/instructions.rs` |

### Goal 3.2 — Assign pension points per period

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can call `AddPoints(year, month, points)` so that a contribution period's points are recorded in the pension account. | Auth | ✅ DONE | Appends `YearPointsEntry` to fixed array |
| MVP | As `[Auth]`, I receive a `YearAlreadyExists` error if I try to add points for a (year, month) that already has an entry so that double-counting is prevented. | Auth | ✅ DONE | BR-8 |
| MVP | As `[Auth]`, I receive a `PointsCapacityExceeded` error when the 64-entry array is full so that a clear signal is given before data is lost. | Auth | ✅ DONE | BR-7 |
| MVP | As `[Auth]`, I can contribute lamports and assign points in a single `Contribute` instruction so that related operations are atomic. | Auth | ✅ DONE | `Contribute` with `points > 0` path |
| R2 | As `[Auth]`, I can add points for more than 64 periods so that pensioners with long contribution histories are fully supported. | Auth | 🔲 TODO | Requires increased capacity or overflow account (OQ-5) |

---

## Activity 4 — Manage Payments

**Goal:** The authority and pensioner can always determine the correct payment amount, and the monthly payment can be recalculated as the pensioner's points history grows.

### Goal 4.1 — Calculate due payments

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]` or `[Pen]`, I can call `CalculateDuePayment` so that the number of full months elapsed since the last payment is computed and logged. | Auth / Pen | ✅ DONE | `SECONDS_IN_MONTH = 2,629,746`; `msg!` output |
| MVP | As `[Auth]` or `[Pen]`, I receive a `PensionerNotActive` error when calculating due payment for a non-Active account so that calculations are only performed on valid accounts. | Auth / Pen | ✅ DONE | BR-5 |
| MVP | As `[Dev]`, I can call the `calculate_due_payment` instruction builder so that the read-only query is constructed with proper account metas. | Dev | ✅ DONE | Builder in `client/instructions.rs` |

### Goal 4.2 — Recalculate monthly payment from points

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can call `RecalculateMonthlyFromPoints(base_lamports, point_multiplier)` so that `monthly_payment` is updated to reflect the pensioner's accumulated points. | Auth | ✅ DONE | `new_monthly = base + Σpoints * multiplier` (saturating) |
| R2 | As `[Auth]`, I am prevented from setting a `monthly_payment` above a configured maximum so that misconfiguration cannot drain the contribution balance in a single withdrawal. | Auth | 🔲 TODO | OQ-8; max cap guard not yet implemented (Risk R-8) |
| R2 | As `[Dev]`, I can call a `recalculate_monthly_from_points` builder so that the client SDK can assemble this instruction without hardcoding account metas. | Dev | 🔲 TODO | Builder missing in `client/instructions.rs` |

---

## Activity 5 — Disburse Pension Payouts

**Goal:** The authority configures where payments go and, together with the pensioner, authorises monthly withdrawals that reduce the held balance.

### Goal 5.1 — Configure payout routing

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can call `StartPayout(recipient)` to designate a payout recipient and enable disbursements so that the pensioner's entitlement begins flowing. | Auth | ✅ DONE | Sets `payout_enabled=1`, `payout_recipient` |
| MVP | As `[Auth]`, I can call `StopPayout` to suspend disbursements so that payments can be halted (e.g. pending review). | Auth | ✅ DONE | Clears payout fields |
| MVP | As `[Auth]`, I can call `ChangePayoutRecipient(new)` while payout is active so that the beneficiary can be updated without stopping and restarting the payout. | Auth | ✅ DONE | BR-18 guard applied |
| MVP | As `[Auth]`, I receive `PayoutAlreadyActive` if I try to start a payout that is already running, and `PayoutNotActive` if I try to stop or change one that is not running. | Auth | ✅ DONE | BR-17, BR-18 |
| MVP | As `[Auth]`, I can call `StartPayoutPeriod` to enable the withdrawal phase independently of setting a recipient so that the payout lifecycle can be managed in stages. | Auth | ✅ DONE | |
| R2 | As `[Dev]`, I can call instruction builders for `StartPayout`, `StopPayout`, `ChangePayoutRecipient`, and `StartPayoutPeriod` from the client SDK. | Dev | ⚠️ PARTIAL | `start_payout` builder exists; others missing |

### Goal 5.2 — Process monthly withdrawals

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]` + `[Pen]` (dual signed), I can call `WithdrawMonthly` so that `monthly_payment` is deducted from `total_contributions_lamports` and `last_payment_timestamp` is updated. | Auth + Pen | ✅ DONE | Simulated deduction — no actual SOL transfer yet (Risk R-4) |
| MVP | As `[Auth]`, I receive a graceful log message (not an error) when the contribution balance is insufficient for a withdrawal so that I can investigate and top up before retrying. | Auth | ✅ DONE | BR-13; returns `Ok(())` with `msg!` |
| MVP | As `[Pen]`, I must co-sign every withdrawal so that no payment can be issued without my consent. | Pen | ✅ DONE | Dual-signature enforced (BR-15) |
| R2 | As `[Pen]`, the SOL is actually transferred to my wallet (or the designated payout recipient) when I withdraw, not just tracked internally. | Pen | 🔲 TODO | Requires CPI `system_instruction::transfer` from pension PDA (OQ-4 / Risk R-4) |
| R2 | As `[Dev]`, I can call a `withdraw_monthly` instruction builder so that the client SDK constructs the dual-signer transaction correctly. | Dev | 🔲 TODO | |

---

## Activity 6 — Monitor & Audit

**Goal:** Any stakeholder can query account state and points history to verify the pension's current position or audit past activity.

### Goal 6.1 — Query points history

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, `[Pen]`, or `[Aud]`, I can call `GetPoints(year)` so that the points for a specific year are logged and verifiable. | All | ✅ DONE | Logs or returns `YearNotFound` |
| MVP | As `[Auth]`, `[Pen]`, or `[Aud]`, I can call `GetAllPoints` so that the complete contribution history is logged in sequence. | All | ✅ DONE | Iterates up to `points_count` entries |
| R2 | As `[Dev]`, I can deserialise a `PensionAccount` off-chain via RPC `getAccountInfo` so that any client can read full account state without issuing an instruction. | Dev | ✅ DONE | Borsh schema shared between program and client |
| R2 | As `[Dev]`, I can call `get_year_points` and `get_all_points` instruction builders so that read-only query transactions are assembled correctly. | Dev | ✅ DONE | Builders in `client/instructions.rs` |

### Goal 6.2 — Verify transaction logs for audit

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Aud]`, every mutating instruction emits a structured `msg!` log containing all relevant field values so that on-chain activity can be replayed and verified. | Aud | ✅ DONE | NFR-2; all handlers emit `msg!` |
| R2 | As `[Aud]`, a hash of the serialised state is logged after each mutation so that tampering with account data off-chain can be detected. | Aud | 🔲 TODO | Technical-solution section 7 / Risk; OQ future |

---

## Activity 7 — Manage Lifecycle Events

**Goal:** The authority records significant life events (death, relocation) that affect whether payments continue and where they flow.

### Goal 7.1 — Record a pensioner's death

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| MVP | As `[Auth]`, I can call `MarkDeceased` so that the pension account status transitions to `Deceased` and no further payments can be calculated or withdrawn. | Auth | ✅ DONE | BR-4 terminal state enforced |
| MVP | As `[Auth]`, I receive `AlreadyDeceased` if I call `MarkDeceased` on an account already in that state so that idempotent calls are safely rejected. | Auth | ✅ DONE | BR-4; AC-2 |
| MVP | As `[Auth]`, I receive `InvalidAuthority` if a non-authority signer attempts to mark a pensioner as deceased so that unauthorised state transitions are blocked. | Auth | ✅ DONE | BR-14 |
| R2 | As `[Auth]`, I can record the `date_of_death` timestamp when marking a pensioner as deceased so that the exact date is auditable on-chain. | Auth | 🔲 TODO | `date_of_death` field exists in state; not yet set in `MarkDeceased` handler |
| R2 | As `[Auth]`, the remaining contribution balance after death is claimable by the designated beneficiary so that funds are not permanently locked. | Auth | 🔲 TODO | OQ-3; beneficiary claim instruction needed |

### Goal 7.2 — Manage account status transitions

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| R2 | As `[Auth]`, I can transition an account from `PrePension` to `Inactive` to `Active` following a defined state machine so that the lifecycle is explicit and auditable. | Auth | 🔲 TODO | State machine not enforced; only `Deceased` has a guard (Risk R-6 / OQ-7) |
| R2 | As `[Auth]`, I can record that a pensioner has moved (`Moved` status) so that payment routing can be updated before payments resume. | Auth | 🔲 TODO | `Moved` variant exists in `PensionStatus`; no transition instruction |
| Future | As `[Auth]`, I can reactivate a `Moved` account once the new address is confirmed so that legitimate cases of address change are supported. | Auth | 🔲 TODO | OQ-7 |

---

## Activity 7 — Govern the Program

**Goal:** The authority can securely transfer control of a pension account or the deployed program, and the system can be evolved without breaking existing accounts.

### Goal 8.1 — Transfer account authority

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| R2 | As `[Auth]`, I can propose an authority transfer so that a new keypair is designated as the next authority. | Auth | 🔲 TODO | Two-phase commit pattern; OQ-1 / Risk R-5 |
| R2 | As the incoming authority, I can accept a pending authority transfer so that the `authority` field is atomically updated only when I confirm. | Auth | 🔲 TODO | Prevents accidental lock-out |
| Future | As `[Auth]`, I can revoke a pending authority transfer so that a mistaken proposal can be cancelled before acceptance. | Auth | 🔲 TODO | |

### Goal 8.2 — Evolve the program safely

| Priority | Story | Role | Status | Notes |
|----------|-------|------|--------|-------|
| R2 | As `[Dev]`, instruction variants have explicit Borsh discriminant tags so that adding new variants does not break existing serialised data. | Dev | 🔲 TODO | Currently relies on Borsh default encoding |
| R2 | As `[Dev]`, the `PensionAccount` struct reserves `N` bytes for future fields so that account size does not need to change on the next version. | Dev | 🔲 TODO | No `reserved` bytes currently |
| Future | As `[Dev]`, the program and client are in separate workspace crates so that client dependency changes do not invalidate the on-chain program build cache. | Dev | 🔲 TODO | Workspace split deferred |

---

## Release Slices Summary

| Release | Scope | User Value |
|---------|-------|-----------|
| **MVP (now)** | Fix `InitializePensioner` `todo!()` panics (R-1), complete full initialization fields, add LiteSVM test coverage for all existing instructions | Program is functional end-to-end; all acceptance criteria can be verified |
| **Release 2** | PDA account addressing (R-3), actual SOL transfer on withdrawal (R-4), authority transfer (R-5), state machine enforcement (R-6), points capacity increase (R-5), payment cap (R-8), CI pipeline | Production-safe; fraud-resistant; accounts are discoverable without an external index |
| **Future** | Workspace split, versioned instructions, reserved state bytes, off-chain indexer, multi-sig governance, beneficiary claim on death | Scalable, upgradeable, compliant with enterprise governance requirements |

---

## Implementation Status Legend

| Tag | Meaning |
|-----|---------|
| ✅ DONE | Implemented and functional in current codebase |
| ⚠️ PARTIAL | Code exists but incomplete (e.g. `todo!()` macros, missing fields) |
| 🔲 TODO | Not yet started |

---

## Previous Epic Tracker (for reference)

| Epic | Title | Status |
|------|-------|--------|
| 1 | Program Foundation | ✅ DONE |
| 2 | Pension Account Lifecycle | ⚠️ PARTIAL — `InitializePensioner` panics due to `todo!()` |
| 3 | Pension Points | ✅ DONE |
| 4 | Payment Calculations | ✅ DONE |
| 5 | Contributions | ✅ DONE |
| 6 | Payout Management | ⚠️ PARTIAL — withdrawal is simulated, not real SOL transfer |
| 7 | Complete InitializePensioner | 🔲 TODO — MVP blocker |
| 8 | PDA-Based Account Addressing | 🔲 TODO — Release 2 |
| 9 | Client SDK completeness | ⚠️ PARTIAL |
| 10 | Test Coverage | ⚠️ PARTIAL — blocked by Epic 7 |
| 11 | Build & DevOps | ⚠️ PARTIAL — `run_local.sh` exists; release profile, Clippy, CI missing |
| 12 | Security Hardening | ⚠️ PARTIAL — ownership/auth checks done; authority transfer missing |
| 13 | Forward Compatibility | 🔲 TODO — Future |


