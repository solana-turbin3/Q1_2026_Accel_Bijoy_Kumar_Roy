use anchor_lang::{
    prelude::{msg, Pubkey},
    solana_program::{native_token::LAMPORTS_PER_SOL, program_pack::Pack},
    system_program, AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::spl_associated_token_account,
    token_2022::spl_token_2022::{
        self,
        extension::{
            metadata_pointer::MetadataPointer, transfer_hook::TransferHook,
            BaseStateWithExtensions, StateWithExtensionsOwned,
        },
        state::{Account, Mint},
    },
    token_interface::{spl_token_metadata_interface::state::TokenMetadata, TokenAccount},
};
use litesvm::{types::TransactionMetadata, LiteSVM};
use litesvm_token::{
    get_spl_account, spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount,
};
use solana_address::Address;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_tlv_account_resolution::state::ExtraAccountMetaList;

fn from_pubkey_to_address(pubkey: &Pubkey) -> Address {
    Address::from(pubkey.to_bytes())
}

fn from_address_to_pubkey(address: &Address) -> Pubkey {
    Pubkey::from(address.to_bytes())
}

fn setup() -> (LiteSVM, Keypair, Address) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    // Airdrop
    svm.airdrop(&payer.pubkey(), 100 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL to payer");

    // Load program
    let program_id = from_pubkey_to_address(&transfer_enabled_vault::ID);
    let program_bytes = include_bytes!("../../../target/deploy/transfer_enabled_vault.so");
    svm.add_program(program_id, program_bytes).unwrap();

    (svm, payer, program_id)
}

