use frame_support::pallet_prelude::*;
use sp_std::vec::Vec;


#[derive(Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct PreMusig2{
    pub secret: [u8; 32],
    pub nonces: [u8; 32],
}
impl Default for PreMusig2{
    fn default() -> Self{
        Self{
            secret: [0; 32],
            nonces: [0; 32],
        }
    }
}
pub fn convert_to_pre_musig2(
    secret: Vec<u8>,
    nonces: Vec<u8>,
) -> PreMusig2{
    let secret = secret.try_into().expect("Vec length is not 32");
    let nonces = nonces.try_into().expect("Vec length is not 32");
    PreMusig2{secret, nonces}
}
