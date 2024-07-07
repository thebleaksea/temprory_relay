use crate as pallet_zkproof;
use pallet_randrng;
// use crate as pallet_insecure_randomness_collective_flip;
use frame_support::traits::{ConstU16, ConstU64};
use sp_core::H256;
use frame_support::traits::Randomness;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

use crate::{LockVolume, AccountBalance, TranscationRecognition};

// use frame_support::pallet_prelude::IsType;
// use pallet_randrng::Event;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;



// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        System: frame_system,
        Zkproof: pallet_zkproof,
		RandRng: pallet_randrng,
		// RandomnessCollectiveFlip: pallet_insecure_randomness_collective_flip,
    }
);



impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


impl pallet_zkproof::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
}


impl pallet_randrng::Config for Test{
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    type RandomnessSource = TestRandomnessSource;
}

pub struct TestRandomnessSource;

impl Randomness<H256, u64> for TestRandomnessSource {
    fn random(_subject: &[u8]) -> (H256, u64) {
        // 在这里返回一个固定的哈希值和一个固定的 u64 值
        (H256::default(), 0)
    }
}



// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    #[allow(unused_mut)]
	let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
    LockVolume::<Test>::insert(1, 1000);
    LockVolume::<Test>::insert(2, 1000);
    AccountBalance::<Test>::insert(1, 1000);
    AccountBalance::<Test>::insert(2, 1000);
    TranscationRecognition::<Test>::insert(1, true);
    
    t.into()
}
