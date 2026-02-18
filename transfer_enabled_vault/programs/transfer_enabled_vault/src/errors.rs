use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("Overflow on addition")]
    Overflow,
    #[msg("Not enough token")]
    NotEnoughToken,
    #[msg("Underflow on subtraction")]
    Underflow,
}
