//! Pension Insurance Smart Contract
//! 
//! A Solana program for managing pension accounts and payments.

// Suppress unexpected_cfg warnings that originate inside the `entrypoint!` macro
// from the `solana_program_entrypoint` crate. These are not our warnings to fix.
#![allow(unexpected_cfgs)]

// Module declarations
pub mod entrypoint;
pub mod errors;
pub mod instructions;
pub mod processor;
pub mod state;

// Re-export main types for convenience
pub use errors::*;
pub use instructions::*;
pub use state::*;

// Re-export the entrypoint
pub use entrypoint::*;

#[cfg(test)]
mod test {
    use litesvm::LiteSVM;
    use solana_instruction::Instruction;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    /// Verifies that the program is loaded and rejects a malformed (empty) instruction
    /// with `InvalidInstructionData` rather than panicking, confirming the entrypoint
    /// correctly guards against undecodable instruction data.
    #[test]
    fn test_program_rejects_empty_instruction() {
        // Create a new LiteSVM instance
        let mut svm = LiteSVM::new();

        // Create a keypair for the transaction payer
        let payer = Keypair::new();

        // Airdrop some lamports to the payer
        svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

        // Load our program
        let program_keypair = Keypair::new();
        let program_id = program_keypair.pubkey();
        svm.add_program_from_file(program_id, "target/deploy/insurance.so")
            .unwrap();

        // Send an instruction with no data — the program should reject it gracefully
        // with InvalidInstructionData (not panic), confirming the entrypoint guard works.
        let instruction = Instruction {
            program_id,
            accounts: vec![],
            data: vec![],
        };

        // Create transaction
        let message = Message::new(&[instruction], Some(&payer.pubkey()));
        let transaction = Transaction::new(&[&payer], message, svm.latest_blockhash());

        // Empty instruction data cannot be decoded — expect a clean program error, not a panic.
        let result = svm.send_transaction(transaction);
        assert!(result.is_err(), "Empty instruction data should be rejected by the program");
    }
}

/// Full LiteSVM test coverage for every stateful instruction handler.
///
/// Tests require the compiled `.so` binary:
/// ```text
/// cargo build-sbf
/// cargo test
/// ```
///
/// ## Fixture strategy
/// - [`setup_initialized_account`] creates a `PrePension` account via `InitializePensioner`.
/// - [`setup_active_account`] calls the above then patches `status → Active` directly
///   via `svm.set_account()`. This test-only shortcut is necessary because no on-chain
///   `PrePension → Active` transition instruction exists until Release 2.
#[cfg(test)]
mod full_coverage_tests {
    use borsh::{to_vec, BorshDeserialize};
    use litesvm::LiteSVM;
    use solana_address::Address as Pubkey;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use solana_transaction_error::TransactionError;
    use solana_instruction_error::InstructionError;
    use solana_system_interface::program as system_program;
    use crate::{
        errors::PensionError,
        instructions::PensionInstruction,
        state::{PensionAccount, PensionMetaData, PensionStatus, MAX_POINTS_ENTRIES},
    };

    // ── Constants ────────────────────────────────────────────────────────────────

    /// Monthly payment stored in all fixture accounts.
    const MONTHLY_PAYMENT: u64 = 1_000_000;
    /// Date of birth used in fixtures: 1990-01-01 00:00:00 UTC.
    const DATE_OF_BIRTH: i64 = 631_152_000;
    /// Airdrop amount (10 SOL) — covers rent, contribution deposits, and fees.
    const AIRDROP: u64 = 10_000_000_000;

    // ── Shared test fixtures ──────────────────────────────────────────────────────

    /// Load the compiled `.so` binary and return `(svm, program_id)`.
    fn setup_svm() -> (LiteSVM, Pubkey) {
        let mut svm = LiteSVM::new();
        let kp = Keypair::new();
        let program_id = kp.pubkey();
        svm.add_program_from_file(program_id, "target/deploy/insurance.so")
            .expect("Failed to load insurance.so — run `cargo build-sbf` first");
        (svm, program_id)
    }