fn setup_add_to_whitelist(
    svm: &mut LiteSVM,
    payer: &Keypair,
    program_id: &Address,
) -> (Address, Address, Keypair, TransactionMetadata) {
    let user = Keypair::new();
    let whitelist =
        Address::find_program_address(&[b"whitelist", user.pubkey().as_ref()], program_id).0;

    let user_db =
        Address::find_program_address(&[b"user_db", user.pubkey().as_ref()], program_id).0;

    let whitelist_accounts = transfer_enabled_vault::accounts::AddToWhiteList {
        admin: from_address_to_pubkey(&payer.pubkey()),
        whitelist: from_address_to_pubkey(&whitelist),
        user_db: from_address_to_pubkey(&user_db),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let whitelist_ix = Instruction {
        program_id: *program_id,
        accounts: whitelist_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: from_pubkey_to_address(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: transfer_enabled_vault::instruction::AddToWhitelist {
            user: from_address_to_pubkey(&user.pubkey()),
        }
        .data(),
    };

    let message = Message::new(&[whitelist_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // Send the transaction and capture the result
    let tx = svm.send_transaction(transaction).unwrap();
    (whitelist, user_db, user, tx)
}

fn setup_create_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
    program_id: &Address,
    name: &String,
    symbol: &String,
    uri: &String,
) -> (Address, TransactionMetadata) {
    // let mint = CreateMint::new(svm, &payer)
    //     .decimals(9)
    //     .authority(&payer.pubkey())
    //     .send()
    //     .unwrap();
    let mint_kp = Keypair::new();
    let token_program = spl_token_2022::ID;
    let system_program = system_program::ID;

    let create_mint_account = transfer_enabled_vault::accounts::CreateMint {
        admin: from_address_to_pubkey(&payer.pubkey()),
        mint: from_address_to_pubkey(&mint_kp.pubkey()),
        token_program,
        system_program,
    }
    .to_account_metas(None);

    let create_mint_ix = Instruction {
        program_id: *program_id,
        accounts: create_mint_account
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: from_pubkey_to_address(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: transfer_enabled_vault::instruction::CreateMint {
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
        }
        .data(),
    };

    let message = Message::new(&[create_mint_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new(&[&payer, &mint_kp], message, recent_blockhash);

    // Send the transaction and capture the result
    let tx = svm.send_transaction(transaction).unwrap();
    (mint_kp.pubkey(), tx)
}

fn setup_mint_token(
    svm: &mut LiteSVM,
    payer: &Keypair,
    program_id: &Address,
    mint: &Address,
    user: &Keypair,
    amount: u64,
) -> (Address, TransactionMetadata) {
    let token_program = spl_token_2022::ID;
    let system_program = system_program::ID;
    let associated_token_program = spl_associated_token_account::ID;

    let user_ata = anchor_spl::associated_token::get_associated_token_address_with_program_id(
        &from_address_to_pubkey(&user.pubkey()),
        &from_address_to_pubkey(mint),
        &token_program,
    );

    println!("{}", user_ata);
    println!("{}", token_program);
    println!("{}", mint);

    let mint_token_accounts = transfer_enabled_vault::accounts::MintToken {
        admin: from_address_to_pubkey(&payer.pubkey()),
        user: from_address_to_pubkey(&user.pubkey()),
        mint: from_address_to_pubkey(&mint),
        user_token_account: user_ata,
        system_program,
        token_program: token_program,
        associated_token_program,
    }
    .to_account_metas(None);

    let mint_token_ix = Instruction {
        program_id: *program_id,
        accounts: mint_token_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: from_pubkey_to_address(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: transfer_enabled_vault::instruction::MintToken { amount }.data(),
    };

    let msg = Message::new(&[mint_token_ix], Some(&payer.pubkey()));

    let transaction = Transaction::new(&[&payer], msg, svm.latest_blockhash());

    let tx = svm.send_transaction(transaction).unwrap();

    (from_pubkey_to_address(&user_ata), tx)
}

fn setup_init_vault(
    svm: &mut LiteSVM,
    payer: &Keypair,
    program_id: &Address,
) -> (Address, TransactionMetadata) {
    let system_program = system_program::ID;
    let vault = Address::find_program_address(&[b"vault"], program_id).0;
    let init_vault_accounts = transfer_enabled_vault::accounts::InitVault {
        admin: from_address_to_pubkey(&payer.pubkey()),
        vault: from_address_to_pubkey(&vault),
        system_program,
    }
    .to_account_metas(None);

    let init_vault_ix = Instruction {
        program_id: *program_id,
        accounts: init_vault_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: from_pubkey_to_address(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: transfer_enabled_vault::instruction::InitVault {}.data(),
    };

    let msg = Message::new(&[init_vault_ix], Some(&payer.pubkey()));

    let transaction = Transaction::new(&[&payer], msg, svm.latest_blockhash());

    let tx = svm.send_transaction(transaction).unwrap();

    return (vault, tx);
}

fn setup_init_extra_account_meta(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Address,
    program_id: &Address,
) -> (Address, TransactionMetadata) {
    let system_program = system_program::ID;
    let extra_account_meta_list =
        Address::find_program_address(&[b"extra-account-metas", mint.as_ref()], program_id).0;
    let init_extra_account_meta_accounts =
        transfer_enabled_vault::accounts::InitializeExtraAccountMetaList {
            payer: from_address_to_pubkey(&payer.pubkey()),
            extra_account_meta_list: from_address_to_pubkey(&extra_account_meta_list),
            mint: from_address_to_pubkey(&mint),
            system_program,
        }
        .to_account_metas(None);

    let init_extra_account_meta_ix = Instruction {
        program_id: *program_id,
        accounts: init_extra_account_meta_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: from_pubkey_to_address(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: transfer_enabled_vault::instruction::InitializeTransferHook {}.data(),
    };

    let msg = Message::new(&[init_extra_account_meta_ix], Some(&payer.pubkey()));

    let transaction = Transaction::new(&[&payer], msg, svm.latest_blockhash());

    let tx = svm.send_transaction(transaction).unwrap();

    return (extra_account_meta_list, tx);
}

fn setup_deposit(
    svm: &mut LiteSVM,
    payer: &Keypair,
    user: &Keypair,
    user_ata: &Address,
    mint: &Address,
    vault: &Address,
    whitelist: &Address,
    user_db: &Address,
    extra_account_meta_list: &Address,
    program_id: &Address,
) -> (Address, TransactionMetadata) {
    let token_program = spl_token_2022::ID;

    let deposit_amount = 5 * LAMPORTS_PER_SOL;

    let vault_ata = CreateAssociatedTokenAccount::new(svm, payer, mint)
        .owner(vault)
        .token_program_id(&from_pubkey_to_address(&token_program))
        .send()
        .unwrap();

    let deposit_accounts = transfer_enabled_vault::accounts::Deposit {
        user: from_address_to_pubkey(&user.pubkey()),

        user_db: from_address_to_pubkey(user_db),
    }
    .to_account_metas(None);

    let deposit_ix = Instruction {
        program_id: *program_id,
        accounts: deposit_accounts
            .into_iter()
            .map(|m| solana_instruction::AccountMeta {
                pubkey: from_pubkey_to_address(&m.pubkey),
                is_signer: m.is_signer,
                is_writable: m.is_writable,
            })
            .collect(),
        data: transfer_enabled_vault::instruction::Deposit {
            amount: deposit_amount,
        }
        .data(),
    };

    let cpi_ix = spl_token_2022::instruction::transfer_checked(
        &token_program,
        &from_address_to_pubkey(&user_ata),
        &from_address_to_pubkey(&mint),
        &from_address_to_pubkey(&vault_ata),
        &from_address_to_pubkey(&user.pubkey()),
        &[],
        deposit_amount,
        9u8,
    )
    .unwrap();

    let mut accounts: Vec<solana_instruction::AccountMeta> = cpi_ix
        .accounts
        .into_iter()
        .map(|m| solana_instruction::AccountMeta {
            pubkey: from_pubkey_to_address(&m.pubkey),
            is_signer: m.is_signer,
            is_writable: m.is_writable,
        })
        .collect();

    accounts.push({
        solana_instruction::AccountMeta {
            pubkey: *program_id,
            is_signer: false,
            is_writable: false,
        }
    });

    accounts.push({
        solana_instruction::AccountMeta {
            pubkey: *extra_account_meta_list,
            is_signer: false,
            is_writable: false,
        }
    });
    accounts.push({
        solana_instruction::AccountMeta {
            pubkey: *whitelist,
            is_signer: false,
            is_writable: false,
        }
    });
    accounts.push(solana_instruction::AccountMeta {
        pubkey: *vault,
        is_signer: false,
        is_writable: false,
    });
    let transfer_ix = Instruction {
        program_id: from_pubkey_to_address(&token_program),
        accounts,
        data: cpi_ix.data,
    };
    let msg = Message::new(&[deposit_ix, transfer_ix], Some(&payer.pubkey()));

    let transaction = Transaction::new(&[&payer, &user], msg, svm.latest_blockhash());

    let tx = svm.send_transaction(transaction).unwrap();
    println!("Transaction logs: {:?}", tx.logs);

    (vault_ata, tx)
}

#[test]
fn test_add_to_whitelist() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut svm, payer, program_id) = setup();

    let (whitelist, user_db, user, _tx) = setup_add_to_whitelist(&mut svm, &payer, &program_id);

    msg!("Add to whitelist successfull");

    let whitelist_account = svm.get_account(&whitelist).unwrap();

    let whitelist_data = transfer_enabled_vault::state::Whitelist::try_deserialize(
        &mut whitelist_account.data.as_ref(),
    )
    .unwrap();

    let user_db_account = svm.get_account(&user_db).unwrap();

    let user_db_data =
        transfer_enabled_vault::state::UserDb::try_deserialize(&mut user_db_account.data.as_ref())
            .unwrap();

    assert_eq!(
        whitelist_data.address,
        from_address_to_pubkey(&user.pubkey())
    );

    assert_eq!(user_db_data.address, from_address_to_pubkey(&user.pubkey()));
}

#[test]
fn test_create_mint() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut svm, payer, program_id) = setup();
    let name = "Turbin3".to_string();
    let symbol = "Tur".to_string();
    let uri = "https://example.com/turbin3.json".to_string();
    let (mint, _tx) = setup_create_mint(&mut svm, &payer, &program_id, &name, &symbol, &uri);

    msg!("Create mint successfull");

    let mint_account = svm.get_account(&mint).unwrap();

    let mint_state = StateWithExtensionsOwned::<Mint>::unpack(mint_account.data).unwrap();

    let transfer_ext = mint_state.get_extension::<TransferHook>();
    assert!(transfer_ext.is_ok());
    println!("{:?}", transfer_ext.unwrap());

    let metadata_ext = mint_state.get_extension::<MetadataPointer>().unwrap();

    assert_eq!(
        metadata_ext.metadata_address.0,
        from_address_to_pubkey(&mint)
    );

    let metadata = mint_state
        .get_variable_len_extension::<TokenMetadata>()
        .unwrap();
    assert_eq!(metadata.name, name);
    assert_eq!(metadata.symbol, symbol);
    assert_eq!(metadata.uri, uri);
}

#[test]
fn test_mint_token() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut svm, payer, program_id) = setup();
    let name = "Turbin3".to_string();
    let symbol = "Tur".to_string();
    let uri = "https://example.com/turbin3.json".to_string();
    let (mint, _tx) = setup_create_mint(&mut svm, &payer, &program_id, &name, &symbol, &uri);

    let user = Keypair::new();
    let amount = 20 * LAMPORTS_PER_SOL;
    let (user_ata, _tx) = setup_mint_token(&mut svm, &payer, &program_id, &mint, &user, amount);

    msg!("Minted token successfully");

    let user_account = svm.get_account(&user_ata).unwrap();
    let user_account_state =
        StateWithExtensionsOwned::<spl_token_2022::state::Account>::unpack(user_account.data)
            .unwrap();
    assert_eq!(user_account_state.base.amount, amount);
    println!("{:?}", user_account_state.base);
}

#[test]
fn test_init_vault() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut svm, payer, program_id) = setup();

    let (vault, _tx) = setup_init_vault(&mut svm, &payer, &program_id);

    let vault_account = svm.get_account(&vault).unwrap();

    let vault_data =
        transfer_enabled_vault::state::Vault::try_deserialize(&mut vault_account.data.as_ref())
            .unwrap();

    assert!(vault_data.bump > 0);
}

#[test]
fn test_init_extra_acount_meta() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut svm, payer, program_id) = setup();
    let name = "Turbin3".to_string();
    let symbol = "Tur".to_string();
    let uri = "https://example.com/turbin3.json".to_string();
    let (mint, _tx) = setup_create_mint(&mut svm, &payer, &program_id, &name, &symbol, &uri);
    let (_extra_account_meta_list, _tx) =
        setup_init_extra_account_meta(&mut svm, &payer, &mint, &program_id);
}

