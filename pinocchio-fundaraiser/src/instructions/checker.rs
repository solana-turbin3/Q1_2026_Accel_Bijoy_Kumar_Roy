use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_token::{instructions::Transfer, state::TokenAccount};

use crate::{errors::PinocchioError, state::fundraiser::Fundraiser};

pub fn process_checker_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, fundraiser, maker_ata, vault, system_program, token_program, _remaining @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut fundraiser_data = fundraiser.try_borrow_mut()?;
    let fundraiser_state: &mut Fundraiser =
        bytemuck::from_bytes_mut(&mut fundraiser_data[..Fundraiser::SIZE]);

    let fundraiser_amount_to_raise = fundraiser_state.amount_to_raise;
    let fundraiser_bump = [fundraiser_state.bump];
    let maker = fundraiser_state.maker;

    drop(fundraiser_data);

    let vault_data = unsafe { vault.borrow_unchecked() };

    let vault_state = unsafe { TokenAccount::from_bytes_unchecked(&vault_data) };

    let vault_balance = vault_state.amount();

    if vault_balance < u64::from_le_bytes(fundraiser_amount_to_raise) {
        return Err(PinocchioError::TargetNotMet.into());
    }

    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.as_ref()),
        Seed::from(&fundraiser_bump),
    ];

    let seeds = Signer::from(&seed);

    Transfer {
        from: vault,
        to: maker_ata,
        authority: fundraiser,
        amount: vault_balance,
    }
    .invoke_signed(&[seeds.clone()])?;

    Ok(())
}
