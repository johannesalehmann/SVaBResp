use crate::input::constants::ConstParsingError;
use chumsky::prelude::SimpleSpan;
use clap::Parser;
use prism_model::{Expression, VariableReference};
use prism_model_builder::ModelBuildingError;
use probabilistic_models::AtomicProposition;
use probabilistic_properties::{Operator, Path};
use tiny_pmc::PrismProperty;

mod input;

mod arg_parsing;
#[cfg(test)]
mod tests;

fn main() {
    let exit_code = match checker() {
        Ok(()) => 0,
        Err(err) => err.print_and_get_error_code(),
    };
    std::process::exit(exit_code);
}

fn checker() -> Result<(), ModelCheckerError> {
    let start_time = std::time::Instant::now();

    let arguments = arg_parsing::Arguments::parse();
    let source = read_model_file(&arguments.model)?;
    let constants = input::constants::parse_const_assignments(&arguments.constants)?;

    let parsed_model_and_objectives = input::prism::parse_prism_and_print_errors(
        Some(&arguments.model),
        &source,
        &[&arguments.property],
    );
    let (prism_model, properties) = match parsed_model_and_objectives {
        None => return Err(ModelCheckerError::ModelAndPropertyParsingError),
        Some((prism_model, properties)) => (prism_model, properties),
    };

    let mut atomic_propositions = Vec::new();
    let properties = prism_objectives_to_atomic_propositions(&mut atomic_propositions, properties);

    let model =
        prism_model_builder::build_model(&prism_model, &atomic_propositions[..], constants)?;
    for (i, property) in properties.iter().enumerate() {
        println!("Checking property {} of {}", i + 1, properties.len());
        match (&property.operator, &property.path) {
            (Operator::ValueOfPMax, Path::Eventually(AtomicProposition { index })) => {
                probabilistic_model_algorithms::mdp::optimistic_value_iteration(
                    &model, *index, 0.000_001,
                );
            }
            _ => panic!("This combination of operator and path formula is not supported"),
        }
    }

    println!("Finished in {:?}", start_time.elapsed());
    Ok(())
}

fn prism_objectives_to_atomic_propositions(
    atomic_proposition: &mut Vec<Expression<VariableReference, SimpleSpan>>,
    properties: Vec<PrismProperty>,
) -> Vec<probabilistic_properties::Property<probabilistic_models::AtomicProposition>> {
    let mut new_properties = Vec::new();
    for property in properties {
        match property.path {
            Path::Eventually(e) => {
                new_properties.push(probabilistic_properties::Property {
                    operator: property.operator,
                    path: Path::Eventually(probabilistic_models::AtomicProposition::new(
                        atomic_proposition.len(),
                    )),
                });
                atomic_proposition.push(e);
            }
        }
    }
    new_properties
}

fn read_model_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}

enum ModelCheckerError {
    InputFileError(std::io::Error),
    ConstParsingError(ConstParsingError),
    ModelAndPropertyParsingError,
    ModelBuildingError(ModelBuildingError),
}

impl ModelCheckerError {
    pub fn print_and_get_error_code(self) -> i32 {
        match self {
            ModelCheckerError::InputFileError(err) => {
                println!("Could not read input file: {err}");
                1
            }
            ModelCheckerError::ConstParsingError(err) => {
                println!("{err}");
                2
            }
            ModelCheckerError::ModelAndPropertyParsingError => 3, // This error is already printed when it is produced
            ModelCheckerError::ModelBuildingError(err) => {
                println!("Error during model building: {:?}", err);
                4
            }
        }
    }
}

impl From<std::io::Error> for ModelCheckerError {
    fn from(value: std::io::Error) -> Self {
        ModelCheckerError::InputFileError(value)
    }
}

impl From<ConstParsingError> for ModelCheckerError {
    fn from(value: ConstParsingError) -> Self {
        ModelCheckerError::ConstParsingError(value)
    }
}

impl From<ModelBuildingError> for ModelCheckerError {
    fn from(value: ModelBuildingError) -> Self {
        ModelCheckerError::ModelBuildingError(value)
    }
}
