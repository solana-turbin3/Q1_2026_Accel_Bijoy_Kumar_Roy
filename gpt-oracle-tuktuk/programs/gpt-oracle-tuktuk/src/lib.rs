use anchor_lang::prelude::*;

mod instructions;
mod state;
pub use instructions::*;
declare_id!("4Sj9vquSMiQvcGCskAq2ygtb3hqmr9yaL9o39GHxPPv8");

#[program]
pub mod gpt_oracle_tuktuk {

    use super::*;
    const AGENT_DESC: &str = "You are an simple AI agent called Dante. \
    Your job is to generate a random word related to Devli May Cry franchise.
    Just a word as output so not provide any other explanation or special character or json";
    pub fn initialize(ctx: Context<InitializeAgent>) -> Result<()> {
        ctx.accounts.initialize_agent(AGENT_DESC.to_string())?;
        Ok(())
    }

    pub fn run_agent(ctx: Context<RunAgent>, text: String) -> Result<()> {
        ctx.accounts.run_agent(text)?;
        Ok(())
    }

    pub fn callback_from_agents(ctx: Context<CallbackFromAgent>, response: String) -> Result<()> {
        ctx.accounts.callback_from_agent(response)?;
        Ok(())
    }

    pub fn schedule(ctx: Context<Schedule>, prompt: String, task_id: u16) -> Result<()> {
        ctx.accounts.schedule(prompt, task_id, ctx.bumps)?;
        Ok(())
    }

    pub fn close_agent(ctx: Context<CloseAgent>) -> Result<()> {
        ctx.accounts.close()?;
        Ok(())
    }
}
