use crate::PrismProperty;
use chumsky::prelude::SimpleSpan;
use prism_model::{Expression, VariableReference};
use probabilistic_properties::{Path, ProbabilitySpecifier};

pub fn prism_objectives_to_atomic_propositions<P: ProbabilitySpecifier>(
    atomic_proposition: &mut Vec<Expression<VariableReference, SimpleSpan>>,
    properties: Vec<
        probabilistic_properties::Property<Expression<VariableReference, SimpleSpan>, P>,
    >,
) -> Vec<probabilistic_properties::Property<probabilistic_models::AtomicProposition, P>> {
    let mut new_properties = Vec::new();
    for property in properties {
        let (path, e) = match property.path {
            Path::Eventually(e) => (
                Path::Eventually(probabilistic_models::AtomicProposition::new(
                    atomic_proposition.len(),
                )),
                e,
            ),
            Path::Generally(e) => (
                Path::Generally(probabilistic_models::AtomicProposition::new(
                    atomic_proposition.len(),
                )),
                e,
            ),
            Path::InfinitelyOften(e) => (
                Path::InfinitelyOften(probabilistic_models::AtomicProposition::new(
                    atomic_proposition.len(),
                )),
                e,
            ),
        };

        new_properties.push(probabilistic_properties::Property {
            operator: property.operator,
            path,
        });
        atomic_proposition.push(e);
    }
    new_properties
}
