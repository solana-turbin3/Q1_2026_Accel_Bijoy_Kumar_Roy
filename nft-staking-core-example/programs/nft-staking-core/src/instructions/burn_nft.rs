use crate::errors::StakingError;
use crate::state::Config;
use anchor_lang::{prelude::*, solana_program::clock::SECONDS_PER_DAY};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        BurnV1CpiBuilder, UpdateCollectionPluginV1CpiBuilder, UpdatePluginV1CpiBuilder,
    },
    types::{Attribute, Attributes, FreezeDelegate, Plugin, PluginType},
    ID as MPL_CORE_ID,
};

#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: PDA Update authority for the collection
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump
    )]
    pub update_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,

    /// CHECK: NFT account to be burned
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,

    /// CHECK: Collection account
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,

    #[account(mut,
        seeds = [b"rewards", config.key().as_ref()],
        bump = config.rewards_bump
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = reward_mint,
        associated_token::authority = user
    )]
    pub user_reward_ata: Account<'info, TokenAccount>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> BurnStakedNft<'info> {
    pub fn burn(&mut self, bumps: &BurnStakedNftBumps) -> Result<()> {
        let base_asset = BaseAssetV1::try_from(&self.nft.to_account_info())?;
        require!(
            base_asset.owner == self.user.key(),
            StakingError::InvalidOwner
        );

        let fetched_attribute_list = match fetch_plugin::<BaseAssetV1, Attributes>(
            &self.nft.to_account_info(),
            PluginType::Attributes,
        ) {
            Err(_) => {
                return Err(StakingError::NotStaked.into());
            }
            Ok((_, attributes, _)) => attributes,
        };

        let mut staked_value: Option<&str> = None;
        let mut staked_at_value: Option<&str> = None;

        for attribute in &fetched_attribute_list.attribute_list {
            match attribute.key.as_str() {
                "staked" => {
                    staked_value = Some(&attribute.value);
                }
                "staked_at" => {
                    staked_at_value = Some(&attribute.value);
                }
                _ => {}
            }
        }

        require!(staked_value == Some("true"), StakingError::NotStaked);
        let current_timestamp = Clock::get()?.unix_timestamp;
        let staked_at_timestamp = staked_at_value
            .ok_or(StakingError::InvalidTimestamp)?
            .parse::<i64>()
            .map_err(|_| StakingError::InvalidTimestamp)?;

        // Calculate staked time in days
        let elapsed_seconds = current_timestamp
            .checked_sub(staked_at_timestamp)
            .ok_or(StakingError::InvalidTimestamp)?;

        let staked_time_days = elapsed_seconds
            .checked_div(SECONDS_PER_DAY as i64)
            .ok_or(StakingError::InvalidTimestamp)?;

        let amount = (staked_time_days as u64)
            .checked_mul(self.config.points_per_stake as u64)
            .ok_or(StakingError::Overflow)?;

        let massive_reward: u64 = if amount > 0 { amount * 1000 } else { 1000 };

        let collection_key = self.collection.key();
        let signer_seeds = &[
            b"update_authority",
            collection_key.as_ref(),
            &[bumps.update_authority],
        ];

        // Unfreeze the NFT (Thaw the asset)
        UpdatePluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
            .invoke_signed(&[signer_seeds])?;

        // Update collection lvl attribute
        let (_, collection_attributes, _) = fetch_plugin::<BaseCollectionV1, Attributes>(
            &self.collection.to_account_info(),
            PluginType::Attributes,
        )
        .map_err(|_| StakingError::CollectionAttributesMissing)?;

        let mut collection_attrs = Vec::new();

        for attr in collection_attributes.attribute_list {
            if attr.key == "total_staked" {
                let mut count: u64 = attr.value.parse().unwrap_or(0);
                count = count.saturating_sub(1);
            }
            collection_attrs.push(attr);
        }

        UpdateCollectionPluginV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .collection(&self.collection.to_account_info())
            .payer(&self.user.to_account_info())
            .authority(Some(&self.update_authority.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: collection_attrs,
            }))
            .invoke_signed(&[signer_seeds])?;

        // Since it's unfrozen, the user (owner) can authorize the burn.
        BurnV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .invoke()?;

        let seeds = &[
            b"config",
            collection_key.as_ref(),
            &[self.config.config_bump],
        ];

        let signer_seed = &[&seeds[..]];
        let cpi_accounts = MintTo {
            mint: self.reward_mint.to_account_info(),
            to: self.user_reward_ata.to_account_info(),
            authority: self.config.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seed,
        );

        token::mint_to(cpi_ctx, massive_reward)?;

        Ok(())
    }
}