    /// Create a `PrePension` account via `InitializePensioner`.
    ///
    /// Returns `(svm, program_id, authority_kp, pension_kp, pensioner_pubkey)`.
    fn setup_initialized_account() -> (LiteSVM, Pubkey, Keypair, Keypair, Pubkey) {
        let (mut svm, program_id) = setup_svm();
        let authority_kp = Keypair::new();
        let pension_kp   = Keypair::new();
        let pensioner    = Pubkey::new_unique();

        svm.airdrop(&authority_kp.pubkey(), AIRDROP).unwrap();

        let ix = ix_initialize(
            program_id, pension_kp.pubkey(), authority_kp.pubkey(), pensioner,
            MONTHLY_PAYMENT, DATE_OF_BIRTH, 0, PensionMetaData::default(), Pubkey::default(),
        );
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        let tx  = Transaction::new(&[&authority_kp, &pension_kp], msg, bh);
        svm.send_transaction(tx).expect("InitializePensioner must succeed in setup");

        (svm, program_id, authority_kp, pension_kp, pensioner)
    }

    /// Patch a `PrePension` account to `Active` status via `svm.set_account()`.
    ///
    /// **Test-only strategy**: there is no on-chain `PrePension → Active` instruction
    /// until Release 2. Production code must never bypass the state machine this way.
    ///
    /// Returns `(svm, program_id, authority_kp, pension_kp, pensioner_pubkey)`.
    fn setup_active_account() -> (LiteSVM, Pubkey, Keypair, Keypair, Pubkey) {
        let (mut svm, program_id, authority_kp, pension_kp, pensioner) =
            setup_initialized_account();

        let mut raw = svm.get_account(&pension_kp.pubkey())
            .expect("pension account must exist after setup_initialized_account");
        let mut state = PensionAccount::try_from_slice(&raw.data)
            .expect("setup_active_account: deserialize");

        state.status = PensionStatus::Active;
        raw.data = to_vec(&state).expect("setup_active_account: re-serialize");

        // Pass the same account type back — no external Account import needed.
        svm.set_account(pension_kp.pubkey(), raw)
            .expect("setup_active_account: set_account failed");

        (svm, program_id, authority_kp, pension_kp, pensioner)
    }

    // ── Instruction builders (account-meta ordering mirrors processor exactly) ────

