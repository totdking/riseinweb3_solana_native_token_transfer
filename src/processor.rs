pub use crate::instructions::TransferIx;

use borsh::{BorshDeserialize, BorshSerialize, to_vec};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::transfer_checked,
    state::{Account, Mint},
};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ix_data: &[u8],
) -> ProgramResult {
    let transfer_data_struct = TransferIx::try_from_slice(ix_data)?;

    msg!("\nINCOMING ONCHAIN DATA, {:?}\n", ix_data);

    let accounts_info_iter = &mut accounts.iter();
    let token_program_info = next_account_info(accounts_info_iter)?;
    let source_info = next_account_info(accounts_info_iter)?;
    let mint_info = next_account_info(accounts_info_iter)?;
    let destination_info = next_account_info(accounts_info_iter)?;
    let authority_info = next_account_info(accounts_info_iter)?;

    let (pda_auth, bump_seed) = Pubkey::find_program_address(&[b"authority"], program_id);
    if pda_auth != *authority_info.key {
        return Err(ProgramError::InvalidSeeds).into();
    }

    let source_account = Account::unpack(&source_info.try_borrow_data()?)?;

    let amount = transfer_data_struct.amount;
    let mint = Mint::unpack(&mint_info.try_borrow_data()?)?;
    let decimals = mint.decimals;

    let ix = transfer_checked(
        token_program_info.key,
        source_info.key,
        mint_info.key,
        destination_info.key,
        authority_info.key,
        &[], //NO MULTISIG ALLOWED
        amount,
        decimals,
    )?;

    let accounts = [
        token_program_info.clone(), //THEY CLAIM THIS IS NOT NEEDED, BUT FOR CLARITY
        source_info.clone(),
        mint_info.clone(),
        destination_info.clone(),
        authority_info.clone(),
    ];

    msg!("Token transfer processing");
    invoke_signed(&ix, &accounts, &[&[b"authority", &[bump_seed]]])
    

}
