#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::Randomness;
pub use pallet::*;
// use pallet_insecure_randomness_collective_flip;
use rand_core::{impls, RngCore};

// use sp_std::vec::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;


#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		type RandomnessSource: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::storage]
	pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type RandomNumber<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub(super) type RandomNumber64<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub(super) type OnInitialize<T: Config> =
		StorageMap<_, Blake2_128Concat, T::BlockNumber, u32, ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		RandomNumberGenerated(u32),
	}
	#[derive(Debug, Encode, Decode)]
	pub struct ZKRng<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> RngCore for ZKRng<T> {
		fn next_u32(&mut self) -> u32 {
			let nonce = Nonce::<T>::get();
			let (random_seed, _) = T::RandomnessSource::random(&nonce.to_le_bytes());
			let random_value = u32::from_le_bytes(random_seed.as_ref()[0..4].try_into().unwrap());
			RandomNumber::<T>::put(random_value);
			Nonce::<T>::put(nonce + 1);
			random_value
		}

		fn next_u64(&mut self) -> u64 {
			let nonce = Nonce::<T>::get();
			let (random_seed, _) = T::RandomnessSource::random(&nonce.to_le_bytes());
			let random_value = u64::from_le_bytes(random_seed.as_ref()[0..8].try_into().unwrap());
			RandomNumber64::<T>::put(random_value);
			Nonce::<T>::put(nonce + 1);
			random_value
		}

		fn fill_bytes(&mut self, dest: &mut [u8]) {
			impls::fill_bytes_via_next(self, dest)
		}

		fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
			Ok(self.fill_bytes(dest))
		}
	}

	pub fn create_rng<T>() -> ZKRng<T> {
		ZKRng(sp_std::marker::PhantomData)
	}


	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_block_number: BlockNumberFor<T>) -> Weight {
			let mut rng: ZKRng<T> = ZKRng(PhantomData);
			let random_value = rng.next_u32();
			Self::deposit_event(pallet::Event::<_>::RandomNumberGenerated(random_value));
			OnInitialize::<T>::insert(&frame_system::Pallet::<T>::block_number(), random_value);
			Weight::from_parts(0, 0)
		}
	}

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
		RngWaring,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::generate_random_number())]
		pub fn generate_random_number(_origin: OriginFor<T>) -> DispatchResult {
			let mut rng: ZKRng<T> = ZKRng(PhantomData);
			let random_value = rng.next_u32();
			Self::deposit_event(pallet::Event::<_>::RandomNumberGenerated(random_value));

			Ok(())
		}
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::generate_random_number64())]
		pub fn generate_random_number64(_origin: OriginFor<T>) -> DispatchResult {
			let mut rng: ZKRng<T> = ZKRng(PhantomData);
			rng.next_u64();

			Ok(())
		}

	

		
	}
}
