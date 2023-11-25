#![no_main]

use risc0_zkvm::guest::env;

use std::io::Read;

use fhe_traits::{DeserializeParametrized, Serialize};
use fhe::bfv::{Ciphertext, BfvParametersBuilder, BfvParameters};

risc0_zkvm::guest::entry!(main);

#[derive(serde::Serialize, serde::Deserialize)]
struct FhePayParam {
    sender_cipher_balance: Vec<u8>,
    ciphertext_amount_sender: Vec<u8>,
    receiver_cipher_balance: Vec<u8>,
    ciphertext_amount_receiver: Vec<u8>
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FhePayResult {
  sender_cipher_balance: Vec<u8>,
  receiver_cipher_balance: Vec<u8>
}

pub fn main() {
    let parameters =  BfvParameters::default_arc(1, 8);

    let mut input_vector = Vec::new();
    env::stdin().read_to_end(&mut input_vector).unwrap();
    let input: FhePayParam = bincode::deserialize(&input_vector).unwrap();

    let sender_cipher_balance: Ciphertext = Ciphertext::from_bytes(&input.sender_cipher_balance, &parameters).unwrap();
    let receiver_cipher_balance: Ciphertext = Ciphertext::from_bytes(&input.receiver_cipher_balance, &parameters).unwrap();

    let ciphertext_amount_sender: Ciphertext = Ciphertext::from_bytes(&input.ciphertext_amount_sender, &parameters).unwrap();
    let ciphertext_amount_receiver: Ciphertext = Ciphertext::from_bytes(&input.ciphertext_amount_receiver, &parameters).unwrap();

    let sender_cipher_new = &sender_cipher_balance - &ciphertext_amount_sender;
    let receiver_cipher_new = &receiver_cipher_balance + &ciphertext_amount_receiver;

    let result = FhePayResult { 
      sender_cipher_balance: sender_cipher_new.to_bytes(), 
      receiver_cipher_balance: receiver_cipher_new.to_bytes() 
    };

    let serialized_result = bincode::serialize(&result).unwrap();
    env::commit(&serialized_result);
}
