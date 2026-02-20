//! Custom error types for the pension insurance program

use solana_program::program_error::ProgramError;
use thiserror::Error;

/// Custom error types for the pension insurance program
#[derive(Error, Debug, Copy, Clone)]
pub enum PensionError {
    /// The pensioner has already been marked as deceased
    #[error("The pensioner has already been marked as deceased")]
    AlreadyDeceased,
    /// Action cannot be performed as the pensioner is not active
    #[error("Action cannot be performed as the pensioner is not active")]
    PensionerNotActive,
    /// The pensioner is active but moved
    #[error("The pensioner is active but moved")]
    PensionerActiveButMoved,
    /// Invalid authority for this operation
    #[error("Invalid authority for this operation")]
    InvalidAuthority,
    /// Account not owned by program
    #[error("Account not owned by program")]
    IncorrectOwner,
    /// Points array full
    #[error("Maximum pension points entries reached")]
    PointsCapacityExceeded,
    /// Duplicate year points entry
    #[error("Points for this year already exist")]
    YearAlreadyExists,
    /// Year points not found
    #[error("Points for this year not found")]
    YearNotFound,
    /// Payout already active
    #[error("Payout already active")]
    PayoutAlreadyActive,
    /// Payout not active
    #[error("Payout not active")]
    PayoutNotActive,
}

impl From<PensionError> for ProgramError {
    fn from(e: PensionError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
