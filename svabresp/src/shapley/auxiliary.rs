use log::trace;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::identities::{One, Zero};

pub fn compute_weights(n: usize) -> Vec<BigRational> {
    trace!("Computing weights");
    let mut factorials = Vec::with_capacity(n + 1);
    let mut current_value = BigInt::one();
    factorials.push(BigInt::one());
    for i in 1..=n {
        current_value *= i;
        factorials.push(current_value.clone());
    }

    let mut weights = Vec::with_capacity(n + 1);
    weights.push(BigRational::zero());
    for i in 1..=n {
        weights.push(
            //match weight_type {
            // WeightType::Shapley =>
            BigRational::new(
                factorials[n - i].clone() * factorials[i - 1].clone(),
                factorials[n].clone(),
            ),
            // WeightType::Banzhaf => BigRational::new(1.into(), BigInt::from(2).pow(n as u32 - 1)),
            // WeightType::Count => BigRational::one(),
            //}
        )
    }
    weights
}
