# Pension Insurance Smart Contract — Product Requirements

## Vision

A blockchain-based pension insurance system on Solana that automates pension lifecycle management — from enrollment and contributions through payment calculations and disbursement — providing transparency, auditability, and trustless operation for insurance companies and pensioners.

---

## Goals

1. **Automate pension lifecycle on-chain** — enrollment, contribution tracking, payment calculation, and disbursement are handled by a single Solana program with no off-chain intermediaries for core logic.
2. **Provide a transparent, auditable record** — every state change (account creation, status transition, contribution, withdrawal) is recorded immutably on the Solana ledger with structured logs.
3. **Enforce business rules deterministically** — authority checks, status guards, points capacity limits, and payment formulas are embedded in program code, removing human discretion from rule enforcement.
4. **Enable points-based pension plans** — support flexible plan designs where monthly payments derive from accumulated contribution points and configurable multipliers.
5. **Offer a production-grade reference implementation** — demonstrate idiomatic native Solana program patterns (no Anchor) suitable for real-world deployment after security audit.

## Non-Goals

- **Token issuance** — the program does not mint or manage SPL tokens; all values are denominated in native SOL (lamports).
- **Fiat on/off-ramp** — currency conversion between SOL and fiat is out of scope; the system operates entirely within the Solana ecosystem.
- **KYC / identity verification** — pensioner identity is represented by a public key; real-world identity binding is handled externally.
- **Multi-program orchestration** — the current scope is a single program; cross-program composability (e.g. with a governance DAO or oracle) is deferred.
- **Mobile / web front-end** — no UI is provided; interaction happens via CLI, client SDK, or RPC.
- **Regulatory compliance automation** — the program provides audit data but does not enforce jurisdiction-specific pension regulations.
- **Account migration / upgrade tooling** — program upgrades and state migrations are not automated in the current scope.

---

## Target User Groups

### Primary

| Group | Description | Key Interactions |
|-------|-------------|------------------|
| **Pension Authority (Insurance Company / DAO)** | Entity that manages the pension programme. Holds the authority keypair. | Initialize accounts, contribute funds, manage points, trigger payouts, mark deceased. |
| **Pensioner** | Individual enrolled in the pension system. Holds a personal keypair. | Co-sign withdrawals, query account state, view points and due payments. |

### Secondary

| Group | Description | Key Interactions |
|-------|-------------|------------------|
| **Payout Recipient (Beneficiary)** | Wallet designated to receive disbursements — may be the pensioner, spouse, or child. | Receive SOL payouts. |
| **System Integrator / Developer** | Builds tooling or UIs on top of the program. | Use client SDK, deserialize accounts via RPC, construct transactions. |
| **Auditor / Regulator** | Reviews on-chain records for compliance. | Read transaction logs, verify account state history. |

---

## Core Features

