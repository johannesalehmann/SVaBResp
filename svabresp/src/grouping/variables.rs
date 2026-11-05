use super::{Grouping, GroupingScheme, StateGroup};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::traits::ReadStateSpace;
use probabilistic_models::valuations::{Type, ValuationBits};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use tiny_pmc::{API, OptionalModel};

/// Groups states by the values of the given variables: Two states are in the same group iff they
/// agree on the value of every variable.
pub struct VariableGrouping {
    variables: Vec<String>,
}

impl VariableGrouping {
    pub fn new(variables: Vec<String>) -> Self {
        Self { variables }
    }
}

impl GroupingScheme for VariableGrouping {
    fn create_groups(
        &mut self,
        model: &mut OptionalModel,
        _query: &ProcessedQuery<API>,
    ) -> Grouping {
        let valuations = model
            .state_valuations
            .as_ref()
            .expect("Grouping by variables requires a model with state valuations");

        let variables = self
            .variables
            .iter()
            .map(|name| {
                let (index, variable_type) = super::variable(valuations, name)
                    .unwrap_or_else(|| panic!("Cannot find variable `{name}` for grouping"));
                if !matches!(variable_type, Type::Bool | Type::Int | Type::Uint) {
                    panic!("Cannot group by variable `{name}` of type {variable_type:?}");
                }
                (index, variable_type)
            })
            .collect::<Vec<_>>();

        let mut groups: Vec<StateGroup> = Vec::new();
        let mut group_indices: HashMap<Vec<Value>, usize> = HashMap::new();

        let states = probabilistic_models::for_concrete_base!(model.as_ref(), |m| m.states());
        for state in states {
            let valuation = valuations.entry(state);
            let values = variables
                .iter()
                .map(|&(index, variable_type)| match variable_type {
                    Type::Bool => Value::Bool(valuation.evaluate_bool(index)),
                    _ => Value::Int(valuation.evaluate_int(index)),
                })
                .collect::<Vec<_>>();

            match group_indices.get(&values) {
                Some(&index) => groups[index].states.push(state),
                None => {
                    let name = self
                        .variables
                        .iter()
                        .zip(&values)
                        .map(|(variable, value)| format!("{variable}={value}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    group_indices.insert(values, groups.len());
                    groups.push(StateGroup {
                        name: format!("({name})"),
                        states: vec![state],
                    });
                }
            }
        }

        Grouping::new(groups)
    }
}

#[derive(Hash, Eq, PartialEq)]
enum Value {
    Int(i64),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{value}"),
        }
    }
}
