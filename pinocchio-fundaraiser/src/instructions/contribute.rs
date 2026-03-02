use bytemuck::{Pod, Zeroable};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::state::{contributor::Contributor, fundraiser::Fundraiser};

#[repr(C)]
#[derive(Zeroable, Pod, Copy, Clone)]
pub struct ContributeArgs {
    pub amount: [u8; 8],
    pub bump: u8,
}

pub fn process_contribute_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [contributor, contributor_account, fundraiser, contributor_ata, vault, system_program, token_program, _remaining @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if data.len() < core::mem::size_of::<ContributeArgs>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let args: &ContributeArgs = bytemuck::from_bytes(data);

    let amount = u64::from_le_bytes(args.amount);

    let bump = [args.bump];

    let seed = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_ref()),
        Seed::from(contributor.address().as_ref()),
        Seed::from(&bump),
    ];

    let seeds = Signer::from(&seed);

    CreateAccount {
        from: contributor,
        to: contributor_account,
        lamports: Rent::get()?.try_minimum_balance(Contributor::SIZE)?,
        space: Contributor::SIZE as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[seeds.clone()])?;

    Transfer {
        from: contributor_ata,
        to: vault,
        authority: contributor,
        amount: amount,
    }
    .invoke()?;

    let mut contributor_data = contributor_account.try_borrow_mut()?;
    let state: &mut Contributor =
        bytemuck::from_bytes_mut(&mut contributor_data[..Contributor::SIZE]);

    state.amount = args.amount;
    state.bump = args.bump;

    let mut fundraiser_data = fundraiser.try_borrow_mut()?;
    let state: &mut Fundraiser = bytemuck::from_bytes_mut(&mut fundraiser_data[..Fundraiser::SIZE]);

    let new_current_amount = u64::from_be_bytes(state.current_amount) + amount;
    state.current_amount = new_current_amount.to_le_bytes();
    Ok(())
}
