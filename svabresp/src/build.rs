use crate::grouping::GroupingScheme;
use prism_model_builder::ModelBuilder;
use prism_model_builder::queries::ProcessedQuery;
use tiny_pmc::parsing::Inputs;
use tiny_pmc::{API, OptionalModel};

pub struct BuiltModel {
    pub model: OptionalModel,
    pub query: ProcessedQuery<API>,
}

pub fn build(files: &[String], constants: &str, grouping: &mut dyn GroupingScheme) -> BuiltModel {
    let constants = tiny_pmc::parsing::parse_const_assignments(constants)
        .unwrap_or_else(|e| panic!("Failed to parse constants: {e:?}"));
    let inputs = Inputs::from_cli_args(files).unwrap_or_else(|e| panic!("{e}"));

    match inputs {
        Inputs::Prism { mut model, queries } => {
            let mut query = single(queries.into_iter().collect());
            grouping.transform_prism(&mut model, &mut query);
            let built = ModelBuilder::new(&mut model)
                .with_all_labels()
                .with_query(query.into())
                .with_constants(constants)
                .with_initial_state_vector()
                .build()
                .unwrap_or_else(|e| panic!("Failed to build model: {e:?}"));
            BuiltModel {
                model: built.model.into_optional(),
                query: built.query,
            }
        }
        Inputs::Explicit { model, queries } => {
            if grouping.requires_prism() {
                panic!("The selected grouping scheme requires a PRISM model");
            }
            BuiltModel {
                model: model.into_optional(),
                query: single(queries.into()),
            }
        }
    }
}

fn single<Q>(queries: Vec<Q>) -> Q {
    let count = queries.len();
    let mut queries = queries.into_iter();
    match (queries.next(), queries.next()) {
        (Some(query), None) => query,
        _ => panic!("Expected exactly one property, got {count}"),
    }
}
