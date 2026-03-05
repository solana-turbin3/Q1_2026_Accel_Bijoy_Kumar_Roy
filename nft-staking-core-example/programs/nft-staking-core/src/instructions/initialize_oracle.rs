use anchor_lang::prelude::*;

use crate::{
    helper::is_transfer_window_open,
    state::{ExternalValidationResult, Oracle, OracleValidation},
};

#[derive(Accounts)]
pub struct InitializeOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = Oracle::INIT_SPACE,
        seeds = [b"oracle",collection.key().as_ref()],
        bump
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    #[account(
        seeds = [b"reward_vault", oracle.key().as_ref()],
        bump,
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeOracle<'info> {
    pub fn initialize(&mut self, bumps: InitializeOracleBumps) -> Result<()> {
        match is_transfer_window_open(Clock::get()?.unix_timestamp) {
            true => {
                msg!("oracle opened at market hours");
                self.oracle.set_inner(Oracle {
                    validation: OracleValidation::V1 {
                        transfer: ExternalValidationResult::Approved,
                        create: ExternalValidationResult::Pass,
                        update: ExternalValidationResult::Pass,
                        burn: ExternalValidationResult::Pass,
                    },
                    bump: bumps.oracle,
                    vault_bump: bumps.reward_vault,
                });
            }
            false => {
                self.oracle.set_inner(Oracle {
                    validation: OracleValidation::V1 {
                        transfer: ExternalValidationResult::Rejected,
                        create: ExternalValidationResult::Pass,
                        update: ExternalValidationResult::Pass,
                        burn: ExternalValidationResult::Pass,
                    },
                    bump: bumps.oracle,
                    vault_bump: bumps.reward_vault,
                });
            }
        }

        Ok(())
    }
}
