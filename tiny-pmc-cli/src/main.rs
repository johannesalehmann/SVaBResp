use prism_model_builder::ModelBuildingError;
use std::collections::HashSet;

mod input;

#[cfg(test)]
mod tests;

fn main() {
    let exit_code = checker();
    std::process::exit(exit_code);
}

fn checker() -> i32 {
    let source = include_str!("tests/files/consensus.2.v1-fixed.prism");
    // let source = include_str!("tests/files/tiny_test.prism");
    let parse = input::prism::parse_prism(Some("tiny_test.prism"), source);
    let prism = match parse {
        None => {
            return 1;
        }
        Some(parse) => parse,
    };

    let objective = "pc1=3 & pc2=3&!(coin1=coin2)";
    let objective = input::prism::parse_expression(objective, &prism.variable_manager);
    let objective = match objective {
        None => {
            return 2;
        }
        Some(objective) => objective,
    };

    let model = prism_model_builder::build_model(&prism, &[objective]);
    let model = match model {
        Ok(model) => model,
        Err(err) => {
            panic!("Error during model building. TODO: Add nicer formatting here")
        }
    };

    let values = probabilistic_model_algorithms::mdp::value_iteration(&model, 0);

    0
}
