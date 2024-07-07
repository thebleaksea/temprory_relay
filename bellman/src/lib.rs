#![cfg_attr(not(feature = "std"), no_std)]

pub mod domain;
pub mod gadgets;
#[cfg(feature = "groth16")]
pub mod groth16;
pub mod multicore;
pub mod multiexp;

use ff::PrimeField;
use sp_std::fmt;
use sp_std::marker::PhantomData;
use sp_std::ops::{Add, Sub};
use sp_std::vec;
use sp_std::vec::Vec;

pub trait Circuit<Scalar: PrimeField> {
    /// Synthesize the circuit into a rank-1 quadratic constraint system
    fn synthesize<CS: ConstraintSystem<Scalar>>(self, cs: &mut CS) -> Result<(), SynthesisError>;
}


/// Represents a variable in our constraint system.
#[derive(Copy, Clone, Debug)]
pub struct Variable(Index);

impl Variable {
    /// This constructs a variable with an arbitrary index.
    /// Circuit implementations are not recommended to use this.
    pub fn new_unchecked(idx: Index) -> Variable {
        Variable(idx)
    }

    /// This returns the index underlying the variable.
    /// Circuit implementations are not recommended to use this.
    pub fn get_unchecked(&self) -> Index {
        self.0
    }
}

/// Represents the index of either an input variable or
/// auxiliary variable.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Index {
    Input(usize),
    Aux(usize),
}

/// This represents a linear combination of some variables, with coefficients
/// in the scalar field of a pairing-friendly elliptic curve group.
#[derive(Clone)]
pub struct LinearCombination<Scalar: PrimeField>(Vec<(Variable, Scalar)>);

impl<Scalar: PrimeField> AsRef<[(Variable, Scalar)]> for LinearCombination<Scalar> {
    fn as_ref(&self) -> &[(Variable, Scalar)] {
        &self.0
    }
}

impl<Scalar: PrimeField> LinearCombination<Scalar> {
    pub fn zero() -> LinearCombination<Scalar> {
        LinearCombination(vec![])
    }
}

impl<Scalar: PrimeField> Add<(Scalar, Variable)> for LinearCombination<Scalar> {
    type Output = LinearCombination<Scalar>;

    fn add(mut self, (coeff, var): (Scalar, Variable)) -> LinearCombination<Scalar> {
        self.0.push((var, coeff));

        self
    }
}

impl<Scalar: PrimeField> Sub<(Scalar, Variable)> for LinearCombination<Scalar> {
    type Output = LinearCombination<Scalar>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, (coeff, var): (Scalar, Variable)) -> LinearCombination<Scalar> {
        self + (coeff.neg(), var)
    }
}

impl<Scalar: PrimeField> Add<Variable> for LinearCombination<Scalar> {
    type Output = LinearCombination<Scalar>;

    fn add(self, other: Variable) -> LinearCombination<Scalar> {
        self + (Scalar::ONE, other)
    }
}

impl<Scalar: PrimeField> Sub<Variable> for LinearCombination<Scalar> {
    type Output = LinearCombination<Scalar>;

    fn sub(self, other: Variable) -> LinearCombination<Scalar> {
        self - (Scalar::ONE, other)
    }
}

impl<'a, Scalar: PrimeField> Add<&'a LinearCombination<Scalar>> for LinearCombination<Scalar> {
    type Output = LinearCombination<Scalar>;

    fn add(mut self, other: &'a LinearCombination<Scalar>) -> LinearCombination<Scalar> {
        for s in &other.0 {
            self = self + (s.1, s.0);
        }

        self
    }
}

impl<'a, Scalar: PrimeField> Sub<&'a LinearCombination<Scalar>> for LinearCombination<Scalar> {
    type Output = LinearCombination<Scalar>;

    fn sub(mut self, other: &'a LinearCombination<Scalar>) -> LinearCombination<Scalar> {
        for s in &other.0 {
            self = self - (s.1, s.0);
        }

        self
    }
}

