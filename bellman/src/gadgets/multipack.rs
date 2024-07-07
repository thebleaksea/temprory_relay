//! Helpers for packing vectors of bits into scalar field elements.

use super::boolean::Boolean;
use super::num::Num;
use super::Assignment;
use crate::{ConstraintSystem, SynthesisError};
use arrayvec::ArrayString;
use core::fmt::Write;
use ff::PrimeField;
pub use sp_std::vec;
pub use sp_std::vec::Vec;

/// Takes a sequence of booleans and exposes them as compact
/// public inputs
pub fn pack_into_inputs<Scalar, CS>(mut cs: CS, bits: &[Boolean]) -> Result<(), SynthesisError>
where
	Scalar: PrimeField,
	CS: ConstraintSystem<Scalar>,
{
	for (i, bits) in bits.chunks(Scalar::CAPACITY as usize).enumerate() {
		let mut num = Num::<Scalar>::zero();
		let mut coeff = Scalar::ONE;
		for bit in bits {
			num = num.add_bool_with_coeff(CS::one(), bit, coeff);

			coeff = coeff.double();
		}

		let mut buf = ArrayString::<32>::new();
		write!(&mut buf, "input {}", i).unwrap();
		let s = buf.as_str();
		let input =
			cs.alloc_input(|| s, || Ok(*num.get_value().get()?))?;
		let mut bbuf = ArrayString::<32>::new();
		write!(&mut bbuf, "packing constraint {}", i).unwrap();
		let ss = bbuf.as_str();

		// num * 1 = input
		cs.enforce(
			|| ss,
			|_| num.lc(Scalar::ONE),
			|lc| lc + CS::one(),
			|lc| lc + input,
		);
	}

	Ok(())
}

pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
	bytes
		.iter()
		.flat_map(|&v| (0..8).rev().map(move |i| (v >> i) & 1 == 1))
		.collect()
}

pub fn bytes_to_bits_le(bytes: &[u8]) -> Vec<bool> {
	bytes.iter().flat_map(|&v| (0..8).map(move |i| (v >> i) & 1 == 1)).collect()
}

pub fn compute_multipacking<Scalar: PrimeField>(bits: &[bool]) -> Vec<Scalar> {
	let mut result = vec![];

	for bits in bits.chunks(Scalar::CAPACITY as usize) {
		let mut cur = Scalar::ZERO;
		let mut coeff = Scalar::ONE;

		for bit in bits {
			if *bit {
				cur.add_assign(&coeff);
			}

			coeff = coeff.double();
		}

		result.push(cur);
	}

	result
}

#[test]
fn test_multipacking() {
	use crate::ConstraintSystem;
	use bls12_381::Scalar;
	use rand_core::{RngCore, SeedableRng};
	use rand_xorshift::XorShiftRng;

	use super::boolean::{AllocatedBit, Boolean};
	use crate::gadgets::test::*;

	let mut rng = XorShiftRng::from_seed([
		0x59, 0x62, 0xbe, 0x3d, 0x76, 0x3d, 0x31, 0x8d, 0x17, 0xdb, 0x37, 0x32, 0x54, 0x06, 0xbc,
		0xe5,
	]);

	for num_bits in 0..1500 {
		let mut cs = TestConstraintSystem::<Scalar>::new();

		let bits: Vec<bool> = (0..num_bits).map(|_| rng.next_u32() % 2 != 0).collect();

		let mut buf = ArrayString::<32>::new();
		write!(&mut buf, "bit {}", i).unwrap();
		let s = buf.as_str();

		let circuit_bits = bits
			.iter()
			.enumerate()
			.map(|(i, &b)| {
				Boolean::from(
					AllocatedBit::alloc(
						cs.namespace(|| s),
						Some(b),
					)
					.unwrap(),
				)
			})
			.collect::<Vec<_>>();

		let expected_inputs = compute_multipacking(&bits);

		pack_into_inputs(cs.namespace(|| "pack"), &circuit_bits).unwrap();

		assert!(cs.is_satisfied());
		assert!(cs.verify(&expected_inputs));
	}
}
