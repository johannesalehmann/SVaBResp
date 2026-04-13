use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use chumsky::prelude::SimpleSpan;
use prism_model::{VariableRange, VariableReference};
use probabilistic_models::{
    AtomicProposition, ModelTypes, ProbabilisticModel, TwoPlayer, Valuation, VectorPredecessors,
};
use probabilistic_properties::Query;
use std::collections::HashMap;
use std::fmt::Formatter;

// TODO: The entire highlighting code does many unnecessary allocations. Check if this impacts
//  performance.
struct VariableHighlightingInfo<V> {
    valuations: Vec<VariableValuation<V>>,
}

impl VariableHighlightingInfo<()> {
    fn insert_group(
        &mut self,
        valuation_name: String,
        group_description: String,
        group_name: String,
    ) {
        let index = self.find_or_create_valuation(valuation_name);
        let valuation = &mut self.valuations[index];
        valuation.entries.push(VariableValuationEntry {
            title: group_description,
            group_name,
            responsibility: (),
        })
    }

    fn find_or_create_valuation(&mut self, valuation_name: String) -> usize {
        for (i, valuation) in self.valuations.iter().enumerate() {
            if valuation.title == valuation_name {
                return i;
            }
        }
        let index = self.valuations.len();
        self.valuations.push(VariableValuation {
            title: valuation_name,
            entries: Vec::new(),
            total_responsibility: (),
        });
        index
    }

    fn add_responsibility(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
    ) -> VariableHighlightingInfo<f64> {
        VariableHighlightingInfo {
            valuations: self
                .valuations
                .iter()
                .map(|v| v.add_responsibility(values))
                .collect(),
        }
    }
}
impl VariableHighlightingInfo<f64> {
    fn compute_influence(&self) -> f64 {
        let average = 1.0 / self.valuations.len() as f64;
        let mut influence = 0.0;

        for valuation in &self.valuations {
            influence += (valuation.total_responsibility - average).abs();
        }

        influence
    }
}

struct VariableValuation<V> {
    title: String,
    entries: Vec<VariableValuationEntry<V>>,
    total_responsibility: V,
}

impl VariableValuation<()> {
    fn add_responsibility(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
    ) -> VariableValuation<f64> {
        let entries: Vec<_> = self
            .entries
            .iter()
            .map(|e| VariableValuationEntry {
                title: e.title.clone(),
                group_name: e.group_name.clone(),
                responsibility: values.get(&e.group_name).map(|v| v.value).unwrap_or(0.0),
            })
            .collect();
        let total_responsibility = entries.iter().map(|e| e.responsibility).sum();
        VariableValuation {
            title: self.title.clone(),
            entries,
            total_responsibility,
        }
    }
}

struct VariableValuationEntry<V> {
    title: String,
    group_name: String,
    responsibility: V,
}

pub struct ValueGroupExtractionScheme {
    variables: Vec<String>,
    variable_types: Option<Vec<VariableType>>,
    variable_references: Option<Vec<VariableReference>>,
    spans: Vec<SimpleSpan>,
    variable_highlighting_infos: Option<Vec<VariableHighlightingInfo<()>>>,
}

impl ValueGroupExtractionScheme {
    pub fn new(variables: Vec<String>) -> Self {
        Self {
            variables,
            variable_types: None,
            variable_references: None,
            spans: Vec::new(),
            variable_highlighting_infos: None,
        }
    }
}

