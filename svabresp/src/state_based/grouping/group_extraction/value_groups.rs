use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use prism_model::{VariableRange, VariableReference};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation, VectorPredecessors,
};
use probabilistic_properties::Property;
use std::collections::HashMap;

pub struct ValueGroupExtractionScheme {
    variables: Vec<String>,
    variable_types: Option<Vec<VariableType>>,
    variable_references: Option<Vec<VariableReference>>,
}

impl ValueGroupExtractionScheme {
    pub fn new(variables: Vec<String>) -> Self {
        Self {
            variables,
            variable_types: None,
            variable_references: None,
        }
    }
}

impl super::GroupExtractionScheme for ValueGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(&mut self, prism_model: &mut PrismModel, property: &mut PrismProperty) {
        let _ = property;

        let mut variable_references = Vec::with_capacity(self.variables.len());
        let mut variable_types = Vec::with_capacity(self.variables.len());
        for variable in &self.variables {
            let reference = prism_model
                .variable_manager
                .get_reference_by_str(variable.as_str())
                .unwrap_or_else(|| panic!("Cannot find variable {} for grouping", variable));
            let variable_type = match prism_model.variable_manager.get(&reference).unwrap().range {
                VariableRange::BoundedInt { .. } => VariableType::BoundedInt,
                VariableRange::UnboundedInt { .. } => VariableType::UnboundedInt,
                VariableRange::Boolean { .. } => VariableType::Bool,
                VariableRange::Float { .. } => {
                    panic!("Cannot group by variable value for variables of type `float`")
                }
            };
            variable_references.push(reference);
            variable_types.push(variable_type);
        }
        self.variable_references = Some(variable_references);
        self.variable_types = Some(variable_types);
    }

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Property<AtomicProposition, f64>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let _ = property;

        let variable_references = self.variable_references.as_ref().unwrap();
        let variable_types = self.variable_types.as_ref().unwrap();

        let mut groups = Vec::new();
        let mut group_indices = HashMap::new();

        for (i, state) in game.states.iter().enumerate() {
            let mut values = Vec::new();
            for (var_ref, var_type) in variable_references.iter().zip(variable_types.iter()) {
                match var_type {
                    VariableType::BoundedInt => values.push(Value::Int(
                        state.valuation.evaluate_bounded_int(var_ref.index),
                    )),
                    VariableType::UnboundedInt => values.push(Value::Int(
                        state.valuation.evaluate_unbounded_int(var_ref.index),
                    )),
                    VariableType::Bool => {
                        values.push(Value::Bool(state.valuation.evaluate_bool(var_ref.index)))
                    }
                }
            }
            if !group_indices.contains_key(&values) {
                let mut name = "(".to_string();
                for (variable, value) in self.variables.iter().zip(values.iter()) {
                    if name.len() > 1 {
                        name += ", ";
                    }
                    name += format!("{}={}", variable, value.to_string()).as_str();
                }
                name += ")";

                group_indices.insert(values, groups.len());
                groups.push((name, vec![i]))
            } else {
                let index = group_indices[&values];
                groups[index].1.push(i);
            }
        }

        let mut builder = Self::GroupType::get_builder();
        for (group_name, states) in groups {
            builder.create_group_from_vec(states, group_name);
        }

        GroupsAndAuxiliary::new(builder.finish())
    }
}

enum VariableType {
    BoundedInt,
    UnboundedInt,
    Bool,
}

#[derive(Hash, Eq, PartialEq)]
enum Value {
    Int(i64),
    Bool(bool),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Int(i) => i.to_string(),
            Value::Bool(b) => b.to_string(),
        }
    }
}
