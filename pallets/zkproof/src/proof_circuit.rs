use bellman::{
	// groth16::{
	// 	verify_proof,Proof,PreparedVerifyingKey,
	// },
	Circuit, ConstraintSystem, SynthesisError,
};
// use bls12_381::{Bls12, Scalar, G1Affine, G1Projective};
use bls12_381::{Scalar, G1Affine, G1Projective};
use ff::{PrimeField};

use super::g1_to_scalar::glaffine_to_primefield;



#[derive(Debug,Clone,Copy)]
pub struct PtradCircuit<S: PrimeField> {
    pub private_key_alice: Option<Scalar>,
    pub public_key_alice: Option<G1Affine>,
    pub public_key_bob: Option<G1Affine>,
    pub trad_volume: Option<S>,
    pub lock_volume: Option<S>,
}

#[derive(Debug,Clone,Copy)]
pub struct TranscationCircle<S: PrimeField> {
    pub private_key_alice: Option<Scalar>,
    pub public_key_alice: Option<G1Affine>,
    pub private_key_bob: Option<Scalar>,
    pub public_key_bob: Option<G1Affine>,
    pub trad_volume_alice: Option<S>,
    pub trad_volume_bob: Option<S>,
    pub trad_volume_alice_give: Option<S>,
    pub trad_volume_alice_get: Option<S>,
    pub trad_volume_bob_give: Option<S>,
    pub trad_volume_bob_get: Option<S>,
}

#[derive(Debug,Clone,Copy)]
pub struct StoreCircuit<S: PrimeField> {
    pub private_key_alice: Option<Scalar>,
    pub public_key_alice: Option<G1Affine>,
    pub trad_volume_alice_get: Option<S>,
    pub trad_volume_alice_give: Option<S>,
    pub trad_volume_alice: Option<S>,
    pub trad_volume_bob_give: Option<S>,
    // transcation_proof: Option<Proof<Bls12>>,
    // transcation_random_key: PreparedVerifyingKey<Bls12>,
}