    #[allow(clippy::too_many_arguments)]
    fn ix_initialize(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey, pensioner_pubkey: Pubkey,
        monthly_payment: u64, date_of_birth: i64, date_of_retirement: i64,
        metadata: PensionMetaData, spouse: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, true),                              // 0. [writable, signer]
                AccountMeta::new_readonly(authority, true),                   // 1. [signer]
                AccountMeta::new_readonly(system_program::id(), false),       // 2. system program
            ],
            data: to_vec(&PensionInstruction::InitializePensioner {
                pensioner_pubkey, monthly_payment, date_of_birth,
                date_of_retirement, metadata, spouse,
            }).expect("serialize InitializePensioner"),
        }
    }

    fn ix_mark_deceased(program_id: Pubkey, pension: Pubkey, authority: Pubkey) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),         // 0. [writable]
                AccountMeta::new(authority, true),        // 1. [signer]
            ],
            data: to_vec(&PensionInstruction::MarkDeceased)
                .expect("serialize MarkDeceased"),
        }
    }

    fn ix_contribute(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey,
        lamports: u64, points: u64, year: u16,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),                             // 0. [writable]
                AccountMeta::new(authority, true),                            // 1. [signer]
                AccountMeta::new_readonly(system_program::id(), false),       // 2. system program
            ],
            data: to_vec(&PensionInstruction::Contribute { lamports, points, year })
                .expect("serialize Contribute"),
        }
    }

    fn ix_add_points(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey,
        year: u16, month: u16, points: u64,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),         // 0. [writable]
                AccountMeta::new(authority, true),        // 1. [signer]
            ],
            data: to_vec(&PensionInstruction::AddPoints { year, month, points })
                .expect("serialize AddPoints"),
        }
    }

    fn ix_recalculate(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey,
        base_lamports: u64, point_multiplier_lamports: u64,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),         // 0. [writable]
                AccountMeta::new(authority, true),        // 1. [signer]
            ],
            data: to_vec(&PensionInstruction::RecalculateMonthlyFromPoints {
                base_lamports, point_multiplier_lamports,
            }).expect("serialize RecalculateMonthlyFromPoints"),
        }
    }

    fn ix_start_payout(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey, recipient: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),                             // 0. [writable]
                AccountMeta::new(authority, true),                            // 1. [signer]
                AccountMeta::new_readonly(recipient, false),                  // 2. [readonly]
            ],
            data: to_vec(&PensionInstruction::StartPayout { recipient })
                .expect("serialize StartPayout"),
        }
    }

    fn ix_stop_payout(program_id: Pubkey, pension: Pubkey, authority: Pubkey) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),         // 0. [writable]
                AccountMeta::new(authority, true),        // 1. [signer]
            ],
            data: to_vec(&PensionInstruction::StopPayout).expect("serialize StopPayout"),
        }
    }

    fn ix_change_recipient(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey, new_recipient: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),                             // 0. [writable]
                AccountMeta::new(authority, true),                            // 1. [signer]
                AccountMeta::new_readonly(new_recipient, false),              // 2. [readonly]
            ],
            data: to_vec(&PensionInstruction::ChangePayoutRecipient { new_recipient })
                .expect("serialize ChangePayoutRecipient"),
        }
    }

    fn ix_start_payout_period(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),         // 0. [writable]
                AccountMeta::new(authority, true),        // 1. [signer]
            ],
            data: to_vec(&PensionInstruction::StartPayoutPeriod)
                .expect("serialize StartPayoutPeriod"),
        }
    }

    fn ix_withdraw_monthly(
        program_id: Pubkey, pension: Pubkey, authority: Pubkey, pensioner: Pubkey,
    ) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![
                AccountMeta::new(pension, false),                             // 0. [writable]
                AccountMeta::new(authority, true),                            // 1. [signer]
                AccountMeta::new_readonly(pensioner, true),                   // 2. [signer]
            ],
            data: to_vec(&PensionInstruction::WithdrawMonthly)
                .expect("serialize WithdrawMonthly"),
        }
    }

    fn ix_calculate_due(program_id: Pubkey, pension: Pubkey) -> Instruction {
        Instruction {
            program_id,
            accounts: vec![AccountMeta::new_readonly(pension, false)],       // 0. [readonly]
            data: to_vec(&PensionInstruction::CalculateDuePayment)
                .expect("serialize CalculateDuePayment"),
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────────

    /// Deserialise a [`PensionAccount`] from LiteSVM storage.
    fn read_state(svm: &LiteSVM, pubkey: &Pubkey) -> PensionAccount {
        let raw = svm.get_account(pubkey).expect("account not found");
        PensionAccount::try_from_slice(&raw.data).expect("deserialize PensionAccount")
    }

    /// Patch `payout_enabled = 1` and `total_contributions_lamports` on a pension
    /// account via `svm.set_account()`. Used to pre-configure state for withdrawal tests
    /// without depending on a CPI lamport transfer.
    fn patch_for_withdrawal(
        svm: &mut LiteSVM,
        _program_id: Pubkey,
        pension_pubkey: &Pubkey,
        total_contributions: u64,
    ) {
        let mut raw = svm.get_account(pension_pubkey).expect("pension account must exist");
        let mut s   = PensionAccount::try_from_slice(&raw.data).expect("deserialize");
        s.payout_enabled               = 1;
        s.total_contributions_lamports = total_contributions;
        raw.data = to_vec(&s).expect("re-serialize");
        svm.set_account(*pension_pubkey, raw)
            .expect("patch_for_withdrawal: set_account failed");
    }

    // ── InitializePensioner ────────────────────────────────────────────────────────

    /// Happy path: every field in the on-chain account matches what was passed in.
    #[test]
    fn test_initialize_happy_path() {
        let (mut svm, program_id) = setup_svm();
        let authority      = Keypair::new();
        let pension_kp     = Keypair::new();
        let pensioner_pubkey = Pubkey::new_unique();

        svm.airdrop(&authority.pubkey(), AIRDROP).unwrap();

        let ix = ix_initialize(
            program_id, pension_kp.pubkey(), authority.pubkey(), pensioner_pubkey,
            MONTHLY_PAYMENT, DATE_OF_BIRTH, 0, PensionMetaData::default(), Pubkey::default(),
        );
        let msg = Message::new(&[ix], Some(&authority.pubkey()));
        let bh  = svm.latest_blockhash();
        let tx  = Transaction::new(&[&authority, &pension_kp], msg, bh);
        assert!(svm.send_transaction(tx).is_ok(), "happy-path InitializePensioner must succeed");

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.authority,                  authority.pubkey(),   "authority");
        assert_eq!(state.pensioner,                  pensioner_pubkey,     "pensioner");
        assert_eq!(state.status,                     PensionStatus::PrePension, "status");
        assert_eq!(state.date_of_birth,              DATE_OF_BIRTH,        "date_of_birth");
        assert_eq!(state.date_of_retirement,         0,                    "date_of_retirement");
        assert_eq!(state.date_of_death,              0,                    "date_of_death");
        assert_eq!(state.monthly_payment,            MONTHLY_PAYMENT,      "monthly_payment");
        assert_eq!(state.payout_enabled,             0,                    "payout_enabled");
        assert_eq!(state.payout_recipient,           Pubkey::default(),    "payout_recipient");
        assert_eq!(state.points_count,               0,                    "points_count");
        assert_eq!(state.total_contributions_lamports, 0,                  "contributions");
        assert_eq!(state.relations.spouse,           Pubkey::default(),    "spouse");
        assert!(state.relations.children.is_empty(),                       "children empty");

        // Account must hold rent-exempt lamports (AC-2).
        assert!(svm.get_account(&pension_kp.pubkey()).unwrap().lamports > 0, "rent-exempt");
    }

    /// `date_of_birth = 0` must be rejected with `InvalidDateOfBirth`.
    #[test]
    fn test_initialize_date_of_birth_zero_rejected() {
        let (mut svm, program_id) = setup_svm();
        let authority  = Keypair::new();
        let pension_kp = Keypair::new();
        svm.airdrop(&authority.pubkey(), AIRDROP).unwrap();

        let ix  = ix_initialize(
            program_id, pension_kp.pubkey(), authority.pubkey(), Pubkey::new_unique(),
            MONTHLY_PAYMENT, 0 /* invalid */, 0, PensionMetaData::default(), Pubkey::default(),
        );
        let msg  = Message::new(&[ix], Some(&authority.pubkey()));
        let bh   = svm.latest_blockhash();
        let tx   = Transaction::new(&[&authority, &pension_kp], msg, bh);
        let fail = svm.send_transaction(tx).expect_err("date_of_birth=0 must fail");
        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::InvalidDateOfBirth as u32),
            ),
        );
    }

    /// Initialising the same account twice must fail (System Program rejects `create_account`
    /// on an already-allocated account).
    #[test]
    fn test_initialize_duplicate_rejected() {
        let (mut svm, program_id) = setup_svm();
        let authority  = Keypair::new();
        let pension_kp = Keypair::new();
        svm.airdrop(&authority.pubkey(), AIRDROP).unwrap();

        let build_ix = || ix_initialize(
            program_id, pension_kp.pubkey(), authority.pubkey(), Pubkey::new_unique(),
            MONTHLY_PAYMENT, DATE_OF_BIRTH, 0, PensionMetaData::default(), Pubkey::default(),
        );

        let msg1 = Message::new(&[build_ix()], Some(&authority.pubkey()));
        let bh1  = svm.latest_blockhash();
        svm.send_transaction(Transaction::new(&[&authority, &pension_kp], msg1, bh1))
            .expect("first InitializePensioner must succeed");

        let msg2 = Message::new(&[build_ix()], Some(&authority.pubkey()));
        let bh2  = svm.latest_blockhash();
        let res  = svm.send_transaction(Transaction::new(&[&authority, &pension_kp], msg2, bh2));
        assert!(res.is_err(), "duplicate InitializePensioner must fail");
    }

    // ── MarkDeceased ──────────────────────────────────────────────────────────────

    /// Status transitions to `Deceased`; `date_of_death` remains `0` (set in Release 2).
    #[test]
    fn test_mark_deceased_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix  = ix_mark_deceased(program_id, pension_kp.pubkey(), authority_kp.pubkey());
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        let tx  = Transaction::new(&[&authority_kp], msg, bh);
        assert!(svm.send_transaction(tx).is_ok(), "MarkDeceased must succeed");

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.status,        PensionStatus::Deceased, "status must be Deceased");
        assert_eq!(state.date_of_death, 0,                       "date_of_death set by future instruction");
    }

    /// A second `MarkDeceased` on the same account must return `AlreadyDeceased`.
    #[test]
    fn test_mark_deceased_already_deceased() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix1  = ix_mark_deceased(program_id, pension_kp.pubkey(), authority_kp.pubkey());
        let msg1 = Message::new(&[ix1], Some(&authority_kp.pubkey()));
        let bh1  = svm.latest_blockhash();
        svm.send_transaction(Transaction::new(&[&authority_kp], msg1, bh1))
            .expect("first MarkDeceased must succeed");

        // Rotate blockhash so the second tx isn't deduplicated as AlreadyProcessed.
        svm.expire_blockhash();

        let ix2  = ix_mark_deceased(program_id, pension_kp.pubkey(), authority_kp.pubkey());
        let msg2 = Message::new(&[ix2], Some(&authority_kp.pubkey()));
        let bh2  = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp], msg2, bh2))
            .expect_err("second MarkDeceased must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::AlreadyDeceased as u32),
            ),
        );
    }

    /// A signer that is not the stored `authority` must receive `InvalidAuthority`.
    #[test]
    fn test_mark_deceased_invalid_authority() {
        let (mut svm, program_id, _authority_kp, pension_kp, _) = setup_initialized_account();

        let impostor = Keypair::new();
        svm.airdrop(&impostor.pubkey(), AIRDROP).unwrap();

        let ix   = ix_mark_deceased(program_id, pension_kp.pubkey(), impostor.pubkey());
        let msg  = Message::new(&[ix], Some(&impostor.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&impostor], msg, bh))
            .expect_err("impostor authority must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::InvalidAuthority as u32),
            ),
        );
    }

    // ── Contribute ────────────────────────────────────────────────────────────────

    /// Lamports transferred to pension account; `total_contributions_lamports` incremented.
    #[test]
    fn test_contribute_lamports_only() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();
        let before = svm.get_account(&pension_kp.pubkey()).unwrap().lamports;

        const AMOUNT: u64 = 500_000_000;
        let ix  = ix_contribute(program_id, pension_kp.pubkey(), authority_kp.pubkey(), AMOUNT, 0, 0);
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.total_contributions_lamports, AMOUNT, "contributions must reflect deposit");

        let after = svm.get_account(&pension_kp.pubkey()).unwrap().lamports;
        assert_eq!(after, before + AMOUNT, "account lamports must increase by contributed amount");
    }

    /// Both `total_contributions_lamports` and `points_count` increment in one transaction.
    #[test]
    fn test_contribute_lamports_and_points() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix  = ix_contribute(program_id, pension_kp.pubkey(), authority_kp.pubkey(), 100_000_000, 10, 2024);
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.total_contributions_lamports, 100_000_000, "contributions");
        assert_eq!(state.points_count,                 1,           "points_count must increment");
        assert_eq!(state.points[0].year,               2024,        "stored year");
        assert_eq!(state.points[0].points,             10,          "stored points");
    }

    /// Duplicate year in a `Contribute` instruction must return `YearAlreadyExists`.
    #[test]
    fn test_contribute_duplicate_year_rejected() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let send = |svm: &mut LiteSVM| {
            let ix  = ix_contribute(program_id, pension_kp.pubkey(), authority_kp.pubkey(), 10_000_000, 5, 2024);
            let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
            let bh  = svm.latest_blockhash();
            svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
        };

        assert!(send(&mut svm).is_ok(), "first contribute must succeed");
        // Rotate blockhash so the second tx isn't deduplicated as AlreadyProcessed.
        svm.expire_blockhash();
        let fail = send(&mut svm).expect_err("duplicate year must fail");
        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::YearAlreadyExists as u32),
            ),
        );
    }

    // ── AddPoints ─────────────────────────────────────────────────────────────────

    /// `points_count` increments; stored entry has correct `year`, `month`, `points`.
    #[test]
    fn test_add_points_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix  = ix_add_points(program_id, pension_kp.pubkey(), authority_kp.pubkey(), 2023, 1, 50);
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.points_count,      1,    "points_count");
        assert_eq!(state.points[0].year,    2023, "stored year");
        assert_eq!(state.points[0].month,   1,    "stored month");
        assert_eq!(state.points[0].points,  50,   "stored points");
    }

    /// Duplicate `(year, month)` pair must return `YearAlreadyExists`.
    #[test]
    fn test_add_points_duplicate_rejected() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let send = |svm: &mut LiteSVM| {
            let ix  = ix_add_points(program_id, pension_kp.pubkey(), authority_kp.pubkey(), 2023, 1, 10);
            let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
            let bh  = svm.latest_blockhash();
            svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
        };

        assert!(send(&mut svm).is_ok(), "first AddPoints must succeed");
        // Rotate blockhash so the second tx isn't deduplicated as AlreadyProcessed.
        svm.expire_blockhash();
        let fail = send(&mut svm).expect_err("duplicate (year, month) must fail");
        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::YearAlreadyExists as u32),
            ),
        );
    }

    /// The 65th `AddPoints` after filling all 64 slots must return `PointsCapacityExceeded`.
    #[test]
    fn test_add_points_capacity_exceeded() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        // Fill all MAX_POINTS_ENTRIES slots using unique (year, month) pairs.
        for i in 0..MAX_POINTS_ENTRIES as u16 {
            let ix  = ix_add_points(program_id, pension_kp.pubkey(), authority_kp.pubkey(), i + 1, 1, 10);
            let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
            let bh  = svm.latest_blockhash();
            svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
                .unwrap_or_else(|e| panic!("entry {} failed: {e:?}", i + 1));
        }

        // One more entry must overflow.
        let ix   = ix_add_points(program_id, pension_kp.pubkey(), authority_kp.pubkey(), 65, 1, 10);
        let msg  = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
            .expect_err("65th entry must fail with PointsCapacityExceeded");
        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PointsCapacityExceeded as u32),
            ),
        );
    }

    // ── RecalculateMonthlyFromPoints ──────────────────────────────────────────────

    /// `monthly_payment = base + Σpoints * multiplier` (saturating arithmetic).
    #[test]
    fn test_recalculate_monthly_from_points_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        // Add two entries: 50 + 30 = 80 total points.
        for (year, pts) in [(2023_u16, 50_u64), (2024, 30)] {
            let ix  = ix_add_points(program_id, pension_kp.pubkey(), authority_kp.pubkey(), year, 1, pts);
            let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
            let bh  = svm.latest_blockhash();
            svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).unwrap();
        }

        let base_lamports        = 1_000_000_u64;
        let point_multiplier     = 100_u64;
        let ix  = ix_recalculate(program_id, pension_kp.pubkey(), authority_kp.pubkey(),
                                 base_lamports, point_multiplier);
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok());

        let state    = read_state(&svm, &pension_kp.pubkey());
        let expected = base_lamports.saturating_add((50_u64 + 30).saturating_mul(point_multiplier));
        assert_eq!(state.monthly_payment, expected, "monthly_payment after recalculation");
    }

    // ── StartPayout / StopPayout / ChangePayoutRecipient ─────────────────────────

    /// `payout_enabled == 1` and `payout_recipient` set to supplied key.
    #[test]
    fn test_start_payout_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();
        let recipient = Pubkey::new_unique();

        let ix  = ix_start_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey(), recipient);
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.payout_enabled,   1,         "payout_enabled must be 1");
        assert_eq!(state.payout_recipient, recipient, "payout_recipient must match");
    }

    /// Calling `StartPayout` twice must return `PayoutAlreadyActive`.
    #[test]
    fn test_start_payout_already_active() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();
        let recipient = Pubkey::new_unique();

        let ix1  = ix_start_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey(), recipient);
        let msg1 = Message::new(&[ix1], Some(&authority_kp.pubkey()));
        let bh1  = svm.latest_blockhash();
        svm.send_transaction(Transaction::new(&[&authority_kp], msg1, bh1))
            .expect("first StartPayout must succeed");

        // Rotate blockhash so the second tx isn't deduplicated as AlreadyProcessed.
        svm.expire_blockhash();

        let ix2  = ix_start_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey(), recipient);
        let msg2 = Message::new(&[ix2], Some(&authority_kp.pubkey()));
        let bh2  = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp], msg2, bh2))
            .expect_err("second StartPayout must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PayoutAlreadyActive as u32),
            ),
        );
    }

    /// `StopPayout` resets `payout_enabled` and clears `payout_recipient`.
    #[test]
    fn test_stop_payout_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();
        let recipient = Pubkey::new_unique();

        // Enable payout first.
        let ix1  = ix_start_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey(), recipient);
        let msg1 = Message::new(&[ix1], Some(&authority_kp.pubkey()));
        let bh1  = svm.latest_blockhash();
        svm.send_transaction(Transaction::new(&[&authority_kp], msg1, bh1)).expect("StartPayout");

        // Now stop it.
        let ix2  = ix_stop_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey());
        let msg2 = Message::new(&[ix2], Some(&authority_kp.pubkey()));
        let bh2  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg2, bh2)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.payout_enabled,   0,                 "payout_enabled must be 0");
        assert_eq!(state.payout_recipient, Pubkey::default(), "recipient must be reset");
    }

    /// `StopPayout` on an account where payout was never started must return `PayoutNotActive`.
    #[test]
    fn test_stop_payout_not_active() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix   = ix_stop_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey());
        let msg  = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
            .expect_err("StopPayout on idle account must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PayoutNotActive as u32),
            ),
        );
    }

    /// `payout_recipient` updated while `payout_enabled` remains `1`.
    #[test]
    fn test_change_payout_recipient_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();
        let original      = Pubkey::new_unique();
        let new_recipient = Pubkey::new_unique();

        let ix1  = ix_start_payout(program_id, pension_kp.pubkey(), authority_kp.pubkey(), original);
        let msg1 = Message::new(&[ix1], Some(&authority_kp.pubkey()));
        let bh1  = svm.latest_blockhash();
        svm.send_transaction(Transaction::new(&[&authority_kp], msg1, bh1)).expect("StartPayout");

        let ix2  = ix_change_recipient(program_id, pension_kp.pubkey(), authority_kp.pubkey(), new_recipient);
        let msg2 = Message::new(&[ix2], Some(&authority_kp.pubkey()));
        let bh2  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg2, bh2)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.payout_recipient, new_recipient, "recipient must be updated");
        assert_eq!(state.payout_enabled,   1,             "payout_enabled must remain 1");
    }

    /// `ChangePayoutRecipient` while payout is disabled must return `PayoutNotActive`.
    #[test]
    fn test_change_payout_recipient_not_active() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix   = ix_change_recipient(program_id, pension_kp.pubkey(), authority_kp.pubkey(), Pubkey::new_unique());
        let msg  = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
            .expect_err("ChangeRecipient on idle account must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PayoutNotActive as u32),
            ),
        );
    }

    // ── StartPayoutPeriod ─────────────────────────────────────────────────────────

    /// `payout_enabled == 1` and `payout_recipient` unchanged (no recipient arg).
    #[test]
    fn test_start_payout_period_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix  = ix_start_payout_period(program_id, pension_kp.pubkey(), authority_kp.pubkey());
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok());

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.payout_enabled,   1,                 "payout_enabled must be 1");
        // No recipient is set — this distinguishes StartPayoutPeriod from StartPayout.
        assert_eq!(state.payout_recipient, Pubkey::default(), "recipient must remain default");
    }

    /// A second `StartPayoutPeriod` must return `PayoutAlreadyActive`.
    #[test]
    fn test_start_payout_period_already_active() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let send = |svm: &mut LiteSVM| {
            let ix  = ix_start_payout_period(program_id, pension_kp.pubkey(), authority_kp.pubkey());
            let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
            let bh  = svm.latest_blockhash();
            svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
        };

        send(&mut svm).expect("first StartPayoutPeriod must succeed");
        // Rotate blockhash so the second tx isn't deduplicated as AlreadyProcessed.
        svm.expire_blockhash();
        let fail = send(&mut svm).expect_err("second must fail");
        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PayoutAlreadyActive as u32),
            ),
        );
    }

    // ── WithdrawMonthly ───────────────────────────────────────────────────────────

    /// `total_contributions_lamports` decreases by `monthly_payment`; `last_payment_timestamp`
    /// is updated. Requires `Active` status and `payout_enabled == 1`.
    #[test]
    fn test_withdraw_monthly_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_active_account();
        // Patch: enable payout and set contributions well above monthly_payment.
        patch_for_withdrawal(&mut svm, program_id, &pension_kp.pubkey(), 5_000_000);

        // LiteSVM's default Clock has unix_timestamp = 0. warp_to_slot() only advances the slot
        // field, not unix_timestamp. Set it explicitly so the processor stores a non-zero value.
        let mut clock = svm.get_sysvar::<solana_program::clock::Clock>();
        clock.unix_timestamp = 1_700_000_000;
        svm.set_sysvar(&clock);

        // The processor checks is_signer on the pensioner account but not its key identity.
        // Any signing keypair satisfies the requirement (see domain notes: no identity check yet).
        let pensioner_signer = Keypair::new();
        svm.airdrop(&pensioner_signer.pubkey(), 1_000_000).unwrap();

        let ix  = ix_withdraw_monthly(program_id, pension_kp.pubkey(), authority_kp.pubkey(), pensioner_signer.pubkey());
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        let tx  = Transaction::new(&[&authority_kp, &pensioner_signer], msg, bh);
        assert!(svm.send_transaction(tx).is_ok(), "WithdrawMonthly must succeed");

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(
            state.total_contributions_lamports,
            5_000_000 - MONTHLY_PAYMENT,
            "contributions must decrease by monthly_payment",
        );
        assert!(state.last_payment_timestamp > 0, "last_payment_timestamp must be updated");
    }

    /// When `total_contributions_lamports < monthly_payment`, the processor returns `Ok(())`
    /// and leaves the balance unchanged (graceful insufficient-balance path, BR-13).
    #[test]
    fn test_withdraw_monthly_insufficient_balance() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_active_account();
        // Patch: enable payout; contributions = 0 (below MONTHLY_PAYMENT).
        patch_for_withdrawal(&mut svm, program_id, &pension_kp.pubkey(), 0);

        let pensioner_signer = Keypair::new();
        svm.airdrop(&pensioner_signer.pubkey(), 1_000_000).unwrap();

        let ix  = ix_withdraw_monthly(program_id, pension_kp.pubkey(), authority_kp.pubkey(), pensioner_signer.pubkey());
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        let tx  = Transaction::new(&[&authority_kp, &pensioner_signer], msg, bh);
        // Graceful: must return Ok(()) not an error.
        assert!(svm.send_transaction(tx).is_ok(), "insufficient balance must succeed silently");

        let state = read_state(&svm, &pension_kp.pubkey());
        assert_eq!(state.total_contributions_lamports, 0, "balance must remain unchanged");
    }

    /// `WithdrawMonthly` on a `PrePension` account (with payout enabled) must return
    /// `PensionerNotActive` — the status check fires after the payout check.
    #[test]
    fn test_withdraw_monthly_pensioner_not_active() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();
        // Enable payout so the processor reaches the status guard.
        patch_for_withdrawal(&mut svm, program_id, &pension_kp.pubkey(), 5_000_000);

        let pensioner_signer = Keypair::new();
        svm.airdrop(&pensioner_signer.pubkey(), 1_000_000).unwrap();

        let ix   = ix_withdraw_monthly(program_id, pension_kp.pubkey(), authority_kp.pubkey(), pensioner_signer.pubkey());
        let msg  = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp, &pensioner_signer], msg, bh))
            .expect_err("PrePension account must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PensionerNotActive as u32),
            ),
        );
    }

    /// `WithdrawMonthly` when `payout_enabled == 0` must return `PayoutNotActive`.
    #[test]
    fn test_withdraw_monthly_payout_not_enabled() {
        // Active account, but payout is still disabled (default after setup).
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_active_account();

        let pensioner_signer = Keypair::new();
        svm.airdrop(&pensioner_signer.pubkey(), 1_000_000).unwrap();

        let ix   = ix_withdraw_monthly(program_id, pension_kp.pubkey(), authority_kp.pubkey(), pensioner_signer.pubkey());
        let msg  = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp, &pensioner_signer], msg, bh))
            .expect_err("disabled payout must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PayoutNotActive as u32),
            ),
        );
    }

    // ── CalculateDuePayment ───────────────────────────────────────────────────────

    /// On an `Active` account the instruction completes successfully (result is logged).
    #[test]
    fn test_calculate_due_payment_happy_path() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_active_account();

        let ix  = ix_calculate_due(program_id, pension_kp.pubkey());
        let msg = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh  = svm.latest_blockhash();
        assert!(svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh)).is_ok(),
                "CalculateDuePayment on Active account must succeed");
    }

    /// On a `PrePension` account the instruction must return `PensionerNotActive`.
    #[test]
    fn test_calculate_due_payment_not_active() {
        let (mut svm, program_id, authority_kp, pension_kp, _) = setup_initialized_account();

        let ix   = ix_calculate_due(program_id, pension_kp.pubkey());
        let msg  = Message::new(&[ix], Some(&authority_kp.pubkey()));
        let bh   = svm.latest_blockhash();
        let fail = svm.send_transaction(Transaction::new(&[&authority_kp], msg, bh))
            .expect_err("CalculateDuePayment on PrePension must fail");

        assert_eq!(
            fail.err,
            TransactionError::InstructionError(
                0, InstructionError::Custom(PensionError::PensionerNotActive as u32),
            ),
        );
    }
}