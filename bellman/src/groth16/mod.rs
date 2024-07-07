//! The [Groth16] proving system.
//!
//! [Groth16]: https://eprint.iacr.org/2016/260

// use group::{prime::PrimeCurveAffine, GroupEncoding, UncompressedEncoding};
use group::{GroupEncoding, UncompressedEncoding};
use pairing::{Engine, MultiMillerLoop};

use crate::SynthesisError;

use crate::multiexp::SourceBuilder;
// use byteorder::{BigEndian};

// ReadBytesExt, WriteBytesExt
// use sp_std::convert::TryInto;
use sp_std::sync::Arc;
// use sp_std::vec;
use sp_std::vec::Vec;

#[cfg(test)]
mod tests;

mod generator;
mod prover;
mod verifier;

pub use self::generator::*;
pub use self::prover::*;
pub use self::verifier::*;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Groth16Error {
	message: &'static str,
}

impl Groth16Error {
	pub fn new(message: &'static str) -> Self {
		Groth16Error { message }
	}
}

#[derive(Clone, Debug)]
pub struct Proof<E: Engine> {
	pub a: E::G1Affine,
	pub b: E::G2Affine,
	pub c: E::G1Affine,
}

impl<E: Engine> PartialEq for Proof<E> {
	fn eq(&self, other: &Self) -> bool {
		self.a == other.a && self.b == other.b && self.c == other.c
	}
}
// export 方法返回三个 Vec<u8> 类型的数据，分别对应 a、b、c 字段的字节表示。
// import 方法接受三个 &[u8] 类型的参数，分别表示 a、b、c 字段的字节表示，并根据这些字节数据构造一个新的 Proof 结构体。
impl<E: Engine> Proof<E> {
	pub fn export(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
		let a_bytes = Vec::from(self.a.to_bytes().as_ref());
		let b_bytes = Vec::from(self.b.to_bytes().as_ref());
		let c_bytes = Vec::from(self.c.to_bytes().as_ref());

		(a_bytes, b_bytes, c_bytes)
	}

	pub fn import(a_bytes: &[u8], b_bytes: &[u8], c_bytes: &[u8]) -> Result<Self, Groth16Error> {
		let mut g1_repr = <E::G1Affine as GroupEncoding>::Repr::default();
		let mut g2_repr = <E::G2Affine as GroupEncoding>::Repr::default();

		#[allow(unused_assignments)]
		let mut a = E::G1Affine::from_bytes(&g1_repr);
		#[allow(unused_assignments)]
		let mut b = E::G2Affine::from_bytes(&g2_repr);
		#[allow(unused_assignments)]
		let mut c = E::G1Affine::from_bytes(&g1_repr);

		if !a_bytes.is_empty() {
			g1_repr.as_mut().copy_from_slice(&a_bytes);
			let g1 = E::G1Affine::from_bytes(&g1_repr);
			a = g1;
		} else {
			return Err(Groth16Error::new("Invalid G1"));
		}

		if !b_bytes.is_empty() {
			g2_repr.as_mut().copy_from_slice(&b_bytes);
			let g2 = E::G2Affine::from_bytes(&g2_repr);
			b = g2;
		} else {
			return Err(Groth16Error::new("Invalid G2"));
		}

		if !c_bytes.is_empty() {
			g1_repr.as_mut().copy_from_slice(&c_bytes);
			let g1 = E::G1Affine::from_bytes(&g1_repr);
			c = g1;
		} else {
			return Err(Groth16Error::new("Invalid G1"));
		}

		Ok(Proof { a: a.unwrap(), b: b.unwrap(), c: c.unwrap() })
	}
}

#[derive(Clone)]
pub struct VerifyingKey<E: Engine> {
	// alpha in g1 for verifying and for creating A/C elements of
	// proof. Never the point at infinity.
	pub alpha_g1: E::G1Affine,

	// beta in g1 and g2 for verifying and for creating B/C elements
	// of proof. Never the point at infinity.
	pub beta_g1: E::G1Affine,
	pub beta_g2: E::G2Affine,

	// gamma in g2 for verifying. Never the point at infinity.
	pub gamma_g2: E::G2Affine,

	// delta in g1/g2 for verifying and proving, essentially the magic
	// trapdoor that forces the prover to evaluate the C element of the
	// proof with only components from the CRS. Never the point at
	// infinity.
	pub delta_g1: E::G1Affine,
	pub delta_g2: E::G2Affine,

	// Elements of the form (beta * u_i(tau) + alpha v_i(tau) + w_i(tau)) / gamma
	// for all public inputs. Because all public inputs have a dummy constraint,
	// this is the same size as the number of inputs, and never contains points
	// at infinity.
	pub ic: Vec<E::G1Affine>,
}

