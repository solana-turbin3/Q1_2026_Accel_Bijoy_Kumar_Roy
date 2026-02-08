use anchor_lang::prelude::*;

use crate::state::Vault;

#[derive(Accounts)]
pub struct InitVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        space =  8 + Vault::INIT_SPACE,
        payer = admin,
        seeds = [b"vault"],
        bump
    )]
    pub vault: Account<'info, Vault>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitVault<'info> {
    pub fn init(&mut self, bumps: InitVaultBumps) -> Result<()> {
        self.vault.bump = bumps.vault;
        Ok(())
    }
}
