use anchor_lang::prelude::*;

use crate::state::Whitelist;

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct InitializeWhitelist<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1 + 1, // 8 bytes for discriminator, 32 bytes for address pubkey, 1 byte for bool,  1 byte for bump
        seeds = [b"whitelist", user.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeWhitelist<'info> {
    pub fn initialize_whitelist(
        &mut self,
        user: Pubkey,
        bumps: InitializeWhitelistBumps,
    ) -> Result<()> {
        // Initialize the whitelist with an empty address vector
        self.whitelist.address = user.key();
        self.whitelist.active = true;
        self.whitelist.bump = bumps.whitelist;
        Ok(())
    }
}
