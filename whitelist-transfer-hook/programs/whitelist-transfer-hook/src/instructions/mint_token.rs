use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{spl_token_2022::extension::transfer_hook::TransferHook, MintTo},
    token_interface::{
        self, spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint,
        Token2022, TokenAccount, TokenInterface, TokenMetadataInitialize,
    },
};

use crate::state::Whitelist;

#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        mint::decimals = 9,
        mint::authority = user,
        extensions::transfer_hook::authority = user,
        extensions::transfer_hook::program_id = crate::ID,
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = user,

        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> TokenFactory<'info> {
    pub fn init_mint(&mut self, amount: u64) -> Result<()> {
        let cpi = CpiContext::new(
            self.token_program.to_account_info(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: self.user_token_account.to_account_info(),
                authority: self.user.to_account_info(),
            },
        );

        token_interface::mint_to(cpi, amount)?;
        Ok(())
    }
}
