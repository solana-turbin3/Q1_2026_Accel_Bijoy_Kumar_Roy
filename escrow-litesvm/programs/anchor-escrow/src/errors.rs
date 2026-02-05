use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Deadline not reached")]
    DeadlineNotReached,
}
