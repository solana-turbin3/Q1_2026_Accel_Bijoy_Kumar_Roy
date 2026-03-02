#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use litesvm::LiteSVM;
    use litesvm_token::{
        get_spl_account,
        spl_token::{self},
        CreateAssociatedTokenAccount, CreateMint, MintTo, TOKEN_ID,
    };

    use solana_address::Address;
    use solana_clock::Clock;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_associated_token_account::get_associated_token_address_with_program_id;
    use spl_token::state::Account as TokenAccount;
    const PROGRAM_ID: &str = "4ibrEMW5F6hKnkW4jVedswYv6H6VtwPN6ar6dvXDN1nT";
    const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
    const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn get_token_balance(svm: &LiteSVM, account: &Address) -> u64 {
        let token_state: TokenAccount =
            get_spl_account(svm, account).expect("Token data not found");
        token_state.amount
    }

    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        // Load program SO file

        let program_bytes = include_bytes!(
            "/home/roy/solana_projs/whitelist-transfer-hook/pinocchio-fundaraiser/target/sbpf-solana-solana/release/pinocchio_fundaraiser.so"
        );

        svm.add_program(program_id(), program_bytes)
            .expect("Failed to add program");

        (svm, payer)
    }

    fn setup_initialize(
        svm: &mut LiteSVM,
        maker: &Keypair,
        program_id: &Address,
    ) -> (Address, Address, Address) {
        assert_eq!(program_id.to_string(), PROGRAM_ID);

        let mint_to_raise = CreateMint::new(svm, &maker)
            .decimals(6)
            .authority(&maker.pubkey())
            .send()
            .unwrap();
        println!("Mint: {}", mint_to_raise);

        let fundraiser_pda = Pubkey::find_program_address(
            &[b"fundraiser".as_ref(), maker.pubkey().as_ref()],
            &program_id,
        );

        println!("Fundraiser PDA: {}", fundraiser_pda.0);
        println!("Fundraiser Bump: {}", fundraiser_pda.1);

        // let vault = spl_associated_token_account::get_associated_token_address(
        //     &fundraiser_pda.0,
        //     &mint_to_raise,
        // );
        let vault_pda = Pubkey::find_program_address(
            &[b"vault".as_ref(), fundraiser_pda.0.as_ref()],
            &program_id,
        );
        let vault = vault_pda.0;
        println!("Vault PDA: {}", vault);
        println!("Vault ATA: {}", vault);

        let amount_to_raise: u64 = 1_000_000;
        let duration: u8 = 2;
        let bump: u8 = fundraiser_pda.1;
        let vault_bump: u8 = vault_pda.1;

        let mut ix_data = vec![0u8];
        ix_data.extend_from_slice(&amount_to_raise.to_le_bytes());
        ix_data.push(duration);
        ix_data.push(bump);
        ix_data.push(vault_bump);

        // let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
        let token_program = TOKEN_PROGRAM_ID;
        let system_program = solana_sdk_ids::system_program::ID;

        let accounts = vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new_readonly(mint_to_raise, false),
            AccountMeta::new(fundraiser_pda.0, false),
            AccountMeta::new(vault, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
            // AccountMeta::new_readonly(associated_token_program, false),
        ];

        let initialize_ix = Instruction {
            program_id: *program_id,
            accounts,
            data: ix_data,
        };

        let message = Message::new(&[initialize_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&maker], message, recent_blockhash);
        let tx = svm.send_transaction(transaction).unwrap();

        println!("\n\nInitialize Fundraiser Transaction Successful!");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
        println!("\n--- Program Logs ---");
        for log in tx.logs {
            println!("{}", log);
        }
        println!("--------------------\n");

        return (mint_to_raise, fundraiser_pda.0, vault);
    }

    fn setup_contribute(
        svm: &mut LiteSVM,
        contributor: &Keypair,
        program_id: &Address,
        mint: &Address,
        maker: &Keypair,
        fundraiser: &Address,
        vault: &Address,
        contribute: u64,
    ) {
        assert_eq!(program_id.to_string(), PROGRAM_ID);
        let token_program_id = TOKEN_PROGRAM_ID;
        let contributor_ata = CreateAssociatedTokenAccount::new(svm, contributor, mint)
            .owner(&contributor.pubkey())
            .token_program_id(&token_program_id)
            .send()
            .unwrap();

        let amount = 100_000_000;
        MintTo::new(svm, maker, mint, &contributor_ata, amount)
            .owner(maker)
            .send()
            .unwrap();

        let contributor_account = Pubkey::find_program_address(
            &[
                b"contributor".as_ref(),
                fundraiser.as_ref(),
                contributor.pubkey().as_ref(),
            ],
            &program_id,
        );

        println!("Contributor PDA: {}", contributor_account.0);
        println!("Contributor Bump: {}", contributor_account.1);

        let bump = contributor_account.1;
        let mut ix_data = vec![1u8];
        ix_data.extend_from_slice(&contribute.to_le_bytes());
        ix_data.push(bump);

        let system_program = solana_sdk_ids::system_program::ID;

        let accounts = vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new(contributor_account.0, false),
            AccountMeta::new(*fundraiser, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(*vault, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program_id, false),
        ];

        let contribute_ix = Instruction {
            program_id: *program_id,
            accounts,
            data: ix_data,
        };

        let message = Message::new(&[contribute_ix], Some(&contributor.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&contributor], message, recent_blockhash);
        let tx = svm.send_transaction(transaction).unwrap();

        println!("\n\nContribute Fundraiser Transaction Successful!");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
        println!("\n--- Program Logs ---");
        for log in tx.logs {
            println!("{}", log);
        }
        println!("--------------------\n");
    }

    fn setup_refund(
        svm: &mut LiteSVM,
        contributor: &Keypair,
        program_id: &Address,
        mint: &Address,
        maker: &Keypair,
        fundraiser: &Address,
        vault: &Address,
    ) {
        assert_eq!(program_id.to_string(), PROGRAM_ID);
        let token_program_id = TOKEN_PROGRAM_ID;
        let contributor_ata = get_associated_token_address_with_program_id(
            &contributor.pubkey(),
            mint,
            &token_program_id,
        );

        let contributor_account = Pubkey::find_program_address(
            &[
                b"contributor".as_ref(),
                fundraiser.as_ref(),
                contributor.pubkey().as_ref(),
            ],
            &program_id,
        );

        println!("Contributor PDA: {}", contributor_account.0);
        println!("Contributor Bump: {}", contributor_account.1);

        let bump = contributor_account.1;
        let ix_data = vec![2u8];

        let system_program = solana_sdk_ids::system_program::ID;

        let accounts = vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new(contributor_account.0, false),
            AccountMeta::new(*fundraiser, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(*vault, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program_id, false),
        ];

        let refund_ix = Instruction {
            program_id: *program_id,
            accounts,
            data: ix_data,
        };

        let message = Message::new(&[refund_ix], Some(&contributor.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&contributor], message, recent_blockhash);
        let tx = svm.send_transaction(transaction).unwrap();

        println!("\n\nRefund Fundraiser Transaction Successful!");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
        println!("\n--- Program Logs ---");
        for log in tx.logs {
            println!("{}", log);
        }
        println!("--------------------\n");
    }

    fn setup_checker(
        svm: &mut LiteSVM,
        program_id: &Address,
        mint: &Address,
        maker: &Keypair,
        fundraiser: &Address,
        vault: &Address,
    ) {
        assert_eq!(program_id.to_string(), PROGRAM_ID);
        let token_program_id = TOKEN_PROGRAM_ID;
        let maker_ata = CreateAssociatedTokenAccount::new(svm, maker, mint)
            .owner(&maker.pubkey())
            .token_program_id(&token_program_id)
            .send()
            .unwrap();

        // let contributor_account = Pubkey::find_program_address(
        //     &[
        //         b"contributor".as_ref(),
        //         fundraiser.as_ref(),
        //         contributor.pubkey().as_ref(),
        //     ],
        //     &program_id,
        // );

        // println!("Contributor PDA: {}", contributor_account.0);
        // println!("Contributor Bump: {}", contributor_account.1);

        // let bump = contributor_account.1;
        let ix_data = vec![3u8];

        let system_program = solana_sdk_ids::system_program::ID;

        let accounts = vec![
            AccountMeta::new(maker.pubkey(), true),
            AccountMeta::new(*fundraiser, false),
            AccountMeta::new(maker_ata, false),
            AccountMeta::new(*vault, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program_id, false),
        ];

        let checker_ix = Instruction {
            program_id: *program_id,
            accounts,
            data: ix_data,
        };

        let message = Message::new(&[checker_ix], Some(&maker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&maker], message, recent_blockhash);
        let tx = svm.send_transaction(transaction).unwrap();

        println!("\n\nChecker Fundraiser Transaction Successful!");
        println!("CUs Consumed: {}", tx.compute_units_consumed);
        println!("\n--- Program Logs ---");
        for log in tx.logs {
            println!("{}", log);
        }
        println!("--------------------\n");
    }

    #[test]
    pub fn test_initialize_instruction() {
        let (mut svm, maker) = setup();

        let program_id = program_id();

        setup_initialize(&mut svm, &maker, &program_id);
    }

    #[test]
    pub fn test_contribute_instruction() {
        let (mut svm, maker) = setup();

        let program_id = program_id();

        let (mint_to_raise, fundraiser_pda, vault) =
            setup_initialize(&mut svm, &maker, &program_id);

        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        let contribute_amount: u64 = 5_000_000;
        setup_contribute(
            &mut svm,
            &contributor,
            &program_id,
            &mint_to_raise,
            &maker,
            &fundraiser_pda,
            &vault,
            contribute_amount,
        );

        let balance = get_token_balance(&svm, &vault);

        assert_eq!(balance, contribute_amount, "Wrong contribution value");
    }

    #[test]
    pub fn test_refund_instruction() {
        let (mut svm, maker) = setup();

        let program_id = program_id();

        let (mint_to_raise, fundraiser_pda, vault) =
            setup_initialize(&mut svm, &maker, &program_id);

        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        setup_contribute(
            &mut svm,
            &contributor,
            &program_id,
            &mint_to_raise,
            &maker,
            &fundraiser_pda,
            &vault,
            500_000,
        );

        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += 3 * 86_400;
        svm.set_sysvar::<Clock>(&clock);

        setup_refund(
            &mut svm,
            &contributor,
            &program_id,
            &mint_to_raise,
            &maker,
            &fundraiser_pda,
            &vault,
        );
    }

    #[test]
    pub fn test_checker_instruction() {
        let (mut svm, maker) = setup();

        let program_id = program_id();

        let (mint_to_raise, fundraiser_pda, vault) =
            setup_initialize(&mut svm, &maker, &program_id);

        let contributor = Keypair::new();
        svm.airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Airdrop failed");

        setup_contribute(
            &mut svm,
            &contributor,
            &program_id,
            &mint_to_raise,
            &maker,
            &fundraiser_pda,
            &vault,
            5_000_000,
        );

        let mut clock = svm.get_sysvar::<Clock>();
        clock.unix_timestamp += 3 * 86_400;
        svm.set_sysvar::<Clock>(&clock);

        setup_checker(
            &mut svm,
            &program_id,
            &mint_to_raise,
            &maker,
            &fundraiser_pda,
            &vault,
        )
    }
}
