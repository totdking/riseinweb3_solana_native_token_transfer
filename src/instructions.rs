use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey, msg};

#[derive(BorshDeserialize, BorshSerialize, Debug, Clone, Copy)]
pub struct TransferIx {
    pub amount: u64,
}

impl TransferIx {
    pub fn unpack(input: &[u8]) -> Result<TransferIx, ProgramError> {
        msg!("Incoming transfer_ix bytes: {:?}", input);

        let unpacked = borsh::from_slice::<TransferIx>(input)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        msg!("Parsed amount: {}", unpacked.amount);
        Ok(unpacked)
    }
}

// pub fn get_amount(amount: u64) -> Result<u64, ProgramError>{
//     let amount_data = borsh::from_slice::<amount>(amount).map_err(|_| ProgramError::InvalidInstructionData)?;
// }