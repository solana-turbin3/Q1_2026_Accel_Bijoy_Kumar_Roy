use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    self, approve, Approve, Mint, TokenAccount, TokenInterface, TransferChecked,
};

use crate::{
    errors::VaultError,
    state::{UserDb, Vault},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = user,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        seeds = [b"vault"], 
        bump=vault.bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [b"user_db", user.key().as_ref()],
        bump
    )]
    pub user_db: Account<'info, UserDb>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = vault,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let seeds: &[&[u8]] = &[b"vault", &[self.vault.bump]];
        let signer = &[&seeds[..]];
        require!(
            self.user_db.deposited >= amount as u128,
            VaultError::NotEnoughToken
        );

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            Approve {
                to: self.vault_token_account.to_account_info(),
                delegate: self.user.to_account_info(),
                authority: self.vault.to_account_info(),
            },
            signer,
        );

        approve(ctx, amount)?;
        self.user_db.deposited = self
            .user_db
            .deposited
            .checked_sub(amount as u128)
            .ok_or(VaultError::Underflow)?;
        Ok(())
    }
}
