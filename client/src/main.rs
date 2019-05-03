/*
   Copyright 2019 Supercomputing Systems AG

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

	   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.

*/

extern crate substrate_api_client;
extern crate runtime_primitives;
extern crate my_node_runtime;
extern crate parity_codec;
extern crate primitives;
extern crate hex_literal;

use substrate_keyring::AccountKeyring;
use node_primitives::{
	Index,
	Hash,
	AccountId,
};
use primitive_types::U256;
use substrate_api_client::{
	Api,
	hexstr_to_u256,
	extrinsic::transfer
};
use runtime_primitives::{
	generic::Era,
};
use primitives::{
	// ed25519,
	sr25519,
	hexdisplay::HexDisplay,
	Pair,
	crypto::Ss58Codec,
	blake2_256,
};
use parity_codec::{Encode, Compact};
use bip39::{Mnemonic, Language, MnemonicType};
use schnorrkel::keys::MiniSecretKey;
use rand::{RngCore, rngs::OsRng};
use my_node_runtime::{
	UncheckedExtrinsic,
	Call,
	SubstraTEEProxyCall,
};
use substrate_bip39::mini_secret_from_entropy;
use hex_literal::hex;

#[macro_use]
extern crate clap;
use clap::App;

trait Crypto {
	type Seed: AsRef<[u8]> + AsMut<[u8]> + Sized + Default;
	type Pair: Pair;
	fn generate_phrase() -> String {
		Mnemonic::new(MnemonicType::Words12, Language::English).phrase().to_owned()
	}
	fn generate_seed() -> Self::Seed {
		let mut seed: Self::Seed = Default::default();
		OsRng::new().unwrap().fill_bytes(seed.as_mut());
		seed
	}
	fn seed_from_phrase(phrase: &str, password: Option<&str>) -> Self::Seed;
	fn pair_from_seed(seed: &Self::Seed) -> Self::Pair;
	fn pair_from_suri(phrase: &str, password: Option<&str>) -> Self::Pair {
		Self::pair_from_seed(&Self::seed_from_phrase(phrase, password))
	}
	fn ss58_from_pair(pair: &Self::Pair) -> String;
	fn public_from_pair(pair: &Self::Pair) -> Vec<u8>;
	fn seed_from_pair(_pair: &Self::Pair) -> Option<&Self::Seed> { None }
	fn print_from_seed(seed: &Self::Seed) {
		let pair = Self::pair_from_seed(seed);
		println!("Seed 0x{} is account:\n  Public key (hex): 0x{}\n  Address (SS58): {}",
			HexDisplay::from(&seed.as_ref()),
			HexDisplay::from(&Self::public_from_pair(&pair)),
			Self::ss58_from_pair(&pair)
		);
	}
	fn print_from_phrase(phrase: &str, password: Option<&str>) {
		let seed = Self::seed_from_phrase(phrase, password);
		let pair = Self::pair_from_seed(&seed);
		println!("Phrase `{}` is account:\n  Seed: 0x{}\n  Public key (hex): 0x{}\n  Address (SS58): {}",
			phrase,
			HexDisplay::from(&seed.as_ref()),
			HexDisplay::from(&Self::public_from_pair(&pair)),
			Self::ss58_from_pair(&pair)
		);
	}
	fn print_from_uri(uri: &str, password: Option<&str>) where <Self::Pair as Pair>::Public: Sized + Ss58Codec + AsRef<[u8]> {
		if let Ok(pair) = Self::Pair::from_string(uri, password) {
			let seed_text = Self::seed_from_pair(&pair)
				.map_or_else(Default::default, |s| format!("\n  Seed: 0x{}", HexDisplay::from(&s.as_ref())));
			println!("Secret Key URI `{}` is account:{}\n  Public key (hex): 0x{}\n  Address (SS58): {}",
				uri,
				seed_text,
				HexDisplay::from(&Self::public_from_pair(&pair)),
				Self::ss58_from_pair(&pair)
			);
		}
		if let Ok(public) = <Self::Pair as Pair>::Public::from_string(uri) {
			println!("Public Key URI `{}` is account:\n  Public key (hex): 0x{}\n  Address (SS58): {}",
				uri,
				HexDisplay::from(&public.as_ref()),
				public.to_ss58check()
			);
		}
	}
}

struct Sr25519;