impl<E: Engine> PartialEq for VerifyingKey<E> {
	fn eq(&self, other: &Self) -> bool {
		self.alpha_g1 == other.alpha_g1
			&& self.beta_g1 == other.beta_g1
			&& self.beta_g2 == other.beta_g2
			&& self.gamma_g2 == other.gamma_g2
			&& self.delta_g1 == other.delta_g1
			&& self.delta_g2 == other.delta_g2
			&& self.ic == other.ic
	}
}

impl<E: Engine> VerifyingKey<E> {
	pub fn export(&self) -> (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, Vec<Vec<u8>>) {
		let alpha_g1_bytes = Vec::from(self.alpha_g1.to_uncompressed().as_ref());
		let beta_g1_bytes = Vec::from(self.beta_g1.to_uncompressed().as_ref());
		let beta_g2_bytes = Vec::from(self.beta_g2.to_uncompressed().as_ref());
		let gamma_g2_bytes = Vec::from(self.gamma_g2.to_uncompressed().as_ref());
		let delta_g1_bytes = Vec::from(self.delta_g1.to_uncompressed().as_ref());
		let delta_g2_bytes = Vec::from(self.delta_g2.to_uncompressed().as_ref());
		let ic_bytes: Vec<Vec<u8>> =
			self.ic.iter().map(|g1| Vec::from(g1.to_uncompressed().as_ref())).collect();

		(
			alpha_g1_bytes,
			beta_g1_bytes,
			beta_g2_bytes,
			gamma_g2_bytes,
			delta_g1_bytes,
			delta_g2_bytes,
			ic_bytes,
		)
	}

	pub fn import(
		alpha_g1_bytes: &[u8],
		beta_g1_bytes: &[u8],
		beta_g2_bytes: &[u8],
		gamma_g2_bytes: &[u8],
		delta_g1_bytes: &[u8],
		delta_g2_bytes: &[u8],
		ic_bytes: Vec<Vec<u8>>,
	) -> Result<Self, Groth16Error> {
		let mut g1_repr = <E::G1Affine as UncompressedEncoding>::Uncompressed::default();
		let mut g2_repr = <E::G2Affine as UncompressedEncoding>::Uncompressed::default();

		#[allow(unused_assignments)]
		let mut alpha_g1 = E::G1Affine::from_uncompressed(&g1_repr);
		#[allow(unused_assignments)]
		let mut beta_g1 = E::G1Affine::from_uncompressed(&g1_repr);
		#[allow(unused_assignments)]
		let mut beta_g2 = E::G2Affine::from_uncompressed(&g2_repr);
		#[allow(unused_assignments)]
		let mut gamma_g2 = E::G2Affine::from_uncompressed(&g2_repr);
		#[allow(unused_assignments)]
		let mut delta_g1 = E::G1Affine::from_uncompressed(&g1_repr);
		#[allow(unused_assignments)]
		let mut delta_g2 = E::G2Affine::from_uncompressed(&g2_repr);

		if !alpha_g1_bytes.is_empty() {
			g1_repr.as_mut().copy_from_slice(alpha_g1_bytes);
			let result = E::G1Affine::from_uncompressed(&g1_repr);
			if result.is_some().into() {
				alpha_g1 = result;
			} else {
				return Err(Groth16Error::new("invalid G1 alpha_g1"));
			}
		} else {
			return Err(Groth16Error::new("alpha_g1 is empty"));
		}

		if !beta_g1_bytes.is_empty() {
			g1_repr.as_mut().copy_from_slice(beta_g1_bytes);
			beta_g1 = E::G1Affine::from_uncompressed(&g1_repr);
			if beta_g1.is_some().into() {
				beta_g1 = beta_g1;
			} else {
				return Err(Groth16Error::new("invalid G1 beta_g1"));
			}
		} else {
			return Err(Groth16Error::new("beta_g1 is empty"));
		}

		if !beta_g2_bytes.is_empty() {
			g2_repr.as_mut().copy_from_slice(beta_g2_bytes);
			beta_g2 = E::G2Affine::from_uncompressed(&g2_repr);
			if beta_g2.is_some().into() {
				beta_g2 = beta_g2;
			} else {
				return Err(Groth16Error::new("invalid G2 beta_g2"));
			}
		} else {
			return Err(Groth16Error::new("beta_g2 is empty"));
		}

		if !gamma_g2_bytes.is_empty() {
			g2_repr.as_mut().copy_from_slice(gamma_g2_bytes);
			gamma_g2 = E::G2Affine::from_uncompressed(&g2_repr);
			if gamma_g2.is_some().into() {
				gamma_g2 = gamma_g2;
			} else {
				return Err(Groth16Error::new("invalid G2 gamma_g2"));
			}
		} else {
			return Err(Groth16Error::new("gamma_g2 is empty"));
		}

		if !delta_g1_bytes.is_empty() {
			g1_repr.as_mut().copy_from_slice(delta_g1_bytes);
			delta_g1 = E::G1Affine::from_uncompressed(&g1_repr);
			if delta_g1.is_some().into() {
				delta_g1 = delta_g1;
			} else {
				return Err(Groth16Error::new("invalid G1 delta_g1"));
			}
		} else {
			return Err(Groth16Error::new("delta_g1 is empty"));
		}

		if !delta_g2_bytes.is_empty() {
			g2_repr.as_mut().copy_from_slice(delta_g2_bytes);
			delta_g2 = E::G2Affine::from_uncompressed(&g2_repr);
			if delta_g2.is_some().into() {
				delta_g2 = delta_g2;
			} else {
				return Err(Groth16Error::new("invalid G2 delta_g2"));
			}
		} else {
			return Err(Groth16Error::new("delta_g2 is empty"));
		}

		
            let ic: Result<Vec<E::G1Affine>, Groth16Error> = ic_bytes
    .into_iter()
    .map(|ic_bytes| {
        g1_repr.as_mut().copy_from_slice(&ic_bytes);
        let g1 = E::G1Affine::from_uncompressed(&g1_repr);
        if g1.is_some().into() {
            Ok(g1.unwrap())
        } else {
            Err(Groth16Error::new("point at infinity"))
        }
    })
    .collect();

		Ok(VerifyingKey { 
			alpha_g1: alpha_g1.unwrap(), 
			beta_g1: beta_g1.unwrap(),
			beta_g2: beta_g2.unwrap(),
			gamma_g2: gamma_g2.unwrap(), 
			delta_g1: delta_g1.unwrap(), 
			delta_g2: delta_g2.unwrap(), 
			ic: ic? 
		})
	}
}
#[derive(Clone)]
pub struct Parameters<E: Engine> {
	pub vk: VerifyingKey<E>,