impl<'a, Scalar: PrimeField> Add<(Scalar, &'a LinearCombination<Scalar>)>
    for LinearCombination<Scalar>
{
    type Output = LinearCombination<Scalar>;

    fn add(
        mut self,
        (coeff, other): (Scalar, &'a LinearCombination<Scalar>),
    ) -> LinearCombination<Scalar> {
        for s in &other.0 {
            let mut tmp = s.1;
            tmp.mul_assign(&coeff);
            self = self + (tmp, s.0);
        }

        self
    }
}

impl<'a, Scalar: PrimeField> Sub<(Scalar, &'a LinearCombination<Scalar>)>
    for LinearCombination<Scalar>
{
    type Output = LinearCombination<Scalar>;

    fn sub(
        mut self,
        (coeff, other): (Scalar, &'a LinearCombination<Scalar>),
    ) -> LinearCombination<Scalar> {
        for s in &other.0 {
            let mut tmp = s.1;
            tmp.mul_assign(&coeff);
            self = self - (tmp, s.0);
        }

        self
    }
}


#[derive(Debug)]
pub enum SynthesisError {
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
    IoError,
    /// During CRS generation, we observed an unconstrained auxiliary variable
    UnconstrainedVariable,
}


impl fmt::Display for SynthesisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let msg = match *self {
            SynthesisError::AssignmentMissing => "an assignment for a variable could not be computed.无法计算变量的赋值",
            SynthesisError::DivisionByZero => "division by zero.除以零",
            SynthesisError::Unsatisfiable => "unsatisfiable constraint system.约束系统不可满足",
            SynthesisError::PolynomialDegreeTooLarge => "polynomial degree is too large.多项式次数过大",
            SynthesisError::UnexpectedIdentity => "encountered an identity element in the CRS.在CRS中遇到了标识元素",
            SynthesisError::IoError => "encountered an I/O error.遇到了I/O错误",
            SynthesisError::UnconstrainedVariable => "auxiliary variable was unconstrained.辅助变量未受约束",
        };
        write!(f, "{}", msg)
    }
}

/// An error during verification.
#[derive(Debug, Clone)]
pub enum VerificationError {
    /// Verification was attempted with a malformed verifying key.
    InvalidVerifyingKey,
    /// Proof verification failed.
    InvalidProof,
}


impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let msg = match *self {
            VerificationError::InvalidVerifyingKey => "malformed verifying key验证密钥格式错误",
            VerificationError::InvalidProof => "proof verification failed证明验证失败",
        };
        write!(f, "{}", msg)
    }
}
/// Represents a constraint system which can have new variables
/// allocated and constrains between them formed.
pub trait ConstraintSystem<Scalar: PrimeField>: Sized {
    /// Represents the type of the "root" of this constraint system
    /// so that nested namespaces can minimize indirection.
    type Root: ConstraintSystem<Scalar>;

    /// Return the "one" input variable
    fn one() -> Variable {
        Variable::new_unchecked(Index::Input(0))
    }

    /// Allocate a private variable in the constraint system. The provided function is used to
    /// determine the assignment of the variable. The given `annotation` function is invoked
    /// in testing contexts in order to derive a unique name for this variable in the current
    /// namespace.
    fn alloc<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<Scalar, SynthesisError>,
        A: FnOnce() -> AR,
        AR: AsRef<str>;

    /// Allocate a public variable in the constraint system. The provided function is used to
    /// determine the assignment of the variable.
    fn alloc_input<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<Scalar, SynthesisError>,
        A: FnOnce() -> AR,
        AR: AsRef<str>;

    /// Enforce that `A` * `B` = `C`. The `annotation` function is invoked in testing contexts
    /// in order to derive a unique name for the constraint in the current namespace.
    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: AsRef<str>,
        LA: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
        LB: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
        LC: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>;

    /// Create a new (sub)namespace and enter into it. Not intended
    /// for downstream use; use `namespace` instead.
    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: AsRef<str>,
        N: FnOnce() -> NR;

    /// Exit out of the existing namespace. Not intended for
    /// downstream use; use `namespace` instead.
    fn pop_namespace(&mut self);