impl Crypto for Sr25519 {
	type Seed = [u8; 32];
	type Pair = sr25519::Pair;

	fn seed_from_phrase(phrase: &str, password: Option<&str>) -> Self::Seed {
		mini_secret_from_entropy(
			Mnemonic::from_phrase(phrase, Language::English)
				.unwrap_or_else(|_|
					panic!("Phrase is not a valid BIP-39 phrase: \n    {}", phrase)
				)
				.entropy(),
			password.unwrap_or("")
		)
			.expect("32 bytes can always build a key; qed")
			.to_bytes()
	}

	fn pair_from_suri(suri: &str, password: Option<&str>) -> Self::Pair {
		sr25519::Pair::from_string(suri, password).expect("Invalid phrase")
	}

	fn pair_from_seed(seed: &Self::Seed) -> Self::Pair {
		MiniSecretKey::from_bytes(seed)
			.expect("32 bytes can always build a key; qed")
			.into()
	}
	fn ss58_from_pair(pair: &Self::Pair) -> String { pair.public().to_ss58check() }
	fn public_from_pair(pair: &Self::Pair) -> Vec<u8> { (&pair.public().0[..]).to_owned() }
}

fn main() {
	let yml = load_yaml!("cli.yml");
	let matches = App::from_yaml(yml).get_matches();
	let port = matches.value_of("port").unwrap_or("9944");

	let mut api = Api::new(format!("ws://127.0.0.1:{}", port));
	api.init();

	// get Alice's AccountNonce
	let accountid = AccountId::from(AccountKeyring::Alice);
	let mut result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
	let mut nonce = hexstr_to_u256(result_str);
	println!("");
	println!("[+] Alice's account nonce is {}", nonce);

	// fund the account of the TEE (= Bob)
	let transfer_amount = 1000;
	println!("");
	println!("[+] Transfer {} to Bob", transfer_amount);
	let xt_1 = transfer("//Alice", "//Bob", U256::from(transfer_amount), nonce, api.genesis_hash.unwrap());
	let mut _xthex = hex::encode(xt_1.encode());
	_xthex.insert_str(0, "0x");
	// println!("extrinsic: {:?}", xt_1);

	// send and watch extrinsic until finalized
	let mut tx_hash = api.send_extrinsic(_xthex).unwrap();
	println!("");
	println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);

	// get the new nonce of Alice
	result_str = api.get_storage("System", "AccountNonce", Some(accountid.encode())).unwrap();
	nonce = hexstr_to_u256(result_str);
	println!("");
	println!("[+] Alice's new account nonce is {}", nonce);

	// generate extrinsic
	// FIXME: the payload must be encrypted with the public key of the TEE
	let payload_encrypted: Hash = hex!["1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"].into();
	let xt_2 = compose_extrinsic_substratee_forward("//Alice", payload_encrypted, nonce, api.genesis_hash.unwrap());

	println!("");
	// println!("extrinsic: {:?}", xt_2);
	_xthex = hex::encode(xt_2.encode());
	_xthex.insert_str(0, "0x");

	// send and watch extrinsic until finalized
	tx_hash = api.send_extrinsic(_xthex).unwrap();
	println!("[+] Transaction got finalized. Hash: {:?}", tx_hash);
	println!("");
}

pub fn compose_extrinsic_substratee_forward(sender: &str, payload_encrypted: Hash, index: U256, genesis_hash: Hash) -> UncheckedExtrinsic {

	let signer = Sr25519::pair_from_suri(sender, Some(""));
	let era = Era::immortal();

	let payload_encrypted_str = payload_encrypted.as_bytes().to_vec();
	let function = Call::SubstraTEEProxy(SubstraTEEProxyCall::forward(payload_encrypted_str));

	let index = Index::from(index.low_u64());
	let raw_payload = (Compact(index), function, era, genesis_hash);

	let signature = raw_payload.using_encoded(|payload| if payload.len() > 256 {
		signer.sign(&blake2_256(payload)[..])
	} else {
		println!("");
		println!("signing {}", HexDisplay::from(&payload));
		signer.sign(payload)
	});

	//let () = signature;
	//let sign = AnySignature::from(signature);

	UncheckedExtrinsic::new_signed(
		index,
		raw_payload.1,
		signer.public().into(),
		signature.into(),
		era,
	)
}