	// Elements of the form ((tau^i * t(tau)) / delta) for i between 0 and
	// m-2 inclusive. Never contains points at infinity.
	pub h: Arc<Vec<E::G1Affine>>,

	// Elements of the form (beta * u_i(tau) + alpha v_i(tau) + w_i(tau)) / delta
	// for all auxiliary inputs. Variables can never be unconstrained, so this
	// never contains points at infinity.
	pub l: Arc<Vec<E::G1Affine>>,

	// QAP "A" polynomials evaluated at tau in the Lagrange basis. Never contains
	// points at infinity: polynomials that evaluate to zero are omitted from
	// the CRS and the prover can deterministically skip their evaluation.
	pub a: Arc<Vec<E::G1Affine>>,

	// QAP "B" polynomials evaluated at tau in the Lagrange basis. Needed in
	// G1 and G2 for C/B queries, respectively. Never contains points at
	// infinity for the same reason as the "A" polynomials.
	pub b_g1: Arc<Vec<E::G1Affine>>,
	pub b_g2: Arc<Vec<E::G2Affine>>,
}

impl<E: Engine> PartialEq for Parameters<E> {
	fn eq(&self, other: &Self) -> bool {
		self.vk == other.vk
			&& self.h == other.h
			&& self.l == other.l
			&& self.a == other.a
			&& self.b_g1 == other.b_g1
			&& self.b_g2 == other.b_g2
	}
}

impl<E: Engine> Parameters<E> {
    pub fn export(
        &self,
    ) -> (
        VerifyingKey<E>,
        Vec<Vec<u8>>,
        Vec<Vec<u8>>,
        Vec<Vec<u8>>,
        Vec<Vec<u8>>,
        Vec<Vec<u8>>,
    ) {
        let vk = &self.vk;

        let h_bytes: Vec<Vec<u8>> = self
            .h
            .iter()
            .map(|g| Vec::from(g.to_uncompressed().as_ref()))
            .collect();
        let l_bytes: Vec<Vec<u8>> = self
            .l
            .iter()
            .map(|g| Vec::from(g.to_uncompressed().as_ref()))
            .collect();
        let a_bytes: Vec<Vec<u8>> = self
            .a
            .iter()
            .map(|g| Vec::from(g.to_uncompressed().as_ref()))
            .collect();
        let b_g1_bytes: Vec<Vec<u8>> = self
            .b_g1
            .iter()
            .map(|g| Vec::from(g.to_uncompressed().as_ref()))
            .collect();
        let b_g2_bytes: Vec<Vec<u8>> = self
            .b_g2
            .iter()
            .map(|g| Vec::from(g.to_uncompressed().as_ref()))
            .collect();

        (vk.clone(), h_bytes, l_bytes, a_bytes, b_g1_bytes, b_g2_bytes)
    }

