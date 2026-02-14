use anchor_lang::prelude::*;
use solana_gpt_oracle::Identity;

#[derive(Accounts)]
pub struct CallbackFromAgent<'info> {
    /// CHECK: Checked in oracle program
    pub identity: Account<'info, Identity>,
}

impl<'info> CallbackFromAgent<'info> {
    pub fn callback_from_agent(&mut self, response: String) -> Result<()> {
        // Check if the callback is from the LLM program
        if !self.identity.to_account_info().is_signer {
            return Err(ProgramError::InvalidAccountData.into());
        }

        msg!("Agent Response from Callback: {:?}", response);
        Ok(())
    }
}
