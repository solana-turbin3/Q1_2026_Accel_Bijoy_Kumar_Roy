use anchor_lang::prelude::*;

use crate::state::UserAccount;

#[derive(Accounts)]
pub struct VrfCallback<'info> {
    /// This check ensure that the vrf_program_identity (which is a PDA) is a singer
    /// enforcing the callback is executed by the VRF program trough CPI
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}

impl<'info> VrfCallback<'info> {
    pub fn callback(&mut self, randomness: [u8; 32]) -> Result<()> {
        let rnd_value = ephemeral_vrf_sdk::rnd::random_u64(&randomness);

        self.user_account.data = rnd_value;

        Ok(())
    }
}
