#![cfg_attr(not(feature = "std"), no_std)]

use bellman::groth16::{
	create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof, Proof,
	VerifyingKey,
};
use bls12_381::{Bls12, G1Affine, G1Projective, Scalar};
use ff::{Field, PrimeField};
// use log::{error, info};
pub use pallet::*;
use pallet_randrng::ZKRng;
mod g1_to_scalar;

// use g1_to_scalar::glaffine_to_primefield;

mod proof_circuit;

use proof_circuit::{PtradCircuit, StoreCircuit, TranscationCircle};
mod store_type;
use store_type::{
	// convert_to_proof_store, 
	convert_to_verification_key, ProofStore, VerificationKey,
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;




#[frame_support::pallet]
pub mod pallet {
	use crate::store_type::convert_to_proof_store;

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::{pallet_prelude::*, 
		// Account
	};

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_randrng::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateProof(),
		VerifyProof(),
	}

	/// Error for the nicks pallet.
	#[pallet::error]
	pub enum Error<T> {
		/// During synthesis, we lacked knowledge of a variable assignment.
		AssignmentMissing,
		/// During synthesis, we divided by zero.
		DivisionByZero,
		/// During synthesis, we constructed an unsatisfiable constraint system.
		Unsatisfiable,
		/// During synthesis, our polynomials ended up being too high of degree
		PolynomialDegreeTooLarge,
		/// During proof generation, we encountered an identity in the CRS
		UnexpectedIdentity,
		/// During proof generation, we encountered an I/O error with the CRS
		// IoError(io::Error),
		UnThinkIOError,
		/// During CRS generation, we observed an unconstrained auxiliary variable
		UnconstrainedVariable,

		NotAgree,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	pub type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, u32, ProofStore, ValueQuery>;
	#[pallet::storage]
	pub type VerificationKeys<T: Config> =
		StorageMap<_, Blake2_128Concat, u32, VerificationKey, ValueQuery>;
	#[pallet::storage]
	pub type LockVolume<T: Config> = StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;
	#[pallet::storage]
	pub type AccountBalance<T: Config> = StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;
	#[pallet::storage]
	pub type TranscationRecognition<T: Config> = StorageMap<_, Blake2_128Concat, u32, bool, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_ptrade_proof())]
		pub fn create_ptrade_proof(_origin: OriginFor<T>, id: u32, account: u128) -> DispatchResult {
			log::info!("create_proof_start");
			let mut rng = pallet_randrng::create_rng::<T>();

			let circuit = PtradCircuit {
				private_key_alice: Some(Scalar::from(100u64)),
				public_key_alice: Some(G1Affine::from(
					G1Projective::generator() * Scalar::from(100u64),
				)),
				public_key_bob: Some(G1Affine::from(
					G1Projective::generator() * Scalar::from(200u64),
				)),
				trad_volume: Some(PrimeField::from_u128(account)),
				lock_volume: Some(PrimeField::from_u128(LockVolume::<T>::get(id))),
			};

			// Create parameters for our circuit
			let params = generate_random_parameters::<Bls12, _, _>(circuit, &mut rng).unwrap();
			log::info!("create_pretrade_param is success");

			let vk = params.vk.clone();
			let (
				verification_alpha_g1,
				verification_beta_g1,
				verification_beta_g2,
				verification_gamma_g2,
				verification_delta_g1,
				verification_delta_g2,
				verification_ic,
			) = vk.export();
			VerificationKeys::<T>::insert(
				id,
				convert_to_verification_key(
					verification_alpha_g1,
					verification_beta_g1,
					verification_beta_g2,
					verification_gamma_g2,
					verification_delta_g1,
					verification_delta_g2,
					verification_ic,
				),
			);

			// Create a Groth16 proof with our parameters.
			let proof = create_random_proof(circuit, &params, &mut rng).unwrap();
			let (proof_a, proof_b, proof_c) = proof.export();
			
			Self::deposit_event(pallet::Event::<_>::CreateProof());


			Proofs::<T>::insert(id, convert_to_proof_store(proof_a, proof_b, proof_c));
			log::info!("transcation_proof had been create");

			let result = verify_proof(&prepare_verifying_key(&params.vk), &proof, &[]);


			if result.is_ok() {
				log::info!("proof is ok");
				AccountBalance::<T>::insert(id,account);
				log::info!("account money is update");
				TranscationRecognition::<T>::insert(id,false);
				log::info!("{:?}'s transcation is init",id);
			} else {
				log::info!("proof is not right,lock is wrong");
			}


			Ok(())
		}
		#[pallet::call_index(1)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_random())]
		pub fn create_random(_origin: OriginFor<T>) -> DispatchResult {
			let mut rng: ZKRng<T> = pallet_randrng::create_rng::<T>();
			Scalar::random(&mut rng);
			Ok(())
		}
		#[pallet::call_index(2)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::verification_proof())]
		pub fn verification_proof(_origin: OriginFor<T>, id: u32) -> DispatchResult {
			log::info!("verification_proof_start");
			let proof_store = Proofs::<T>::get(id);
			let proof = Proof::<Bls12>::import(
				&proof_store.transcation_proof_a,
				&proof_store.transcation_proof_b,
				&proof_store.transcation_proof_c,
			)
			.unwrap();
			let verification_key = VerificationKeys::<T>::get(id);
			let vk = VerifyingKey::<Bls12>::import(
				&verification_key.verification_alpha_g1,
				&verification_key.verification_beta_g1,
				&verification_key.verification_beta_g2,
				&verification_key.verification_gamma_g2,
				&verification_key.verification_delta_g1,
				&verification_key.verification_delta_g2,
				verification_key.verification_ic.iter().map(|x| x.to_vec()).collect(),
			)
			.unwrap();

			let pvk = prepare_verifying_key(&vk);

			let result = verify_proof(&pvk, &proof, &[]);
			Self::deposit_event(pallet::Event::<_>::VerifyProof());

			log::info!("verification_proof_end");

			if result.is_ok() {
				log::info!("proof is ok");
			} else {
				log::info!("proof is not ok");
			}
			log::info!("result is {:?}", result);
			Ok(())
		}
		#[pallet::call_index(3)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_transcation_proof())]
		pub fn create_transcation_proof(_origin: OriginFor<T>, id: u32, trade_self: u32, trade_object: u32) -> DispatchResult {

			let flag = TranscationRecognition::<T>::get(trade_object);
			
			if flag == false{
				return Err(Error::<T>::NotAgree.into());
			}
			TranscationRecognition::<T>::insert(trade_self, true);
			log::info!("create_proof_start");
			let mut rng = pallet_randrng::create_rng::<T>();


			let circuit = TranscationCircle {
				private_key_alice: Some(Scalar::from(100u64)),
				public_key_alice: Some(G1Affine::from(
					G1Projective::generator() * Scalar::from(200u64),
				)),
				private_key_bob: Some(Scalar::from(200u64)),
				public_key_bob: Some(G1Affine::from(
					G1Projective::generator() * Scalar::from(100u64),
				)),
				trad_volume_alice: Some(PrimeField::from_u128(1000u128)),
				trad_volume_bob: Some(PrimeField::from_u128(1000u128)),
				trad_volume_alice_get: Some(PrimeField::from_u128(500u128)),
				trad_volume_alice_give: Some(PrimeField::from_u128(500u128)),
				trad_volume_bob_get: Some(PrimeField::from_u128(500u128)),
				trad_volume_bob_give: Some(PrimeField::from_u128(500u128)),
			};

			// Create parameters for our circuit
			let params = generate_random_parameters::<Bls12, _, _>(circuit, &mut rng).unwrap();

			log::info!("transcation params is create success");

			let vk = params.vk.clone();
			let (
				verification_alpha_g1,
				verification_beta_g1,
				verification_beta_g2,
				verification_gamma_g2,
				verification_delta_g1,
				verification_delta_g2,
				verification_ic,
			) = vk.export();
			VerificationKeys::<T>::insert(
				id,
				convert_to_verification_key(
					verification_alpha_g1,
					verification_beta_g1,
					verification_beta_g2,
					verification_gamma_g2,
					verification_delta_g1,
					verification_delta_g2,
					verification_ic,
				),
			);

			// Create a Groth16 proof with our parameters.
			let proof = create_random_proof(circuit, &params, &mut rng).unwrap();
			let (proof_a, proof_b, proof_c) = proof.export();
			Proofs::<T>::insert(id, convert_to_proof_store(proof_a, proof_b, proof_c));
			log::info!("transcation_proof had been create");
			Self::deposit_event(pallet::Event::<_>::CreateProof());

			Ok(())
		}
		#[pallet::call_index(4)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::create_store_proof())]
		pub fn create_store_proof(_origin: OriginFor<T>, id: u32) -> DispatchResult {
			log::info!("create_proof_start");
			let mut rng = pallet_randrng::create_rng::<T>();

			let circuit = StoreCircuit {
				private_key_alice: Some(Scalar::from(100u64)),
				public_key_alice: Some(G1Affine::from(
					G1Projective::generator() * Scalar::from(100u64),
				)),
				trad_volume_alice_get: Some(PrimeField::from_u128(500u128)),
				trad_volume_alice_give: Some(PrimeField::from_u128(500u128)),
				trad_volume_alice: Some(PrimeField::from_u128(1000u128)),
				trad_volume_bob_give: Some(PrimeField::from_u128(500u128)),
			};

			// Create parameters for our circuit
			let params = generate_random_parameters::<Bls12, _, _>(circuit, &mut rng).unwrap();
			log::info!("store params is success generate");

			let vk = params.vk.clone();
			let (
				verification_alpha_g1,
				verification_beta_g1,
				verification_beta_g2,
				verification_gamma_g2,
				verification_delta_g1,
				verification_delta_g2,
				verification_ic,
			) = vk.export();
			VerificationKeys::<T>::insert(
				id,
				convert_to_verification_key(
					verification_alpha_g1,
					verification_beta_g1,
					verification_beta_g2,
					verification_gamma_g2,
					verification_delta_g1,
					verification_delta_g2,
					verification_ic,
				),
			);

			// Create a Groth16 proof with our parameters.
			let proof = create_random_proof(circuit, &params, &mut rng).unwrap();
			let (proof_a, proof_b, proof_c) = proof.export();
			Proofs::<T>::insert(id, convert_to_proof_store(proof_a, proof_b, proof_c));
			log::info!("store_proof had been create");
			Self::deposit_event(pallet::Event::<_>::CreateProof());

			Ok(())
		}
		#[pallet::call_index(5)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::lock_volume())]
		pub fn lock_volume(_origin: OriginFor<T>, id: u32,volume: u128) -> DispatchResult {
			LockVolume::<T>::insert(id,volume);
			Ok(())
		}
		#[pallet::call_index(6)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::have_a_trade())]
		pub fn have_a_trade(_origin: OriginFor<T>, id: u32) -> DispatchResult {
			TranscationRecognition::<T>::insert(id,true);
			Ok(())
		}
	}
}
