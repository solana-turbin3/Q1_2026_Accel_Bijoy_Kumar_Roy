use bytemuck::{Pod, Zeroable};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{clock::Clock, rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::InitializeAccount3;

use crate::{constant::MIN_AMOUNT_TO_RAISE, state::fundraiser::Fundraiser};

#[repr(C)]
#[derive(Zeroable, Pod, Copy, Clone)]
pub struct InitializeArgs {
    pub amount: [u8; 8],
    pub duration: u8,
    pub bump: u8,
    pub vault_bump: u8,
}

pub fn process_initialize_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_to_raise, fundraiser, vault, system_program, token_program, _remaining @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if data.len() < core::mem::size_of::<InitializeArgs>() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let args: &InitializeArgs = bytemuck::from_bytes(&data);

    let mint_data = unsafe { mint_to_raise.borrow_unchecked() };

    if mint_data.len() < 82 {
        return Err(ProgramError::InvalidAccountData);
    }
    let amount_u64 = u64::from_le_bytes(args.amount);
    let min_amount_u64 = u64::from_le_bytes(MIN_AMOUNT_TO_RAISE);
    if amount_u64 < min_amount_u64 {
        return Err(ProgramError::InvalidArgument);
    }

    let bump = [args.bump];
    let seed = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_ref()),
        Seed::from(&bump),
    ];
    let seeds = Signer::from(&seed);

    CreateAccount {
        from: maker,
        to: fundraiser,
        lamports: Rent::get()?.try_minimum_balance(Fundraiser::SIZE)?,
        space: Fundraiser::SIZE as u64,
        owner: &crate::ID,
    }
    .invoke_signed(&[seeds.clone()])?;

    // let (vault_pda, vault_bump) =
    //     Address::find_program_address(&[b"vault", fundraiser.address().as_ref()], &crate::ID);

    let vault_bump = [args.vault_bump];
    let vault_pda = Address::create_program_address(
        &[b"vault", fundraiser.address().as_ref(), &vault_bump],
        &crate::ID,
    )?;

    // Create {
    //     funding_account: maker,
    //     account: vault,
    //     wallet: fundraiser,
    //     mint: mint_to_raise,
    //     system_program: system_program,
    //     token_program: token_program,
    // }
    // .invoke()?;

    let vault_seed = [
        Seed::from(b"vault"),
        Seed::from(fundraiser.address().as_ref()),
        Seed::from(&vault_bump),
    ];
    let vault_signer = Signer::from(&vault_seed);

    CreateAccount {
        from: maker,
        to: vault,
        lamports: Rent::get()?.try_minimum_balance(165)?, // 165 bytes is exactly one Token Account
        space: 165,
        owner: token_program.address(),
    }
    .invoke_signed(&[vault_signer.clone()])?;

    InitializeAccount3 {
        account: vault,
        mint: mint_to_raise,
        owner: fundraiser.address(),
    }
    .invoke()?;

    let mut fundraiser_data = fundraiser.try_borrow_mut()?;
    let state: &mut Fundraiser = bytemuck::from_bytes_mut(&mut fundraiser_data[..Fundraiser::SIZE]);

    state.maker = maker.address().to_bytes();
    state.mint_to_raise = mint_to_raise.address().to_bytes();
    state.amount_to_raise = args.amount;
    state.current_amount = 0u64.to_le_bytes();
    state.time_started = bytemuck::cast(Clock::get()?.unix_timestamp.to_le_bytes());
    state.duration = args.duration;
    state.bump = args.bump;

    Ok(())
}
