//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
use super::*;

#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;

#[benchmarks]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn generate_random_number64() {
		let caller: T::AccountId = whitelisted_caller();
		#[extrinsic_call]
		generate_random_number64(RawOrigin::Signed(caller));

	}
	
	impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}