impl super::GroupExtractionScheme for ValueGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(
        &mut self,
        prism_model: &mut PrismModel,
        property: &mut PrismProperty,
        atomic_propositions: &mut Vec<prism_model::Expression<VariableReference, SimpleSpan>>,
        character_to_line: &prism_parser::CharacterToLineMap,
    ) {
        let _ = (property, atomic_propositions, character_to_line);

        let mut variable_references = Vec::with_capacity(self.variables.len());
        let mut variable_types = Vec::with_capacity(self.variables.len());
        let mut variable_highlighting_info = Vec::with_capacity(self.variables.len());
        for variable in &self.variables {
            let reference = prism_model
                .variable_manager
                .get_reference_by_str(variable.as_str())
                .unwrap_or_else(|| panic!("Cannot find variable {} for grouping", variable));
            let variable = prism_model.variable_manager.get(&reference).unwrap();
            self.spans.push(variable.span.clone());
            let variable_type = match variable.range {
                VariableRange::BoundedInt { .. } => VariableType::BoundedInt,
                VariableRange::UnboundedInt { .. } => VariableType::UnboundedInt,
                VariableRange::Boolean { .. } => VariableType::Bool,
                VariableRange::Float { .. } => {
                    panic!("Cannot group by variable value for variables of type `float`")
                }
            };
            variable_references.push(reference);
            variable_types.push(variable_type);
            variable_highlighting_info.push(VariableHighlightingInfo {
                valuations: Vec::new(),
            })
        }
        self.variable_references = Some(variable_references);
        self.variable_types = Some(variable_types);
        self.variable_highlighting_infos = Some(variable_highlighting_info);
    }

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Query<i64, f64, AtomicProposition>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let _ = property;

        let variable_references = self.variable_references.as_ref().unwrap();
        let variable_types = self.variable_types.as_ref().unwrap();
        let variable_highlighting_infos = self.variable_highlighting_infos.as_mut().unwrap();

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

                for (variable_index, variable_name) in self.variables.iter().enumerate() {
                    let variable_valuation_string =
                        format!("`{}={}`", variable_name, values[variable_index]);
                    let mut group_descriptor = Vec::new();
                    for (other_var_index, other_var_name) in self.variables.iter().enumerate() {
                        if other_var_index == variable_index {
                            continue;
                        }
                        group_descriptor
                            .push(format!("`{}={}`", other_var_name, values[other_var_index]));
                    }
                    let group_descriptor = group_descriptor.join(", ");

                    variable_highlighting_infos[variable_index].insert_group(
                        variable_valuation_string,
                        group_descriptor,
                        name.clone(),
                    );
                }
                group_indices.insert(values, groups.len());
                groups.push((name, vec![i]));
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

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        player_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        let _ = (switching_pairs, player_names);
        use crate::syntax_highlighting::*;
        let mut highlighting = SyntaxHighlighting::new();

        let colour_ramp_index = 3;

        let is_probabilistic = switching_pairs.contains_non_simple_pairs();

        let variable_highlighting_infos = self
            .variable_highlighting_infos
            .as_ref()
            .unwrap()
            .iter()
            .map(|v| v.add_responsibility(values))
            .collect::<Vec<_>>();

        let aggregated_switching_pairs = switching_pairs
            .clone()
            .aggregate_by_minimal_switching_pair();

        for ((name, span), highlighting_infos) in self
            .variables
            .iter()
            .zip(self.spans.iter())
            .zip(variable_highlighting_infos.iter())
        {
            let influence = highlighting_infos.compute_influence();

            let mut tooltip = Vec::new();

            tooltip.push(format!(
                "Impact of `{}`'s value on responsibility: <ColoredNumber>{},{}</ColoredNumber>",
                name, influence, colour_ramp_index,
            ));

            let mut tooltip_switching_pairs = Vec::new();

            tooltip.push("\n\n## Responsibility per variable value".to_string());
            for valuation in &highlighting_infos.valuations {
                if self.variables.len() == 1 {
                    tooltip.push(format!(
                        "\n- {}: <ColoredNumber>{},{}</ColoredNumber>",
                        valuation.title, valuation.total_responsibility, colour_ramp_index,
                    ));
                } else {
                    tooltip.push(format!(
                        "\n- {}: <ColoredNumber>{},{}</ColoredNumber> total responsibility",
                        valuation.title, valuation.total_responsibility, colour_ramp_index,
                    ));

                    for group in &valuation.entries {
                        tooltip.push(format!(
                            "\n    - {}: <ColoredNumber>{},{}</ColoredNumber>",
                            group.title, group.responsibility, colour_ramp_index,
                        ))
                    }
                }
                for group in &valuation.entries {
                    if let Some(value) = values.get(&group.group_name)
                        && value.value > 0.0
                    {
                        tooltip_switching_pairs
                            .push(format!("\n\n### Switching pairs of `{}`", group.group_name));
                        let (_, switching_pair_text) = aggregated_switching_pairs
                            .value_and_tool_tip_text(
                                "Variable",
                                colour_ramp_index,
                                &group.group_name,
                                values,
                                player_names,
                                is_probabilistic,
                                true,
                            );
                        tooltip_switching_pairs.push("\n\n".to_string());
                        tooltip_switching_pairs.push(switching_pair_text);
                    }
                }
            }

            tooltip.push("\n\n".to_string());

            let tooltip = tooltip.join("") + &tooltip_switching_pairs.join("");

            highlighting.add_highlight(Highlight::new(
                span.start,
                span.end,
                Colour::new(colour_ramp_index, influence),
                tooltip,
            ))
        }

        Some(highlighting)
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(val) => {
                write!(f, "{}", val)
            }
            Value::Bool(true) => {
                write!(f, "true")
            }
            Value::Bool(false) => {
                write!(f, "false")
            }
        }
    }
}