#[test]
fn test_deposit() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut svm, payer, program_id) = setup();
    let (whitelist, user_db, user, _tx) = setup_add_to_whitelist(&mut svm, &payer, &program_id);
    let name = "Turbin3".to_string();
    let symbol = "Tur".to_string();
    let uri = "https://example.com/turbin3.json".to_string();
    let (mint, _tx) = setup_create_mint(&mut svm, &payer, &program_id, &name, &symbol, &uri);
    let amount = 20 * LAMPORTS_PER_SOL;
    let (user_ata, _tx) = setup_mint_token(&mut svm, &payer, &program_id, &mint, &user, amount);
    let (vault, _tx) = setup_init_vault(&mut svm, &payer, &program_id);
    let (extra_account_meta_list, tx) =
        setup_init_extra_account_meta(&mut svm, &payer, &mint, &program_id);
    println!("Transaction logs(extra_account_meta_list): {:?}", tx.logs);
    println!("u:{}", user_ata);
    // println!("v:{}", user_ata);
    let (vault_ata, _tx) = setup_deposit(
        &mut svm,
        &payer,
        &user,
        &user_ata,
        &mint,
        &vault,
        &whitelist,
        &user_db,
        &extra_account_meta_list,
        &program_id,
    );

    let vault_ata_account = svm.get_account(&vault_ata).unwrap();
    let vault_ata_state =
        StateWithExtensionsOwned::<Account>::unpack(vault_ata_account.data).unwrap();
    assert_eq!(vault_ata_state.base.amount, 5 * LAMPORTS_PER_SOL);
}
