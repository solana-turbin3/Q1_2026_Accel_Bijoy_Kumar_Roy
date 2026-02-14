use anchor_lang::prelude::*;

use crate::state::Agent;

#[derive(Accounts)]
pub struct CloseAgent<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        close = payer,
        seeds = [b"agent"],
        bump
    )]
    pub agent: Account<'info, Agent>,
    pub system_program: Program<'info, System>,
}

impl<'info> CloseAgent<'info> {
    pub fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
