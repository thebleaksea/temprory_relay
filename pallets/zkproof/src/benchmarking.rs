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
    fn create_ptrade_proof(){
        let id: u32 = 1;
        let account: u128 = 1000;
        lock_volume;
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        create_ptrade_proof(RawOrigin::Signed(caller), id, account);

    }

    // #[benchmark]
    // fn verification_proof(){
    //     let id: u32 = 1;
    //     let account: u128 = 1000;
    //     lock_volume;
    //     create_ptrade_proof;
    //     let caller: T::AccountId = whitelisted_caller();
    //     #[extrinsic_call]
    //     verification_proof(RawOrigin::Signed(caller), id);
    // }

    #[benchmark]
    fn lock_volume(){
        let id: u32 = 1;
        let account: u128 = 1000;
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        lock_volume(RawOrigin::Signed(caller), id, account);
    }

    #[benchmark]
    fn create_random() {
        // 开始计时
        let caller: T::AccountId = whitelisted_caller();
        #[extrinsic_call]
        create_random(RawOrigin::Signed(caller));

    }

    impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);
}

