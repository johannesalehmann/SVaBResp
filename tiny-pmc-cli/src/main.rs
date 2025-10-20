use chumsky::prelude::SimpleSpan;
use prism_model::{Path, VariableReference};

mod input;

#[cfg(test)]
mod tests;

fn main() {
    let exit_code = checker();
    std::process::exit(exit_code);
}

fn checker() -> i32 {
    let start_time = std::time::Instant::now();
    // let source = include_str!("tests/files/consensus.2.v1-fixed.prism");
    // let source = include_str!("tests/files/consensus.4.prism");
    let source = include_str!("tests/files/consensus.6.v1.prism");
    let objective = "PMax = ? [F \"finished\"&!\"agree\"]";
    let constants = "K=2;N=6";
    let constants = input::constants::parse_const_assignments(constants);
    let parsed_model_and_objectives =
        input::prism::parse_prism(Some("tiny_test.prism"), source, &[objective]);
    let (prism_model, properties) = match parsed_model_and_objectives {
        None => {
            return 1;
        }
        Some((prism_model, properties)) => (prism_model, properties),
    };

    let atomic_propositions = properties
        .into_iter()
        .map(|o| match o.path {
            Path::Eventually(e) => e,
        })
        .collect::<Vec<_>>();

    let model = prism_model_builder::build_model(&prism_model, &atomic_propositions[..], constants);
    let model = match model {
        Ok(model) => model,
        Err(err) => {
            panic!("Error during model building: {:?}", err)
        }
    };

    probabilistic_model_algorithms::mdp::optimistic_value_iteration(&model, 0, 0.000_001);

    println!("Finished in {:?}", start_time.elapsed());
    0
}
