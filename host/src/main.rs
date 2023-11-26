use std::env;

/* For reading and writing files
use std::fs::File;
use std::io::Write;
use std::io::Read; */

// These constants represent the RISC-V ELF and the image ID generated by risc0-build.
// The ELF is used for proving and the ID is used for verification.
use methods::{
    FHE_GUEST_ELF, FHE_GUEST_ID
};
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt, ReceiptMetadata};

// FHE
use fhe_traits::{FheDecoder, FheEncoder, FheEncrypter, FheDecrypter, Serialize, DeserializeParametrized};
use fhe::bfv::{BfvParametersBuilder, Ciphertext, Encoding, Plaintext, PublicKey, SecretKey, BfvParameters};
use rand::{rngs::OsRng, thread_rng};

#[derive(serde::Serialize, serde::Deserialize)]
struct KeyPair{
  sk: Vec<u8>,
  pk: Vec<u8>
}

#[macro_use] extern crate rocket;
use rocket::serde::json::Json;

#[get("/generateKeyPair")]
fn generate_key_pair() -> String {
  let parameters = BfvParameters::default_arc(1, 8);
  let mut rng = thread_rng();
 
  let secret_key = SecretKey::random(&parameters, &mut OsRng);
  let public_key = PublicKey::new(&secret_key, &mut rng);

  /*@dev Write to local files
  let mut file = File::create("pkey.json").unwrap();
  let serialized_pk = serde_json::to_vec(&public_key.to_bytes()).unwrap();
  file.write(&serialized_pk);

  file = File::create("skey.json").unwrap();
  let serialized_sk = serde_json::to_vec(&secret_key.to_bytes()).unwrap();
  file.write(&serialized_sk); */

  let key_pair = KeyPair { sk: secret_key.to_bytes(), pk: public_key.to_bytes() };
  let serialized_key_pair = serde_json::to_string(&key_pair).unwrap();
  serialized_key_pair
}

// Encryption on demand

#[derive(rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
struct EncryptionWrapper {
  key: Vec<u8>,
  plaintext: u64
}

#[post("/encrypt", format="json", data="<encryption_params>")]
fn encrypt(encryption_params: Json<EncryptionWrapper>) -> Vec<u8> {
  let pkey = &encryption_params.key;
  let plaintext = encryption_params.plaintext;

  let parameters = BfvParameters::default_arc(1, 8);
  let mut rng = thread_rng();

  /*@dev Read from a local file
  let mut file = File::open("pkey.json").unwrap();
  let mut ser_pk = String::new();
  file.read_to_string(&mut ser_pk).unwrap();
  let pkey: Vec<u8>= serde_json::from_str(&ser_pk).unwrap();*/
  
  let public_key = PublicKey::from_bytes(&pkey, &parameters).unwrap();

  let plaintext_encoded = Plaintext::try_encode(&[plaintext], Encoding::poly(), &parameters).unwrap();
  let ciphertext = PublicKey::try_encrypt(&public_key, &plaintext_encoded, &mut rng).unwrap();

  /*@dev Write to a local file
  let mut file = File::create("cipher.json").unwrap();
  file.write(&serialized_cipher); */

  let serialized_cipher = serde_json::to_vec(&ciphertext.to_bytes()).unwrap();
  serialized_cipher
}

// Decryption on demand

#[derive(rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
struct DecryptionWrapper {
  key: Vec<u8>,
  ciphertext: Vec<u8>
}

#[post("/decrypt", format="json", data="<decryption_params>")]
fn decrypt(decryption_params: Json<DecryptionWrapper>) -> Vec<u8>{
  let secret_key_vec = &decryption_params.key;
  let ciphertext_vec = &decryption_params.ciphertext;

  let parameters = BfvParameters::default_arc(1, 8);

  /*@dev Read from a local file
  let mut file = File::open("skey.json").unwrap();
  let mut ser_sk = String::new();
  file.read_to_string(&mut ser_sk).unwrap();
  let skey: Vec<u8>= serde_json::from_str(&ser_sk).unwrap();
  */
  
  /*@dev Read from a local file
  file = File::open("cipher.json").unwrap();
  let mut ser_c = String::new();
  file.read_to_string(&mut ser_c).unwrap();
  let ciph: Vec<u8>= serde_json::from_str(&ser_c).unwrap();
  */

  let secret_key = SecretKey::from_bytes(&secret_key_vec, &parameters).unwrap();
  let ciphertext = Ciphertext::from_bytes(&ciphertext_vec, &parameters).unwrap();

  let decrypted_plaintext = SecretKey::try_decrypt(&secret_key, &ciphertext).unwrap();
  let decrypted_vector = Vec::<i64>::try_decode(&decrypted_plaintext, Encoding::poly()).unwrap();
  println!("decrypted_result {}", decrypted_vector[0]);

  let serialized_value = serde_json::to_vec(&decrypted_vector).unwrap();
  serialized_value
}

