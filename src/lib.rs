// use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint;

pub mod processor;
use processor::process_instruction;
pub mod instructions;

entrypoint!(process_instruction);
