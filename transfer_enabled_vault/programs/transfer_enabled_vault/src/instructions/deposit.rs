use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    errors::VaultError,
    state::{UserDb, Vault},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user_db", user.key().as_ref()],
        bump
    )]
    pub user_db: Account<'info, UserDb>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        self.user_db.deposited = self
            .user_db
            .deposited
            .checked_add(amount as u128)
            .ok_or(VaultError::Overflow)?;
        // let cpi_accounts = TransferChecked {
        //     from: self.user_token_account.to_account_info(),
        //     mint: self.mint.to_account_info(),
        //     to: self.vault_token_account.to_account_info(),
        //     authority: self.user.to_account_info(),
        // };

        // let cpi_ctx = CpiContext::new(
        //     self.token_program.to_account_info(),
        //     cpi_accounts
        // ).with_remaining_accounts(remaining_accounts.to_vec());

        // token_interface::transfer_checked(
        //     cpi_ctx,
        //     amount,
        //     self.mint.decimals
        // )?;
        // let mut transfer_ix = spl_token_2022::instruction::transfer_checked(
        //     &self.token_program.key(),
        //     &self.user_token_account.key(),
        //     &self.mint.key(),
        //     &self.vault_token_account.key(),
        //     &self.user.key(),
        //     &[],
        //     amount,
        //     self.mint.decimals,
        // )?;

        // for acc in remaining_accounts {
        //     transfer_ix.accounts.push(AccountMeta {
        //         pubkey: *acc.key,
        //         is_signer: acc.is_signer,
        //         is_writable: acc.is_writable,
        //     });
        // }

        // let mut invoke_accounts = vec![
        //     self.user_token_account.to_account_info(),
        //     self.mint.to_account_info(),
        //     self.vault_token_account.to_account_info(),
        //     self.user.to_account_info(),
        //     self.token_program.to_account_info(),
        // ];

        // invoke_accounts.extend_from_slice(remaining_accounts);

        // anchor_lang::solana_program::program::invoke(
        //     &transfer_ix,
        //     &invoke_accounts,
        // )?;
        Ok(())
    }
}