    /// Gets the "root" constraint system, bypassing the namespacing.
    /// Not intended for downstream use; use `namespace` instead.
    fn get_root(&mut self) -> &mut Self::Root;

    /// Begin a namespace for this constraint system.
    fn namespace<NR, N>(&mut self, name_fn: N) -> Namespace<'_, Scalar, Self::Root>
    where
        NR: AsRef<str>,
        N: FnOnce() -> NR,
    {
        self.get_root().push_namespace(name_fn);

        Namespace(self.get_root(), PhantomData)
    }
}

/// This is a "namespaced" constraint system which borrows a constraint system (pushing
/// a namespace context) and, when dropped, pops out of the namespace context.
pub struct Namespace<'a, Scalar: PrimeField, CS: ConstraintSystem<Scalar>>(
    &'a mut CS,
    PhantomData<Scalar>,
);

impl<'cs, Scalar: PrimeField, CS: ConstraintSystem<Scalar>> ConstraintSystem<Scalar>
    for Namespace<'cs, Scalar, CS>
{
    type Root = CS::Root;

    fn one() -> Variable {
        CS::one()
    }

    fn alloc<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<Scalar, SynthesisError>,
        A: FnOnce() -> AR,
        AR: AsRef<str>,
    {
        self.0.alloc(annotation, f)
    }

    fn alloc_input<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<Scalar, SynthesisError>,
        A: FnOnce() -> AR,
        AR: AsRef<str>,
    {
        self.0.alloc_input(annotation, f)
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: AsRef<str>,
        LA: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
        LB: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
        LC: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
    {
        self.0.enforce(annotation, a, b, c)
    }

    // Downstream users who use `namespace` will never interact with these
    // functions and they will never be invoked because the namespace is
    // never a root constraint system.

    fn push_namespace<NR, N>(&mut self, _: N)
    where
        NR: AsRef<str>,
        N: FnOnce() -> NR,
    {
        panic!("only the root's push_namespace should be called");
    }

    fn pop_namespace(&mut self) {
        panic!("only the root's pop_namespace should be called");
    }

    fn get_root(&mut self) -> &mut Self::Root {
        self.0.get_root()
    }
}

impl<'a, Scalar: PrimeField, CS: ConstraintSystem<Scalar>> Drop for Namespace<'a, Scalar, CS> {
    fn drop(&mut self) {
        self.get_root().pop_namespace()
    }
}

/// Convenience implementation of ConstraintSystem<Scalar> for mutable references to
/// constraint systems.
impl<'cs, Scalar: PrimeField, CS: ConstraintSystem<Scalar>> ConstraintSystem<Scalar>
    for &'cs mut CS
{
    type Root = CS::Root;

    fn one() -> Variable {
        CS::one()
    }

    fn alloc<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<Scalar, SynthesisError>,
        A: FnOnce() -> AR,
        AR: AsRef<str>,
    {
        (**self).alloc(annotation, f)
    }

    fn alloc_input<F, A, AR>(&mut self, annotation: A, f: F) -> Result<Variable, SynthesisError>
    where
        F: FnOnce() -> Result<Scalar, SynthesisError>,
        A: FnOnce() -> AR,
        AR: AsRef<str>,
    {
        (**self).alloc_input(annotation, f)
    }

    fn enforce<A, AR, LA, LB, LC>(&mut self, annotation: A, a: LA, b: LB, c: LC)
    where
        A: FnOnce() -> AR,
        AR: AsRef<str>,
        LA: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
        LB: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
        LC: FnOnce(LinearCombination<Scalar>) -> LinearCombination<Scalar>,
    {
        (**self).enforce(annotation, a, b, c)
    }

    fn push_namespace<NR, N>(&mut self, name_fn: N)
    where
        NR: AsRef<str>,
        N: FnOnce() -> NR,
    {
        (**self).push_namespace(name_fn)
    }

    fn pop_namespace(&mut self) {
        (**self).pop_namespace()
    }

    fn get_root(&mut self) -> &mut Self::Root {
        (**self).get_root()
    }
}
