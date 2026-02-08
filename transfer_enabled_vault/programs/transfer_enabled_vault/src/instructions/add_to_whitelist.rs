use anchor_lang::prelude::*;

use crate::state::{UserDb, Whitelist};

#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct AddToWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + Whitelist::INIT_SPACE,
        seeds = [b"whitelist", user.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    #[account(
        init,
        payer = admin,
        space = 8 + UserDb::INIT_SPACE,
        seeds = [b"user_db", user.key().as_ref()],
        bump
    )]
    pub user_db: Account<'info, UserDb>,

    pub system_program: Program<'info, System>,
}

impl<'info> AddToWhiteList<'info> {
    pub fn add_user(&mut self, user: Pubkey, bumps: AddToWhiteListBumps) -> Result<()> {
        self.whitelist.address = user.key();
        self.whitelist.bump = bumps.whitelist;

        self.user_db.address = user.key();
        self.user_db.bump = bumps.user_db;
        self.user_db.deposited = 0;

        Ok(())
    }
}
