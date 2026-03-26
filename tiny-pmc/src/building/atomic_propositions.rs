use chumsky::prelude::SimpleSpan;
use prism_model::{Expression, VariableReference};

pub fn prism_objectives_to_atomic_propositions<I, F>(
    atomic_proposition: &mut Vec<Expression<VariableReference, SimpleSpan>>,
    queries: Vec<probabilistic_properties::Query<I, F, Expression<VariableReference, SimpleSpan>>>,
) -> Vec<probabilistic_properties::Query<I, F, probabilistic_models::AtomicProposition>> {
    let mut new_properties = Vec::new();
    for query in queries {
        new_properties.push(query.map_e(&mut |e| {
            let res = probabilistic_models::AtomicProposition::new(atomic_proposition.len());
            atomic_proposition.push(e);
            res
        }));
    }
    new_properties
}