    pub fn import(
        vk: VerifyingKey<E>,
        h_bytes: Vec<Vec<u8>>,
        l_bytes: Vec<Vec<u8>>,
        a_bytes: Vec<Vec<u8>>,
        b_g1_bytes: Vec<Vec<u8>>,
        b_g2_bytes: Vec<Vec<u8>>,
    ) -> Result<Self, Groth16Error> {
        let mut g1_repr = <E::G1Affine as UncompressedEncoding>::Uncompressed::default();
        let mut g2_repr = <E::G2Affine as UncompressedEncoding>::Uncompressed::default();

        let h: Result<Vec<E::G1Affine>, Groth16Error> = h_bytes
            .into_iter()
            .map(|h_bytes| {
                g1_repr.as_mut().copy_from_slice(&h_bytes); 
                let g1 = E::G1Affine::from_uncompressed(&g1_repr);
                if g1.is_some().into() {
                    Ok(g1.unwrap())
                } else {
                    Err(Groth16Error::new("point at infinity"))
                }
            })
            .collect();

        let l: Result<Vec<E::G1Affine>, Groth16Error> = l_bytes
            .into_iter()
            .map(|l_bytes| {
                g1_repr.as_mut().copy_from_slice(&l_bytes); 
                let g1 = E::G1Affine::from_uncompressed(&g1_repr);
                if g1.is_some().into() {
                    Ok(g1.unwrap())
                } else {
                    Err(Groth16Error::new("point at infinity"))
                }
            })
            .collect();
        
        let a: Result<Vec<E::G1Affine>, Groth16Error> = a_bytes
            .into_iter()
            .map(|a_bytes| {
                g1_repr.as_mut().copy_from_slice(&a_bytes); 
                let g1 = E::G1Affine::from_uncompressed(&g1_repr);
                if g1.is_some().into() {
                    Ok(g1.unwrap())
                } else {
                    Err(Groth16Error::new("point at infinity"))
                }
            })
            .collect();
        
        let b_g1: Result<Vec<E::G1Affine>, Groth16Error> = b_g1_bytes
            .into_iter()
            .map(|b_g1_bytes| {
                g1_repr.as_mut().copy_from_slice(&b_g1_bytes); 
                let g1 = E::G1Affine::from_uncompressed(&g1_repr);
                if g1.is_some().into() {
                    Ok(g1.unwrap())
                } else {
                    Err(Groth16Error::new("point at infinity"))
                }
            })
            .collect();

        let b_g2: Result<Vec<E::G2Affine>, Groth16Error> = b_g2_bytes
            .into_iter()
            .map(|b_g2_bytes| {
                g2_repr.as_mut().copy_from_slice(&b_g2_bytes); 
                let g2 = E::G2Affine::from_uncompressed(&g2_repr);
                if g2.is_some().into() {
                    Ok(g2.unwrap())
                } else {
                    Err(Groth16Error::new("point at infinity"))
                }
            })
            .collect();


        Ok(Parameters {
            vk,
            h: Arc::new(h?),
            l: Arc::new(l?),
            a: Arc::new(a?),
            b_g1: Arc::new(b_g1?),
            b_g2: Arc::new(b_g2?),
        })
    }
}

pub struct PreparedVerifyingKey<E: MultiMillerLoop> {
    /// Pairing result of alpha*beta
    alpha_g1_beta_g2: E::Gt,
    /// -gamma in G2
    neg_gamma_g2: E::G2Prepared,
    /// -delta in G2
    neg_delta_g2: E::G2Prepared,
    /// Copy of IC from `VerifiyingKey`.
    ic: Vec<E::G1Affine>,
}

pub trait ParameterSource<E: Engine> {
	type G1Builder: SourceBuilder<E::G1Affine>;
	type G2Builder: SourceBuilder<E::G2Affine>;

	fn get_vk(&mut self, num_ic: usize) -> Result<VerifyingKey<E>, SynthesisError>;
	fn get_h(&mut self, num_h: usize) -> Result<Self::G1Builder, SynthesisError>;
	fn get_l(&mut self, num_l: usize) -> Result<Self::G1Builder, SynthesisError>;
	fn get_a(
		&mut self,
		num_inputs: usize,
		num_aux: usize,
	) -> Result<(Self::G1Builder, Self::G1Builder), SynthesisError>;
	fn get_b_g1(
		&mut self,
		num_inputs: usize,
		num_aux: usize,
	) -> Result<(Self::G1Builder, Self::G1Builder), SynthesisError>;
	fn get_b_g2(
		&mut self,
		num_inputs: usize,
		num_aux: usize,
	) -> Result<(Self::G2Builder, Self::G2Builder), SynthesisError>;
}


