use anchor_lang::prelude::*;

#[account]
pub struct Whitelist {
    pub address: Pubkey,
    pub active: bool,
    pub bump: u8,
}
