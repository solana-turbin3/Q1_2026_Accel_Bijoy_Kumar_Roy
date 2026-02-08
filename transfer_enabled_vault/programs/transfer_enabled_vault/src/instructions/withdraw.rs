use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::state::Vault;

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
        token::mint = mint,
        token::authority = vault,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &mut self,
        amount: u64,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<()> {
        let seeds: &[&[u8]] = &[b"vault", &[self.vault.bump]];
        let signer = &[&seeds[..]];

        let cpi_accounts = TransferChecked {
            from: self.vault_token_account.to_account_info(),
            mint: self.mint.to_account_info(),
            to: self.user_token_account.to_account_info(),
            authority: self.vault.to_account_info(),
        };

        let cpi_ctx =
            CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_accounts, signer)
                .with_remaining_accounts(remaining_accounts.to_vec());

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        Ok(())
    }
}
