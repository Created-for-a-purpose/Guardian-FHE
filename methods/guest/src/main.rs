#![no_main]

use risc0_zkvm::guest::env;

use std::io::Read;

use fhe_traits::{DeserializeParametrized, Serialize};
use fhe::bfv::{Ciphertext, BfvParametersBuilder};

risc0_zkvm::guest::entry!(main);

#[derive(serde::Serialize, serde::Deserialize)]
struct FheParam {
    ciphtxt1: Vec<u8>,
    ciphtxt2: Vec<u8>,
}

pub fn main() {
    let parameters = BfvParametersBuilder::new()
      .set_degree(2048)
      .set_moduli(&[0x3fffffff000001])
      .set_plaintext_modulus(1 << 10)
      .build_arc().unwrap();

    let mut input_vector = Vec::new();
    env::stdin().read_to_end(&mut input_vector).unwrap();
    let input: FheParam = bincode::deserialize(&input_vector).unwrap();

    let ciph1: Ciphertext = Ciphertext::from_bytes(&input.ciphtxt1, &parameters).unwrap();
    let ciph2: Ciphertext = Ciphertext::from_bytes(&input.ciphtxt2, &parameters).unwrap();

    let ciph_out = &ciph1 + &ciph2;
    let result = ciph_out.to_bytes();

    env::commit(&result);
}
