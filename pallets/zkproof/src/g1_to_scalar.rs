use bls12_381::{G1Affine,Scalar}; 
use ff::{PrimeField};

//仅仅只用于比较,不用于计算,因为这个函数会丢失精度.
fn bytes_to_u128(bytes: &[u8; 96]) -> u128 {
    let mut result = 0u128;

    for &byte in bytes {
        result = (result << 8) | byte as u128;
    }

    result
}

pub fn glaffine_to_primefield<S: PrimeField>(g1_affine: G1Affine) -> S {
    let g1_value = g1_affine.to_uncompressed();
    let g1 = bytes_to_u128(&g1_value);
    let g1_s = PrimeField::from_u128(g1);
    g1_s
}
#[allow(unused)]
pub fn glaffine_to_scalar(g1_affine: G1Affine) -> Scalar {
    let g1_value = g1_affine.to_uncompressed();
    let g1 = bytes_to_u128(&g1_value);
    let g1_s = Scalar::from_u128(g1);
    g1_s
}
#[allow(unused)]
pub fn prime_field_to_scalar<S: PrimeField>(p1: S) -> Scalar {
    // 将PrimeField转换为其表示形式
    let repr = p1.to_repr();

    // 将表示形式转换为字节数组
    let bytes = repr.as_ref();

    // 将字节数组转换为u128
    let mut u128_number = 0;
    for &byte in bytes.iter().rev() {
        u128_number = (u128_number << 8) | byte as u128;
    }

    // 将u128转换为Scalar
    let p1_s = Scalar::from_u128(u128_number);

    p1_s
}



#[allow(unused)]
pub fn is_less_or_equal_than<S: PrimeField>(a: &S, b: &S) -> bool{
    let a_repr = a.to_repr();
    let b_repr = b.to_repr();
    for (a, b) in a_repr.as_ref().iter().zip(b_repr.as_ref().iter()).rev() {
        if a < b {
            return true;
        } else if a > b {
            return false;
        }
    }
    true
}

