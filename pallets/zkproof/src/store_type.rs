use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;

#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ProofStore {
	pub transcation_proof_a: [u8; 48],
	pub transcation_proof_b: [u8; 96],
	pub transcation_proof_c: [u8; 48],
}

impl Default for ProofStore {
	fn default() -> Self {
		Self {
			transcation_proof_a: [0; 48],
			transcation_proof_b: [0; 96],
			transcation_proof_c: [0; 48],
		}
	}
}

pub fn convert_to_proof_store(
	transcation_proof_a: Vec<u8>,
	transcation_proof_b: Vec<u8>,
	transcation_proof_c: Vec<u8>,
) -> ProofStore {
	let transcation_proof_a = transcation_proof_a.try_into().expect("Vec length is not 48");
	let transcation_proof_b = transcation_proof_b.try_into().expect("Vec length is not 96");
	let transcation_proof_c = transcation_proof_c.try_into().expect("Vec length is not 48");

	ProofStore { transcation_proof_a, transcation_proof_b, transcation_proof_c }
}

#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct VerificationKey {
	pub verification_alpha_g1: [u8; 96],
	pub verification_beta_g1: [u8; 96],
	pub verification_beta_g2: [u8; 192],
	pub verification_gamma_g2: [u8; 192],
	pub verification_delta_g1: [u8; 96],
	pub verification_delta_g2: [u8; 192],
	pub verification_ic: [[u8; 96]; 1],
}

impl Default for VerificationKey {
	fn default() -> Self {
		Self {
			verification_alpha_g1: [0; 96],
			verification_beta_g1: [0; 96],
			verification_beta_g2: [0; 192],
			verification_gamma_g2: [0; 192],
			verification_delta_g1: [0; 96],
			verification_delta_g2: [0; 192],
			verification_ic: [[0; 96]; 1],
		}
	}
}

pub fn convert_to_verification_key(
	verification_alpha_g1: Vec<u8>,
	verification_beta_g1: Vec<u8>,
	verification_beta_g2: Vec<u8>,
	verification_gamma_g2: Vec<u8>,
	verification_delta_g1: Vec<u8>,
	verification_delta_g2: Vec<u8>,
	verification_ic: Vec<Vec<u8>>,
) -> VerificationKey {
	let verification_alpha_g1 = verification_alpha_g1.try_into().expect("Vec length is not 96");
	let verification_beta_g1 = verification_beta_g1.try_into().expect("Vec length is not 96");
	let verification_beta_g2 = verification_beta_g2.try_into().expect("Vec length is not 192");
	let verification_gamma_g2 = verification_gamma_g2.try_into().expect("Vec length is not 192");
	let verification_delta_g1 = verification_delta_g1.try_into().expect("Vec length is not 96");
	let verification_delta_g2 = verification_delta_g2.try_into().expect("Vec length is not 192");

	let mut ic_array: [[u8; 96]; 1] = [[0; 96]; 1];
	for (i, item) in verification_ic.into_iter().enumerate() {
		ic_array[i] = item.try_into().expect("Vec length is not 96");
	}

	VerificationKey {
		verification_alpha_g1,
		verification_beta_g1,
		verification_beta_g2,
		verification_gamma_g2,
		verification_delta_g1,
		verification_delta_g2,
		verification_ic: ic_array,
	}
}
