// use std::fmt::format;
use borsh::{BorshSerialize};
use borsh::{BorshDeserialize};
use std::io::{self, Write};
use std::time::Instant;
use token_tranfer::instructions::TransferIx;
use token_tranfer::processor::process_instruction;
use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
    },
    solana_program_test::{processor, tokio, ProgramTest},
    solana_sdk::{signature::Signer, signer::keypair::Keypair, transaction::Transaction},
    spl_token::instruction,
    spl_token::state::{Account, Mint},
};

#[tokio::test]
async fn tf_success() {
    // let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let mut amt = String::new();
    println!("How much SOL do you want to send? ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut amt)
        .map_err(|_| format!("could not get amount"));
    let amount = amt.trim().parse::<u64>().expect("INvalid input for amount");

    let program_id = Pubkey::new_unique();
    let source = Keypair::new();
    let destination = Keypair::new();
    let mint = Keypair::new();
    let (authority, _bump) = Pubkey::find_program_address(&[b"authority"], &program_id);

    // Add the program to the test framework
    let program_test =
        ProgramTest::new("token_tranfer", program_id, processor!(process_instruction));
    let decimals = 9;
    let rent = Rent::default();

    //Start the program test asnychrous test
    let (banks_client, payer, recent_blockhash) = program_test.start().await;
    let start = Instant::now();

    //CREATE MINT ACC INSTRUCTION
    let create_mint_acc_ix = system_instruction::create_account(
        &payer.pubkey(), // payer that will be paying for the creation of the account
        &mint.pubkey(), // the account to be created where the tokens (sol or custom) will be sent to
        rent.minimum_balance(Mint::LEN), // sol needed by the account to be rent exempt
        Mint::LEN as u64, // the space of the account created
        &spl_token::id(), // program that owns the account (e.g system_program, token_program, native loader)
    );
    //INITIALIZE THE MINT PROCESS
    let init_mint_ix = instruction::initialize_mint(
        &spl_token::id(), //Program Id
        &mint.pubkey(),   // mint account
        &payer.pubkey(),  // mint authority (who is allowed to mint more tokens)
        None,             //freeze authority (who can freeze the tokens)
        decimals,         // decimals(9 for solana and 6 for usdc)
    )
    .unwrap();
    let tx = Transaction::new_signed_with_payer(
        //IX DATA
        &[create_mint_acc_ix, init_mint_ix],
        //
        Some(&payer.pubkey()), // Payer
        &[&payer, &mint],      // signers
        recent_blockhash,      // recent blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();

    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    // SET UP SOURCE ACCOUNT OWNED BY PDA
    let create_source_acc_ix = system_instruction::create_account(
        &payer.pubkey(),                    // payer of the tx
        &source.pubkey(),                   // account to be created
        rent.minimum_balance(Account::LEN), // the amount of rent to be sent for the type of basic Account it is
        Account::LEN as u64,                // The space the account will occupy on the chain
        &spl_token::id(),                   // Program that owns the account
    );
    let init_source_acc_ix = instruction::initialize_account(
        &spl_token::id(), // token_program id
        &source.pubkey(), // account to be initialized
        &mint.pubkey(),   // The mint account where the tokens to be sent from
        &authority,       // the owner of the account
    )
    .unwrap();
    let tx2 = Transaction::new_signed_with_payer(
        &[create_source_acc_ix, init_source_acc_ix],
        Some(&payer.pubkey()),
        &[&payer, &source],
        recent_blockhash,
    );
    banks_client.process_transaction(tx2).await.unwrap();

    //SET UP DESTINATION ACCOUNT
    let create_destination_acc_ix = system_instruction::create_account(
        &payer.pubkey(),
        &destination.pubkey(),
        rent.minimum_balance(Account::LEN),
        Account::LEN as u64,
        &spl_token::id(),
    );
    let init_destination_acc_ix = instruction::initialize_account(
        &spl_token::id(),
        &destination.pubkey(),
        &mint.pubkey(),
        &authority,
    )
    .unwrap();
    let tx3 = Transaction::new_signed_with_payer(
        &[create_destination_acc_ix, init_destination_acc_ix],
        Some(&payer.pubkey()),
        &[&payer, &destination],
        recent_blockhash,
    );
    banks_client.process_transaction(tx3).await.unwrap();

    // MINT SOME TOKENS TO THE PDA ACCOUNT
    let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
    let mint_to_pda_ix = instruction::mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        &source.pubkey(),
        &payer.pubkey(), // must match the mint authority during initialize mint
        &[],             // if authority is a pda, provide seed used to sign for invoke_signed
        amount,
    )
    .unwrap();
    let tx4 = Transaction::new_signed_with_payer(
        &[mint_to_pda_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx4).await.unwrap();

    // Create the ix following the acc order expected by the program to transfer the tokens from the source to the destination.
    let accounts = vec![
        // new_readonly -> non writable, new -> writable
        AccountMeta::new_readonly(spl_token::id(), false), // spl token program
        AccountMeta::new(source.pubkey(), false),          //source token account
        AccountMeta::new_readonly(mint.pubkey(), false),   // Mint account
        AccountMeta::new(destination.pubkey(), false),     // Destination Token account
        AccountMeta::new_readonly(authority, false),       // pda authority
    ];

    let ix_data = TransferIx { amount };

    let serialized_data = ix_data.try_to_vec().unwrap();

    let ix_with_bincode = Instruction::new_with_bytes(
        // This is for borsh serialization
        // This just means it will load the instruction / function processed by the program id functions
        program_id,       //Program id of target program
        &serialized_data, // data
        accounts,         // list of accountMetas
    );

    let tx5 = Transaction::new_signed_with_payer(
        &[ix_with_bincode],    // instruction
        Some(&payer.pubkey()), // payer for the tx
        &[&payer],             // list of signers
        recent_blockhash, // recent blockhash to avoid replay attacks or double block submission
    );
    banks_client.process_transaction(tx5).await.unwrap();

    // println!("the balance of the source account is {:?}", destination.pubkey().balance());
    let account = banks_client
        .get_account(destination.pubkey())
        .await
        .unwrap()
        .unwrap();

    let token_account = Account::unpack(&account.data).unwrap();
    println!(
        "the amount in the token account is {:?}",
        token_account.amount
    );

    let duration = start.elapsed();
    println!("\nTHE DURATION OF THIS TEST IS {:?}\n", duration);
}
