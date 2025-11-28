use crate::PrismProperty;
use chumsky::prelude::SimpleSpan;
use prism_model::{Expression, VariableReference};
use probabilistic_properties::Path;

pub fn prism_objectives_to_atomic_propositions(
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
