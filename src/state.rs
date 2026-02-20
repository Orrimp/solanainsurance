//! State definitions for the pension insurance program

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const MAX_POINTS_ENTRIES: usize = 64; // adjustable capacity

/// Configuration parameters for the pension system
#[derive(BorshSerialize, BorshDeserialize, Debug, Default, Clone, Copy)]
pub struct PensionMetaData {
    /// Minimum age for pension eligibility
    pub min_pension_age: u8,
    /// Maximum age for pension eligibility
    pub max_pension_age: u8,
    /// Points required per year of contribution
    pub points_per_year: u64,
    /// Base monthly payment in lamports
    pub base_monthly_payment: u64,
    /// Incremental payment per point in lamports
    pub payment_per_point: u64,
}

/// Status of a pension account
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum PensionStatus {
    /// Pensioner is not yet eligible for pension payments
    PrePension,
    /// Pensioner is inactive, payments on hold
    Inactive,
    /// Pensioner is active and eligible for payments
    Active,
    /// Pensioner is deceased, payments stopped
    Deceased,
    /// Pensioner has moved, payments may be redirected
    Moved,
}

/// Account data structure for storing pension information
#[derive(BorshSerialize, BorshDeserialize, Debug)]
#[derive(Copy, Clone, Default)]
pub struct YearPointsEntry {
    pub year: u16,
    pub month: u16,
    pub points: u64,
}

/// Relationship between pensioner and relative
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Relations{
    pub pensioner: Pubkey,
    pub children: Vec<Pubkey>,
    pub spouse: Pubkey,
}


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PensionAccount {
    /// The public key of the insurance company/DAO that manages this pension
    pub authority: Pubkey,
    /// The public key of the pensioner's wallet
    pub pensioner: Pubkey,
    /// Current status of the pension (Active or Deceased)
    pub status: PensionStatus,
    /// Pensioner personal data (for reference; not sensitive)
    pub date_of_birth: i64,
    /// Pension system configuration metadata
    pub metadata: PensionMetaData,
    /// Unix timestamp of retirement
    pub date_of_retirement: i64,
    /// Unix timestamp of death (0 if still alive)
    pub date_of_death: i64,
    /// Monthly payment amount in lamports (1 SOL = 1,000,000,000 lamports)
    pub monthly_payment: u64,
    /// Unix timestamp of the last payment
    pub last_payment_timestamp: i64,
    /// Whether an external payout is currently active
    pub payout_enabled: u8, // 0 = false, 1 = true
    /// Current payout recipient (valid only if payout_enabled == 1)
    pub payout_recipient: Pubkey,
    /// Number of populated point entries
    pub points_count: u8,
    /// Fixed-size array of yearly pension point entries
    pub points: [YearPointsEntry; MAX_POINTS_ENTRIES],
    /// Total lamports contributed (tracking deposits)
    pub total_contributions_lamports: u64,
    /// Relations of this pensioner to other accounts
    pub relations: Relations,
}

impl PensionAccount {
    /// Compute serialized size dynamically using Borsh (avoids manual field math & padding issues)
    pub fn serialized_size() -> usize {
        // Construct a zeroed dummy with full points capacity; length equals max serialized size.
        let pensioner = PensionAccount {
            authority: Pubkey::default(),
            pensioner: Pubkey::default(),
            status: PensionStatus::Active,
            date_of_birth: 0,
            metadata: PensionMetaData {
                min_pension_age: 0,
                max_pension_age: 0,
                points_per_year: 0,
                base_monthly_payment: 0,
                payment_per_point: 0,
            },
            relations: Relations {
                pensioner: Pubkey::default(),
                children: vec![],
                spouse: Pubkey::default(),
            },
            date_of_retirement: 0,
            date_of_death: 0,
            monthly_payment: 0,
            last_payment_timestamp: 0,
            payout_enabled: 0,
            payout_recipient: Pubkey::default(),
            points_count: 0,
            points: [YearPointsEntry::default(); MAX_POINTS_ENTRIES],
            total_contributions_lamports: 0,
        };
        borsh::to_vec(&pensioner).expect("serialize PensionAccount").len()
    }
}