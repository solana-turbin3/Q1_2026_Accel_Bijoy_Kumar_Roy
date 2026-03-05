use anchor_lang::{prelude::*, system_program::{Transfer, transfer}};

use crate::{constants::REWARD_IN_LAMPORTS, errors::StakingError, helper::{is_near_boundary, is_transfer_window_open}, state::{ExternalValidationResult, Oracle, OracleValidation}};

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"oracle",collection.key().as_ref()],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: Collection account will be checked by the mpl core program
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    #[account(
        mut, 
        seeds = [b"reward_vault", oracle.key().as_ref()],
        bump = oracle.vault_bump,
    )]
    pub reward_vault: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateOracle<'info>  {
      pub fn update(&mut self) -> Result<()> {
        msg!("{:?}", self.oracle.validation);
        match is_transfer_window_open(Clock::get()?.unix_timestamp) {
            true => {
                require!(
                    self.oracle.validation
                        == OracleValidation::V1 {
                            transfer: ExternalValidationResult::Rejected,
                            create: ExternalValidationResult::Pass,
                            burn: ExternalValidationResult::Pass,
                            update: ExternalValidationResult::Pass
                        },
                    StakingError::AlreadyUpdated
                );

                self.oracle.validation = OracleValidation::V1 {
                    transfer: ExternalValidationResult::Approved,
                    create: ExternalValidationResult::Pass,
                    burn: ExternalValidationResult::Pass,
                    update: ExternalValidationResult::Pass,
                };
            }
            false => {
                require!(
                    self.oracle.validation
                        == OracleValidation::V1 {
                            transfer: ExternalValidationResult::Approved,
                            create: ExternalValidationResult::Pass,
                            burn: ExternalValidationResult::Pass,
                            update: ExternalValidationResult::Pass
                        },
                    StakingError::AlreadyUpdated
                );

                self.oracle.validation = OracleValidation::V1 {
                    transfer: ExternalValidationResult::Rejected,
                    create: ExternalValidationResult::Pass,
                    burn: ExternalValidationResult::Pass,
                    update: ExternalValidationResult::Pass,
                };
            }
        }

        let reward_vault_balance = self.reward_vault.lamports();
        let oracle = self.oracle.key();
        let signer_seeds = &[b"reward_vault", oracle.as_ref(), &[self.oracle.bump]];
         msg!("Here");
        if is_near_boundary(Clock::get()?.unix_timestamp) && reward_vault_balance > REWARD_IN_LAMPORTS
        {
            transfer(
                CpiContext::new_with_signer(
                    self.system_program.to_account_info(),
                    Transfer {
                        from: self.reward_vault.to_account_info(),
                        to: self.payer.to_account_info(),
                    },
                    &[signer_seeds],
                ),
                REWARD_IN_LAMPORTS,
            )?
        }else {
            msg!("Outside Boundary");
        }
        Ok(())
    }

}