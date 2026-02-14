use crate::state::Agent;
use anchor_lang::{prelude::*, solana_program::instruction::Instruction, InstructionData};
use solana_gpt_oracle::ContextAccount;
use std::str::FromStr;
use tuktuk_program::{
    compile_transaction,
    cron::types::TriggerV0,
    tuktuk::{
        cpi::{accounts::QueueTaskV0, queue_task_v0},
        program::Tuktuk,
    },
    types::QueueTaskArgsV0,
    TransactionSourceV0,
};
#[derive(Accounts)]
pub struct Schedule<'info> {
    /// CHECK: This is dangerous
    #[account(
        mut,
        address = Pubkey::from_str("HTKdPx6ZeZWB8J7xY2BgxAQxbQUhV4jUyb8rPdT2gbkr").unwrap()
    )]
    pub user: Signer<'info>,
    /// CHECK: This is safe because we don't read or write from this account
    // #[account(
    //     mut,
    //     seeds = [b"user", user.key().as_ref()],
    //     bump = user_account.bump,
    // )]
    // pub user_account: Account<'info, UserAccount>,
    // /// CHECK: The oracle queue
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,
    #[account(seeds = [b"agent"], bump)]
    pub agent: Account<'info, Agent>,
    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,

    #[account(mut)]
    /// CHECK: Don't need to parse this account, just using it in CPI
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: Don't need to parse this account, just using it in CPI
    pub task_queue_authority: UncheckedAccount<'info>,
    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    /// CHECK: Via seeds
    #[account(
        mut,
        seeds = [b"queue_authority"],
        bump
    )]
    pub queue_authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub tuktuk_program: Program<'info, Tuktuk>,
}

impl<'info> Schedule<'info> {
    pub fn schedule(&mut self, prompt: String, task_id: u16, bumps: ScheduleBumps) -> Result<()> {
        msg!("Scheduling with a PDA queue authority");
        let (compiled_tx, _) = compile_transaction(
            vec![Instruction {
                program_id: crate::ID,
                accounts: crate::__cpi_client_accounts_run_agent::RunAgent {
                    payer: self.user.to_account_info(),
                    interaction: self.interaction.to_account_info(),
                    agent: self.agent.to_account_info(),
                    context_account: self.context_account.to_account_info(),
                    oracle_program: self.oracle_program.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                }
                .to_account_metas(None)
                .to_vec(),
                data: crate::instruction::RunAgent { text: prompt }.data(),
            }],
            vec![],
        )
        .unwrap();

        queue_task_v0(
            CpiContext::new_with_signer(
                self.tuktuk_program.to_account_info(),
                QueueTaskV0 {
                    payer: self.queue_authority.to_account_info(),
                    queue_authority: self.queue_authority.to_account_info(),
                    task_queue: self.task_queue.to_account_info(),
                    task_queue_authority: self.task_queue_authority.to_account_info(),
                    task: self.task.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                },
                &[&["queue_authority".as_bytes(), &[bumps.queue_authority]]],
            ),
            QueueTaskArgsV0 {
                trigger: tuktuk_program::TriggerV0::Timestamp(Clock::get()?.unix_timestamp + 10),
                transaction: TransactionSourceV0::CompiledV0(compiled_tx),
                crank_reward: None,
                free_tasks: 1,
                id: task_id,
                description: "test".to_string(),
            },
        )?;

        Ok(())
    }
}