// FHE operations

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

#[derive(rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
struct FhePayWrapper {
  sender_pk: Vec<u8>,
  sender_cipher_balance: Vec<u8>,
  receiver_pk: Vec<u8>,
  receiver_cipher_balance: Vec<u8>,
  plaintext_amount: u64
}

#[post("/fhe_pay", format="json", data="<fhe_wrapper>")]
fn fhe_pay(fhe_wrapper: Json<FhePayWrapper>) -> Vec<u8>{
  let sender_pk = &fhe_wrapper.sender_pk;
  let sender_cipher_balance = &fhe_wrapper.sender_cipher_balance;
  let receiver_pk = &fhe_wrapper.receiver_pk;
  let receiver_cipher_balance = &fhe_wrapper.receiver_cipher_balance;
  let plaintext_amount = fhe_wrapper.plaintext_amount;

  let parameters = BfvParameters::default_arc(1, 8);
  let mut rng = thread_rng();

  let sender_public_key = PublicKey::from_bytes(&sender_pk, &parameters).unwrap();
  let receiver_public_key = PublicKey::from_bytes(&receiver_pk, &parameters).unwrap();

  let sender_cipher_balance = Ciphertext::from_bytes(&sender_cipher_balance, &parameters).unwrap();
  let receiver_cipher_balance = Ciphertext::from_bytes(&receiver_cipher_balance, &parameters).unwrap();
  
  let plaintext_amount_encoded = Plaintext::try_encode(&[plaintext_amount], Encoding::poly(), &parameters).unwrap();
  let ciphertext_amount_sender = PublicKey::try_encrypt(&sender_public_key, &plaintext_amount_encoded, &mut rng).unwrap();
  let ciphertext_amount_receiver = PublicKey::try_encrypt(&receiver_public_key, &plaintext_amount_encoded, &mut rng).unwrap();

  let fhe_pay_params = FhePayParam { 
    sender_cipher_balance: sender_cipher_balance.to_bytes(), 
    ciphertext_amount_sender: ciphertext_amount_sender.to_bytes(), 
    receiver_cipher_balance: receiver_cipher_balance.to_bytes(), 
    ciphertext_amount_receiver: ciphertext_amount_receiver.to_bytes() 
  };

  // Verifiable computation
  let env = ExecutorEnv::builder()
    .write_slice(&bincode::serialize(&fhe_pay_params).unwrap())
    .build().unwrap();

  // Obtain the default prover.
  let prover = default_prover();
  // Produce a receipt by proving the specified ELF binary.
  let receipt = prover.prove_elf(env, FHE_GUEST_ELF).unwrap();
  // Deserialize resulting ciphertext
  let result: Vec<u8> = receipt.journal.decode().unwrap();
  // Verify the receipt
  receipt.verify(FHE_GUEST_ID).unwrap();

  if env::var("RISC0_DEV_MODE").unwrap() != "1" {
    let metadata= receipt.get_metadata();
    match metadata{
      Ok(m) => {
        let vec32 = risc0_zkvm::serde::to_vec(&m).unwrap();
        let post_digest = m.post.digest();
        let post_digest_vec = bincode::serialize(&post_digest).unwrap();
        let metadata_vecu8 = bincode::serialize(&vec32).unwrap();
        let image_id = bincode::serialize(&FHE_GUEST_ID).unwrap();
        let journal = bincode::serialize(&receipt.journal).unwrap();
        let inner = receipt.inner;
        let segment_receipt = inner.flat();
        match segment_receipt{
          Ok(sr) => {
            let seal_bytes: Vec<u8> = sr[0].get_seal_bytes();
            // Return seal bytes, image id, post digest, journal and receipt metadata
            return [&seal_bytes[..], &image_id[..], &post_digest_vec[..], &journal[..], &metadata_vecu8[..]].concat();
          }
          Err(error) => {
            println!("Error: {}", error);
            return vec![];
          }
        }
      }
      Err(error) => {
        println!("Error: {}", error);
        return vec![];
      }
    };
  }

  let fhe_pay_result: FhePayResult = bincode::deserialize(&result).unwrap();
  let serialized_fhe_pay_result = serde_json::to_vec(&fhe_pay_result).unwrap();
  serialized_fhe_pay_result
}

use rocket::http::Header;
use rocket::{Request, Response};
use rocket::fairing::{Fairing, Info, Kind};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Attaching CORS headers to responses",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

#[launch]
fn rocket() -> _ {
  env::set_var("BONSAI_API_KEY", "IXviFK2L345w82NBqy8rOwLVzMVWcKh5XdhhYmD7");
  env::set_var("BONSAI_API_URL", "https://api.bonsai.xyz/swagger-ui/");
  // env::set_var("RISC0_DEV_MODE", "1");

  rocket::build().attach(CORS).mount("/", routes![generate_key_pair, encrypt, decrypt, fhe_pay])
}