impl<S:PrimeField> Circuit<S>for StoreCircuit<S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        // 计算alice的公私钥是否正确，证明此交易是alice执行的
        let private_key_alice_value = (self.private_key_alice).unwrap();
        let compute_public_key_alice_value = G1Affine::from(G1Projective::generator() * private_key_alice_value);
        let compute_public_key_alice_change = Some(glaffine_to_primefield(compute_public_key_alice_value));
        let compute_public_key_alice = cs.alloc(
            || "compute_public_key_alice",
            || compute_public_key_alice_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let public_key_alice_value = (self.public_key_alice).unwrap();
        let public_key_alice_change = Some(glaffine_to_primefield(public_key_alice_value));
        let public_key_alice = cs.alloc(
            || "public_key_alice",
            || public_key_alice_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "compute_public_key_alice = public_key_alice",
            |lc| lc + compute_public_key_alice,
            |lc| lc + CS::one(),
            |lc| lc + public_key_alice,
        );
        // alice的交易,检查是否alice交易剩余+alice交易给出=alice总铸币
        let trad_volume_alice_value = self.trad_volume_alice;
        let trad_volume_alice = cs.alloc(
            || "trad_volume_alice",
            || trad_volume_alice_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let trad_volume_alice_give_value = self.trad_volume_alice_give;
        let trad_volume_alice_give = cs.alloc(
            || "trad_volume_alice_give",
            || trad_volume_alice_give_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let trad_volume_alice_get_value = self.trad_volume_alice_get;
        let trad_volume_alice_get = cs.alloc(
            || "trad_volume_alice_get",
            || trad_volume_alice_get_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "trad_volume_alice = trad_volume_alice_give + trad_volume_alice_get",
            |lc| lc + trad_volume_alice_give,
            |lc| lc + CS::one(),
            |lc| lc + trad_volume_alice - trad_volume_alice_get,
        );
        
        // //检查交易证明是否合法
        // let alice_proof = (self.transcation_proof).unwrap();
        // let alice_randome_key = self.transcation_random_key;
        // let alice_verification_result = verify_proof(&alice_randome_key, &alice_proof, &[]);
        // let alice_proof_var = cs.alloc(
        //     || "alice_proof",
        //     || if alice_verification_result.is_ok() {Ok(PrimeField::from_u128(1u128))} else {Ok(PrimeField::from_u128(0u128))},
        // )?;
        // let alice_valid_var = cs.alloc(
        //     || "alice_valid",
        //     || Ok(PrimeField::from_u128(1u128)),
        // )?;
        // cs.enforce(
        //     || "alice_valid = alice_proof_var",
        //     |lc| lc + alice_proof_var,
        //     |lc| lc + CS::one(),
        //     |lc| lc + alice_valid_var,
        // );

        Ok(())
    }
}


impl<S:PrimeField> Circuit<S>for TranscationCircle<S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        // alice的私钥与bob的公钥执行判断
        let private_key_alice_value = (self.private_key_alice).unwrap();
        let compute_public_key_alice_value = G1Affine::from(G1Projective::generator() * private_key_alice_value);
        let compute_public_key_alice_change = Some(glaffine_to_primefield(compute_public_key_alice_value));
        let compute_public_key_alice = cs.alloc(
            || "compute_public_key_alice",
            || compute_public_key_alice_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let public_key_bob_value = (self.public_key_bob).unwrap();
        let public_key_bob_change = Some(glaffine_to_primefield(public_key_bob_value));
        let public_key_bob = cs.alloc(
            || "public_key_bob",
            || public_key_bob_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "public_key_bob = compute_public_key_alice",
            |lc| lc + public_key_bob,
            |lc| lc + CS::one(),
            |lc| lc + compute_public_key_alice,
        );
        // bob的私钥与alice的公钥执行判断
        let private_key_bob_value = (self.private_key_bob).unwrap();
        let compute_public_key_bob_value = G1Affine::from(G1Projective::generator() * private_key_bob_value);
        let compute_public_key_bob_change = Some(glaffine_to_primefield(compute_public_key_bob_value));
        let compute_public_key_bob = cs.alloc(
            || "compute_public_key_bob",
            || compute_public_key_bob_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let public_key_alice_value = (self.public_key_alice).unwrap();
        let public_key_alice_change = Some(glaffine_to_primefield(public_key_alice_value));
        let public_key_alice = cs.alloc(
            || "public_key_alice",
            || public_key_alice_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "public_key_alice = compute_public_key_bob",
            |lc| lc + public_key_alice,
            |lc| lc + CS::one(),
            |lc| lc + compute_public_key_bob,
        );
        // alice的交易,检查是否alice交易剩余+alice交易给出=alice总铸币
        let trad_volume_alice_value = self.trad_volume_alice;
        let trad_volume_alice = cs.alloc(
            || "trad_volume_alice",
            || trad_volume_alice_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let trad_volume_alice_give_value = self.trad_volume_alice_give;
        let trad_volume_alice_give = cs.alloc(
            || "trad_volume_alice_give",
            || trad_volume_alice_give_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let trad_volume_alice_get_value = self.trad_volume_alice_get;
        let trad_volume_alice_get = cs.alloc(
            || "trad_volume_alice_get",
            || trad_volume_alice_get_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "trad_volume_alice = trad_volume_alice_give + trad_volume_alice_get",
            |lc| lc + trad_volume_alice_give,
            |lc| lc + CS::one(),
            |lc| lc + trad_volume_alice - trad_volume_alice_get,
        );
        // bob的交易,检查是否bob交易剩余+bob交易给出=bob总铸币
        let trad_volume_bob_value = self.trad_volume_bob;
        let trad_volume_bob = cs.alloc(
            || "trad_volume_bob",
            || trad_volume_bob_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let trad_volume_bob_give_value = self.trad_volume_bob_give;
        let trad_volume_bob_give = cs.alloc(
            || "trad_volume_bob_give",
            || trad_volume_bob_give_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let trad_volume_bob_get_value = self.trad_volume_bob_get;
        let trad_volume_bob_get = cs.alloc(
            || "trad_volume_bob_get",
            || trad_volume_bob_get_value.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "trad_volume_bob = trad_volume_bob_give + trad_volume_bob_get",
            |lc| lc + trad_volume_bob_give,
            |lc| lc + CS::one(),
            |lc| lc + trad_volume_bob - trad_volume_bob_get,
        );

        // // 分配alice的证明
        // let alice_proof = (self.alice_trad_proof).unwrap();
        // let alice_randome_key = self.alice_random_key;
        // let alice_verification_result = verify_proof(&alice_randome_key, &alice_proof, &[]);
        // let alice_proof_var = cs.alloc(
        //     || "alice_proof",
        //     || if alice_verification_result.is_ok() {Ok(PrimeField::from_u128(1u128))} else {Ok(PrimeField::from_u128(0u128))},
        // )?;
        // let alice_valid_var = cs.alloc(
        //     || "alice_valid",
        //     || Ok(PrimeField::from_u128(1u128)),
        // )?;
        // // 添加约束来验证 alice_proof 是否有效
        // cs.enforce(
        //     || "alice_proof is valid",
        //     |lc| lc + alice_proof_var,
        //     |lc| lc + CS::one(),
        //     |lc| lc + alice_valid_var,
        // );
        // // 分配bob的证明
        // let bob_proof = (self.bob_trad_proof).unwrap();
        // let bob_randome_key = self.bob_random_key;
        // let bob_verification_result = verify_proof(&bob_randome_key, &bob_proof, &[]);
        // let bob_proof_var = cs.alloc(
        //     || "bob_proof",
        //     || if bob_verification_result.is_ok() {Ok(PrimeField::from_u128(1u128))} else {Ok(PrimeField::from_u128(0u128))},
        // )?;
        // let bob_valid_var = cs.alloc(
        //     || "bob_valid",
        //     || Ok(PrimeField::from_u128(1u128)),
        // )?;
        // // 添加约束来验证 bob_proof 是否有效
        // cs.enforce(
        //     || "bob_proof is valid",
        //     |lc| lc + bob_proof_var,
        //     |lc| lc + CS::one(),
        //     |lc| lc + bob_valid_var,
        // );
        Ok(())

    }
}


impl <S: PrimeField> Circuit<S> for PtradCircuit<S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let private_key_alice_value = (self.private_key_alice).unwrap();
        let compute_public_key_alice_value = G1Affine::from(G1Projective::generator() * private_key_alice_value);
        let compute_public_key_alice_change = Some(glaffine_to_primefield(compute_public_key_alice_value));
        let compute_public_key_alice = cs.alloc(
            || "compute_public_key_alice",
            || compute_public_key_alice_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let public_key_alice_value = (self.public_key_alice).unwrap();
        let public_key_alice_change = Some(glaffine_to_primefield(public_key_alice_value));
        let public_key_alice = cs.alloc(
            || "public_key_alice",
            || public_key_alice_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "public_key_alice = compute_public_key_alice",
            |lc| lc + public_key_alice,
            |lc| lc + CS::one(),
            |lc| lc + compute_public_key_alice,
        );
        
        let alice_public_key_store_value = G1Affine::from(G1Projective::generator() * Scalar::from(100u64));
        let alice_public_key_store_change = Some(glaffine_to_primefield(alice_public_key_store_value));
        let alice_public_key_store_c = cs.alloc(
            || "alice_public_key_store",
            || alice_public_key_store_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "public_key_alice = alice_public_key_store",
            |lc| lc + public_key_alice,
            |lc| lc + CS::one(),
            |lc| lc + alice_public_key_store_c,
        );

        let bob_public_key_store_value = G1Affine::from(G1Projective::generator() * Scalar::from(200u64));
        let bob_public_key_store_change = Some(glaffine_to_primefield(bob_public_key_store_value));
        let bob_public_key_store_c = cs.alloc(
            || "bob_public_key_store",
            || bob_public_key_store_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        let public_key_bob_value = (self.public_key_bob).unwrap();
        let public_key_bob_change = Some(glaffine_to_primefield(public_key_bob_value));
        let public_key_bob = cs.alloc(
            || "public_key_bob",
            || public_key_bob_change.ok_or(SynthesisError::AssignmentMissing),
        )?;
        cs.enforce(
            || "public_key_bob = bob_public_key_store",
            |lc| lc + public_key_bob,
            |lc| lc + CS::one(),
            |lc| lc + bob_public_key_store_c,
        );

        let trad_volume_value = self.trad_volume;
		let trad_volume = cs.alloc(
			|| "trad_volume",
			|| trad_volume_value.ok_or(SynthesisError::AssignmentMissing),
		)?;
		let lock_volume_value = self.lock_volume;
		let lock_volume = cs.alloc(
			|| "lock_volume",
			|| lock_volume_value.ok_or(SynthesisError::AssignmentMissing),
		)?;
        cs.enforce(
            || "lock_volume =  trad_volume",
            |lc| lc + lock_volume ,
            |lc| lc + CS::one(),
            |lc| lc + trad_volume,
        );


		Ok(())

    }
}
