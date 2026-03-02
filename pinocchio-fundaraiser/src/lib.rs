#![cfg_attr(not(test), no_std)]
use pinocchio::{
    address::declare_id, entrypoint, error::ProgramError, AccountView, Address, ProgramResult,
};

use crate::instructions::FundraiserInstrctions;

mod constant;
mod errors;
mod instructions;
mod state;
mod tests;

entrypoint!(process_instructions);

declare_id!("4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT");

fn process_instructions(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match FundraiserInstrctions::try_from(discriminator)? {
        FundraiserInstrctions::Initialize => {
            instructions::process_initialize_instruction(accounts, data)?
        }
        FundraiserInstrctions::Contribute => {
            instructions::process_contribute_instruction(accounts, data)?
        }
        FundraiserInstrctions::Refund => instructions::process_refund_instruction(accounts, data)?,
        FundraiserInstrctions::Checker => {
            instructions::process_checker_instruction(accounts, data)?
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }
    Ok(())
}
