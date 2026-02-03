use anchor_lang::{prelude::*, system_program};

use crate::state::whitelist::Whitelist;

#[derive(Accounts)]
pub struct RemoveFromWhiteList<'info> {
    #[account(
        mut,
        //address = 
    )]
    pub admin: Signer<'info>,
    pub user: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"whitelist", user.key().as_ref()],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveFromWhiteList<'info> {
    pub fn remove_from_whitelist(&mut self) -> Result<()> {
        self.whitelist.active = false;
        Ok(())
    }
}
