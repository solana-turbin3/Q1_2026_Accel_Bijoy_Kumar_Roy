use anchor_lang::prelude::*;

mod constants;
mod errors;
mod helper;
mod instructions;
mod state;
use instructions::*;

declare_id!("F11ecQ5d8DZwmz1jahKJ4wev7cL6Ww6t5qtzGdL3Xmhg");

#[program]
pub mod nft_staking_core {
    use super::*;

    pub fn create_collection(
        ctx: Context<CreateCollection>,
        name: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts.create_collection(name, uri, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<Mint>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_nft(name, uri, &ctx.bumps)
    }

    pub fn initialize_config(
        ctx: Context<InitConfig>,
        points_per_stake: u32,
        freeze_period: u8,
    ) -> Result<()> {
        ctx.accounts
            .init_config(points_per_stake, freeze_period, &ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(&ctx.bumps)
    }

    pub fn burn(ctx: Context<BurnStakedNft>) -> Result<()> {
        ctx.accounts.burn(&ctx.bumps)
    }

    pub fn initialize_oracle(ctx: Context<InitializeOracle>) -> Result<()> {
        ctx.accounts.initialize(ctx.bumps)
    }

    pub fn update_oracle(ctx: Context<UpdateOracle>) -> Result<()> {
        ctx.accounts.update()
    }

    pub fn transfer_nft(ctx: Context<TransferNFT>) -> Result<()> {
        ctx.accounts.transfer()
    }
}
