use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_token::{
    instructions::{CloseAccount, Transfer},
    state::TokenAccount,
};

use crate::{
    constant::SECONDS_TO_DAYS,
    errors::PinocchioError,
    state::{contributor::Contributor, fundraiser::Fundraiser},
};

pub fn process_refund_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [contributor, contributor_account, fundraiser, contributor_ata, vault, system_program, token_program, _remaining @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !contributor.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut fundraiser_data = fundraiser.try_borrow_mut()?;
    let fundraiser_state: &mut Fundraiser =
        bytemuck::from_bytes_mut(&mut fundraiser_data[..Fundraiser::SIZE]);

    let fundraiser_time_started = fundraiser_state.time_started;
    let fundraiser_duration = fundraiser_state.duration;
    let fundraiser_amount_to_raise = fundraiser_state.amount_to_raise;
    let fundraiser_bump = [fundraiser_state.bump];
    let maker = fundraiser_state.maker;

    drop(fundraiser_data);

    let current_time = Clock::get()?.unix_timestamp;

    let time_started = i64::from_le_bytes(bytemuck::cast(fundraiser_time_started));

    let elapsed_seconds = current_time - time_started;

    let duration_seconds = (fundraiser_duration as i64) * SECONDS_TO_DAYS;

    if elapsed_seconds < duration_seconds {
        return Err(PinocchioError::FundraiserNotEnded.into());
    }

    let vault_data = unsafe { vault.borrow_unchecked() };

    let vault_state = unsafe { TokenAccount::from_bytes_unchecked(&vault_data) };

    let vault_balance = vault_state.amount();

    if vault_balance > u64::from_le_bytes(fundraiser_amount_to_raise) {
        return Err(PinocchioError::TargetMet.into());
    }

    let mut contributor_data = contributor_account.try_borrow_mut()?;
    let contributor_state: &mut Contributor =
        bytemuck::from_bytes_mut(&mut contributor_data[..Contributor::SIZE]);

    let refund_amount = u64::from_le_bytes(contributor_state.amount);

    drop(contributor_data);

    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.as_ref()),
        Seed::from(&fundraiser_bump),
    ];

    let seeds = Signer::from(&seed);

    Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: refund_amount,
    }
    .invoke_signed(&[seeds.clone()])?;

    let lamports_to_return = contributor_account.lamports();

    contributor.set_lamports(contributor.lamports() + lamports_to_return);
    contributor_account.set_lamports(0);

    let mut closed_data = contributor_account.try_borrow_mut()?;
    closed_data.fill(0);

    Ok(())
}
