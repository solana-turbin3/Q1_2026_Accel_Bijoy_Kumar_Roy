use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::extension::ExtensionType,
    token_interface::{
        spl_token_metadata_interface::state::TokenMetadata, token_metadata_initialize, Mint,
        TokenInterface, TokenMetadataInitialize,
    },
};

#[derive(Accounts)]
pub struct CreateMint<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        mint::decimals = 9,
        mint::authority = admin,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = crate::ID,
        extensions::metadata_pointer::authority = admin,
        extensions::metadata_pointer::metadata_address = mint.key(),
    )]
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateMint<'info> {
    pub fn create_mint(&mut self, name: String, symbol: String, uri: String) -> Result<()> {
        let token_metadata = TokenMetadata {
            update_authority: Some(self.admin.key()).try_into()?,
            mint: self.mint.key(),
            name: name.clone(),
            symbol: symbol.clone(),
            uri: uri.clone(),
            additional_metadata: vec![],
        };

        // Calculate space for mint with metadata pointer and token metadata extensions
        let mint_space = ExtensionType::try_calculate_account_len::<
            anchor_spl::token_2022::spl_token_2022::state::Mint,
        >(&[ExtensionType::MetadataPointer, ExtensionType::TransferHook])?;

        let metadata_len = token_metadata.tlv_size_of()?;
        let rent = Rent::get()?;

        let required_rent = rent.minimum_balance(mint_space + metadata_len);
        let current_lamports = self.mint.to_account_info().lamports();

        if required_rent > current_lamports {
            let lamports_needed = required_rent - current_lamports;

            anchor_lang::system_program::transfer(
                CpiContext::new(
                    self.system_program.to_account_info(),
                    anchor_lang::system_program::Transfer {
                        from: self.admin.to_account_info(),
                        to: self.mint.to_account_info(),
                    },
                ),
                lamports_needed,
            )?;
        }

        let cpi_accounts = TokenMetadataInitialize {
            program_id: self.token_program.to_account_info(),
            mint: self.mint.to_account_info(),
            metadata: self.mint.to_account_info(),
            mint_authority: self.admin.to_account_info(),
            update_authority: self.admin.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        token_metadata_initialize(cpi_ctx, name, symbol, uri)?;
        Ok(())
    }
}