impl<'a, E: Engine> ParameterSource<E> for &'a Parameters<E> {
	type G1Builder = (Arc<Vec<E::G1Affine>>, usize);
	type G2Builder = (Arc<Vec<E::G2Affine>>, usize);

	fn get_vk(&mut self, _: usize) -> Result<VerifyingKey<E>, SynthesisError> {
		Ok(self.vk.clone())
	}

	fn get_h(&mut self, _: usize) -> Result<Self::G1Builder, SynthesisError> {
		Ok((self.h.clone(), 0))
	}

	fn get_l(&mut self, _: usize) -> Result<Self::G1Builder, SynthesisError> {
		Ok((self.l.clone(), 0))
	}

	fn get_a(
		&mut self,
		num_inputs: usize,
		_: usize,
	) -> Result<(Self::G1Builder, Self::G1Builder), SynthesisError> {
		Ok(((self.a.clone(), 0), (self.a.clone(), num_inputs)))
	}

	fn get_b_g1(
		&mut self,
		num_inputs: usize,
		_: usize,
	) -> Result<(Self::G1Builder, Self::G1Builder), SynthesisError> {
		Ok(((self.b_g1.clone(), 0), (self.b_g1.clone(), num_inputs)))
	}

	fn get_b_g2(
		&mut self,
		num_inputs: usize,
		_: usize,
	) -> Result<(Self::G2Builder, Self::G2Builder), SynthesisError> {
		Ok(((self.b_g2.clone(), 0), (self.b_g2.clone(), num_inputs)))
	}
}

#[cfg(test)]
mod test_with_bls12_381 {
	use super::*;
	use crate::{Circuit, ConstraintSystem, SynthesisError};

	use bls12_381::{Bls12, Scalar};
	use ff::{Field, PrimeField};
	use rand::thread_rng;
	use sp_std::ops::MulAssign;

	#[test]
	fn serialization() {
		struct MySillyCircuit<Scalar: PrimeField> {
			a: Option<Scalar>,
			b: Option<Scalar>,
		}

		impl<Scalar: PrimeField> Circuit<Scalar> for MySillyCircuit<Scalar> {
			fn synthesize<CS: ConstraintSystem<Scalar>>(
				self,
				cs: &mut CS,
			) -> Result<(), SynthesisError> {
				let a = cs.alloc(|| "a", || self.a.ok_or(SynthesisError::AssignmentMissing))?;
				let b = cs.alloc(|| "b", || self.b.ok_or(SynthesisError::AssignmentMissing))?;
				let c = cs.alloc_input(
					|| "c",
					|| {
						let mut a = self.a.ok_or(SynthesisError::AssignmentMissing)?;
						let b = self.b.ok_or(SynthesisError::AssignmentMissing)?;

						a.mul_assign(&b);
						Ok(a)
					},
				)?;

				cs.enforce(|| "a*b=c", |lc| lc + a, |lc| lc + b, |lc| lc + c);

				Ok(())
			}
		}

		let mut rng = thread_rng();

		let params = generate_random_parameters::<Bls12, _, _>(
			MySillyCircuit { a: None, b: None },
			&mut rng,
		)
		.unwrap();

		{
			let mut v = vec![];

			params.write(&mut v).unwrap();
			assert_eq!(v.len(), 2136);

			let de_params = Parameters::read(&v[..], true).unwrap();
			assert!(params == de_params);

			let de_params = Parameters::read(&v[..], false).unwrap();
			assert!(params == de_params);
		}

		let pvk = prepare_verifying_key::<Bls12>(&params.vk);

		for _ in 0..100 {
			let a = Scalar::random(&mut rng);
			let b = Scalar::random(&mut rng);
			let mut c = a;
			c.mul_assign(&b);

			let proof =
				create_random_proof(MySillyCircuit { a: Some(a), b: Some(b) }, &params, &mut rng)
					.unwrap();

			let mut v = vec![];
			proof.write(&mut v).unwrap();

			assert_eq!(v.len(), 192);

			let de_proof = Proof::read(&v[..]).unwrap();
			assert!(proof == de_proof);

			assert!(verify_proof(&pvk, &proof, &[c]).is_ok());
			assert!(verify_proof(&pvk, &proof, &[a]).is_err());
		}
	}
}