| # | Feature | Description |
|---|---------|-------------|
| F1 | **Pension Account Management** | Create, store, and manage pension accounts with rich state (authority, pensioner, status, metadata, relations, contribution balance). |
| F2 | **Status Lifecycle** | Support five statuses (PrePension → Inactive → Active → Deceased &#124; Moved) with authority-controlled, guarded transitions. |
| F3 | **Points Accumulation** | Record per-year/month contribution points (up to 64 entries), query individual years or full history. |
| F4 | **Payment Calculation** | Compute due payments based on elapsed time (30.4375-day Gregorian month) and recalculate monthly amount from points. |
| F5 | **Contributions** | Accept SOL deposits into pension accounts with optional simultaneous points assignment; track cumulative total. |
| F6 | **Payout Management** | Start/stop payouts, designate or change recipient, enable payout periods, and process dual-signed monthly withdrawals. |
| F7 | **Configurable Metadata** | Store per-account parameters (min/max age, points-per-year, base payment, payment-per-point) for flexible plan design. |
| F8 | **Family Relations** | Record spouse and children public keys for future survivor-benefit routing. |

---

## Domain Constraints & Business Rules

### Account Rules
- **BR-1**: A pension account is owned by the on-chain program. Only the program may read/write its data.
- **BR-2**: The `authority` field is set at initialization and currently cannot be changed (no transfer instruction yet).
- **BR-3**: Account size is fixed at initialization based on `PensionAccount::serialized_size()` (includes 64 point slots).

### Status Transitions
- **BR-4**: `Deceased` is a terminal state — no further status changes are permitted once set.
- **BR-5**: `CalculateDuePayment` only executes for `Active` accounts; other statuses return `PensionerNotActive`.
- **BR-6**: `WithdrawMonthly` requires `Active` status and `payout_enabled == 1`.

### Points
- **BR-7**: Points are stored in a fixed-size array of 64 entries. Exceeding capacity returns `PointsCapacityExceeded`.
- **BR-8**: A (year, month) pair is unique per account. Duplicates return `YearAlreadyExists`.
- **BR-9**: Only the authority may add points.

### Financial
- **BR-10**: All monetary values are in lamports (u64). One SOL = 1,000,000,000 lamports.
- **BR-11**: Monthly payment calculation uses `SECONDS_IN_MONTH = 2,629,746` (365.25 days / 12 months * 86,400 seconds).
- **BR-12**: All arithmetic uses saturating operations — overflow silently caps at `u64::MAX` rather than panicking.
- **BR-13**: Withdrawal is rejected (no-op with log) if `total_contributions_lamports < monthly_payment`.

### Authorization
- **BR-14**: Every mutating instruction (except read-only queries) requires the authority's signature.
- **BR-15**: `WithdrawMonthly` requires dual signatures: authority **and** pensioner.
- **BR-16**: Account ownership (`pension_account.owner == program_id`) is validated before any deserialization.

### Payout
- **BR-17**: `StartPayout` and `StartPayoutPeriod` fail with `PayoutAlreadyActive` if payout is already enabled.
- **BR-18**: `StopPayout` and `ChangePayoutRecipient` fail with `PayoutNotActive` if payout is disabled.

---

## Acceptance Criteria

### AC-1 Pension Account Initialization
- Given a valid authority keypair and pensioner pubkey, when `InitializePensioner` is called, then a new on-chain account is created with the correct authority, pensioner, `Active` status, specified monthly payment, and the current clock timestamp as `last_payment_timestamp`.
- Given an already-initialized pension account address, when `InitializePensioner` is called again, then the transaction fails.

### AC-2 Mark Deceased
- Given an `Active` pension account, when the authority calls `MarkDeceased`, then status becomes `Deceased` and the log contains the pensioner's pubkey.
- Given a `Deceased` pension account, when `MarkDeceased` is called, then the instruction returns `AlreadyDeceased`.
- Given a valid pension account, when a non-authority signer calls `MarkDeceased`, then the instruction returns `InvalidAuthority`.

### AC-3 Calculate Due Payment
- Given an `Active` account where 2 full months have elapsed since `last_payment_timestamp`, when `CalculateDuePayment` is called, then the log reports `2 * monthly_payment` lamports.
- Given an `Active` account where less than one month has elapsed, then the log reports 0 lamports.
- Given a non-`Active` account, then the instruction returns `PensionerNotActive`.

### AC-4 Points Management
- Given an account with 0 points, when `AddPoints(year=2025, month=6, points=100)` is called, then `points_count` becomes 1 and the entry is stored at index 0.
- Given an account that already has an entry for (2025, 6), when `AddPoints(year=2025, month=6, ...)` is called, then the instruction returns `YearAlreadyExists`.
- Given an account with `points_count == 64`, when `AddPoints` is called, then the instruction returns `PointsCapacityExceeded`.
- When `GetPoints(year=2025)` is called, then the log contains the points value for that year.
- When `GetAllPoints` is called, then the log contains every stored (year, month, points) triple.

### AC-5 Contributions
- Given a valid authority, when `Contribute(lamports=1_000_000, points=50, year=2025)` is called, then `total_contributions_lamports` increases by 1,000,000, a point entry is added, and the SOL balance of the pension account increases.
- Given `points=0`, then no point entry is added and no duplicate check is performed.

### AC-6 Payment Recalculation
- Given an account with total points = 500, when `RecalculateMonthlyFromPoints(base=1000, multiplier=10)` is called, then `monthly_payment` becomes `1000 + 500*10 = 6000`.

### AC-7 Payout Lifecycle
- When `StartPayout(recipient)` is called, then `payout_enabled` becomes 1 and `payout_recipient` is set.
- When `StopPayout` is called on an active payout, then `payout_enabled` becomes 0 and `payout_recipient` resets.
- When `ChangePayoutRecipient(new)` is called on an active payout, then `payout_recipient` updates.
- Calling `StartPayout` when payout is already active returns `PayoutAlreadyActive`.
- Calling `StopPayout` when payout is inactive returns `PayoutNotActive`.

### AC-8 Monthly Withdrawal
- Given `payout_enabled == 1`, `status == Active`, and `total_contributions >= monthly_payment`, when authority + pensioner co-sign `WithdrawMonthly`, then `total_contributions` decreases by `monthly_payment` and `last_payment_timestamp` updates.
- Given insufficient contributions, then no deduction occurs and a log message is emitted.

### AC-9 Security
- Any instruction called with an account not owned by the program returns `IncorrectOwner`.
- Any authority-guarded instruction called without a valid signature returns `MissingRequiredSignature`.
- Any authority-guarded instruction called by a signer that does not match the stored authority returns `InvalidAuthority`.

---

## Open Questions

| # | Question | Impact | Status |
|---|----------|--------|--------|
| OQ-1 | How should authority transfer work? Single-step or two-phase commit with pending/accept pattern? | Affects governance model and security (Epic 12). | Open |
| OQ-2 | Should the program use PDAs instead of random keypairs for pension accounts? This enables deterministic address derivation but changes the creation flow. | Affects client SDK, account discoverability, and idempotent creation (Epic 8). | Open |
| OQ-3 | What happens to remaining contribution balance when a pensioner is marked Deceased? Should it be claimable by a beneficiary, returned to authority, or locked? | Affects payout rules and beneficiary logic. | Open |
| OQ-4 | Should `WithdrawMonthly` perform an actual SOL transfer (CPI) or continue with the current simulated balance deduction? | Affects real-world usability and security surface. | Open |
| OQ-5 | Is 64 the right capacity for points entries? With monthly granularity, 64 entries covers ~5.3 years. Should this be increased or made dynamic? | Affects account size and rent cost. | Open |
| OQ-6 | Should `Relations` (spouse, children) be stored in the same account or in separate linked accounts for privacy and size management? | Affects account layout, rent costs, and data access patterns. | Open |
| OQ-7 | How should status transitions beyond `Deceased` work? Is there a defined state machine (e.g. PrePension → Active, Active → Moved → Active)? | Affects processor validation logic. Currently only Deceased is enforced. | Open |
| OQ-8 | Should there be a maximum monthly payment cap or contribution ceiling to prevent misconfiguration? | Affects financial safety rules. | Open |
| OQ-9 | How is the program upgraded in production? Upgradeable BPF loader or immutable deploy with versioned instruction set? | Affects deployment strategy and forward compatibility. | Open |
| OQ-10 | Should the `Contribute` instruction accept contributions from non-authority wallets (e.g. employer matching or third-party deposits)? | Affects authorization model; currently restricted to authority only. | Open |

---

## Stakeholders

### FR-1 Pension Account Lifecycle

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-1.1 | The system shall allow an authority to create (initialize) a new pension account for a pensioner. | Must |
| FR-1.2 | Each pension account shall store: authority, pensioner pubkey, status, date of birth, retirement date, death date, monthly payment amount, last payment timestamp, contribution balance, pension metadata, and family relations. | Must |
| FR-1.3 | A pension account shall support the following statuses: **PrePension**, **Inactive**, **Active**, **Deceased**, **Moved**. | Must |
| FR-1.4 | Only the designated authority may transition an account to the **Deceased** state. | Must |
| FR-1.5 | Once marked **Deceased**, the account shall not be reactivated (irreversible transition). | Must |
| FR-1.6 | Payment calculations shall only be performed for accounts in **Active** status. | Must |

### FR-2 Pension Points System

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-2.1 | The authority shall be able to add pension points for a specific year and month. | Must |
| FR-2.2 | Duplicate entries for the same year+month shall be rejected. | Must |
| FR-2.3 | The system shall support up to 64 year/month point entries per account. | Must |
| FR-2.4 | Any party shall be able to query points for a specific year. | Should |
| FR-2.5 | Any party shall be able to query the full points history. | Should |

### FR-3 Payment Calculations

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-3.1 | The system shall calculate due payments based on elapsed time since last payment using a 30.4375-day month (Gregorian average). | Must |
| FR-3.2 | Calculations shall use saturating arithmetic to prevent overflow. | Must |
| FR-3.3 | The authority shall be able to recalculate monthly payment based on accumulated points: `new_monthly = base_lamports + total_points * point_multiplier_lamports`. | Must |

### FR-4 Contributions

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-4.1 | The authority shall be able to contribute SOL (lamports) to a pension account via on-chain transfer. | Must |
| FR-4.2 | Contributions may optionally include pension points for a given year. | Should |
| FR-4.3 | The account shall track total cumulative contributions. | Must |

### FR-5 Payout Management

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-5.1 | The authority shall be able to start a payout to a designated recipient address. | Must |
| FR-5.2 | The authority shall be able to stop an active payout. | Must |
| FR-5.3 | The authority shall be able to change the payout recipient while payout is active. | Must |
| FR-5.4 | The authority shall be able to start a payout period (enable withdrawals). | Must |
| FR-5.5 | Monthly withdrawals shall deduct from the contribution balance and update the last-payment timestamp. Both authority and pensioner must sign. | Must |
| FR-5.6 | Withdrawals shall be rejected if the contribution balance is insufficient. | Must |

### FR-6 Pension Metadata & Configuration

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-6.1 | Each account shall store configurable metadata: min/max pension age, points per year, base monthly payment, and payment per point. | Should |
| FR-6.2 | The account shall store family relations (children, spouse) for future payout routing or survivor benefits. | Should |

## Non-Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| NFR-1 | **Security** — All mutating instructions must verify account ownership and authority signatures before state changes. | Must |
| NFR-2 | **Auditability** — Every state-changing instruction shall emit structured log messages with relevant fields. | Must |
| NFR-3 | **Accuracy** — Financial calculations must use industry-standard constants and safe arithmetic. | Must |
| NFR-4 | **Immutability** — Deployed program logic is tamper-proof on the Solana blockchain. | Must |
| NFR-5 | **Availability** — The program is available 24/7 as long as the Solana network operates. | Must |
| NFR-6 | **Performance** — Instructions should execute within Solana's default compute-unit budget. | Should |
| NFR-7 | **Testability** — All instructions must be testable via LiteSVM without requiring a live validator. | Should |

## Glossary

| Term | Definition |
|------|------------|
| **Lamports** | Smallest unit of SOL (1 SOL = 1,000,000,000 lamports). |
| **PDA** | Program Derived Address — deterministic account address derived from seeds. |
| **CPI** | Cross-Program Invocation — calling the System Program to create accounts or transfer SOL. |
| **Rent exemption** | Minimum lamport balance required so Solana does not garbage-collect an account. |
| **Pension points** | Abstract units accumulated per contribution period that influence monthly payment amount. |
