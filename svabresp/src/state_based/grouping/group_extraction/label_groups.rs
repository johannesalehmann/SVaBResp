use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use chumsky::prelude::SimpleSpan;
use prism_model::VariableReference;
use probabilistic_models::{
    AtomicProposition, AtomicPropositions, ModelTypes, ProbabilisticModel, TwoPlayer,
    VectorPredecessors,
};
use probabilistic_properties::Query;
use std::collections::HashMap;

struct LabelDetails {
    label_name: String,
    definition_span: SimpleSpan,
    contained_in_players: Vec<String>,
    label_index: Option<usize>,
}

pub struct LabelGroupExtractionScheme {
    labels: Vec<String>,
    label_atomic_propositions: Option<Vec<AtomicProposition>>,
    label_details: Vec<LabelDetails>,
}

impl LabelGroupExtractionScheme {
    pub fn new(labels: Vec<String>) -> Self {
        if labels.len() >= 128 {
            panic!("Currently, at most 127 labels can be used for label-based state grouping");
        }
        Self {
            labels,
            label_atomic_propositions: None,
            label_details: Vec::new(),
        }
    }

    fn no_labels_text() -> &'static str {
        "no labels"
    }
}

impl super::GroupExtractionScheme for LabelGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(
        &mut self,
        prism_model: &mut PrismModel,
        property: &mut PrismProperty,
        atomic_propositions: &mut Vec<prism_model::Expression<VariableReference, SimpleSpan>>,
    ) {
        let _ = property;
        let mut label_atomic_propositions = Vec::new();

        self.label_details.push(LabelDetails {
            label_name: Self::no_labels_text().to_string(),
            definition_span: prism_model.model_type.get_span().clone(),
            contained_in_players: vec![],
            label_index: None,
        });

        for (label_index, label) in self.labels.iter().enumerate() {
            let prism_label = prism_model
                .labels
                .by_name(label.as_str())
                .unwrap_or_else(|| panic!("Could not find label with name `{}`", label));

            let index = atomic_propositions.len();
            atomic_propositions.push(prism_label.condition.clone());
            label_atomic_propositions.push(AtomicProposition::new(index));

            self.label_details.push(LabelDetails {
                label_name: label.to_string(),
                definition_span: prism_label.name.span,
                contained_in_players: Vec::new(),
                label_index: Some(label_index),
            })
        }

        self.label_atomic_propositions = Some(label_atomic_propositions);
    }

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Query<i64, f64, AtomicProposition>,
    ) -> GroupsAndAuxiliary<Self::GroupType> {
        let _ = property;
        let label_atomic_propositions = self.label_atomic_propositions.as_ref().unwrap();
        let mut groups = HashMap::new();
        for (i, state) in game.states.iter().enumerate() {
            let mut index = 0u128;
            for (j, label) in label_atomic_propositions.iter().enumerate() {
                if state.atomic_propositions.get_value(label.index) {
                    index += 1u128 << j;
                }
            }
            if !groups.contains_key(&index) {
                // This string concatenation is not very pretty, but for n<128, this should be plenty fast
                let mut name = String::new();
                for (label, ap_index) in self.labels.iter().zip(label_atomic_propositions.iter()) {
                    if state.atomic_propositions.get_value(ap_index.index) {
                        if name.len() > 0 {
                            name += ", ";
                        }
                        name += label;
                    }
                }
                if name.is_empty() {
                    name = Self::no_labels_text().to_string();
                }

                groups.insert(index, (name, vec![i]));
            } else {
                groups.get_mut(&index).unwrap().1.push(i);
            }
        }

        let mut builder = Self::GroupType::get_builder();
        for (group_mask, (group_name, states)) in groups {
            for label_details in &mut self.label_details {
                if let Some(label_index) = label_details.label_index {
                    if group_mask & (1 << (label_index)) != 0 {
                        label_details.contained_in_players.push(group_name.clone());
                    }
                } else {
                    // If this label_detail has no label_index, then it must be the label_detail
                    // for the unlabelled states.
                    if group_mask == 0 {
                        label_details.contained_in_players.push(group_name.clone());
                    }
                }
            }

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
        use crate::syntax_highlighting::*;
        let mut highlighting = SyntaxHighlighting::new();

        let is_probabilistic = switching_pairs.contains_non_simple_pairs();

        let aggregated_switching_pairs = switching_pairs
            .clone()
            .aggregate_by_minimal_switching_pair();

        for label_details in &self.label_details {
            let mut tooltip = Vec::new();

            let mut total_responsibility = 0.0;
            let mut group_details = Vec::new();
            for group_name in &label_details.contained_in_players {
                let (value, details) = aggregated_switching_pairs.value_and_tool_tip_text(
                    group_name,
                    values,
                    player_names,
                    is_probabilistic,
                );
                total_responsibility += value;
                let overview = format!("{}: {}", group_name, value);
                let details = format!("Details of {}:\n\n{}", group_name, details);
                group_details.push((value, overview, details))
            }

            tooltip.push(format!(
                "Contained in groups with total responsibility {}",
                total_responsibility
            ));

            group_details.sort_unstable_by(|(v1, _, _), (v2, _, _)| {
                v1.partial_cmp(v2)
                    .expect("Encountered NaN while sorting label groups by responsibility value")
            });

            for (_, overview, _) in &group_details {
                tooltip.push(format!("\n\n- {}", overview));
            }
            // tooltip.push("\n\n**Details:**".to_string());
            // for (_, _, details) in &group_details {
            //     tooltip.push(format!("\n\n{}", details));
            // }

            let tooltip = tooltip.join("");

            highlighting.add_highlight(Highlight::new(
                label_details.definition_span.start,
                label_details.definition_span.end,
                Colour::new(2, total_responsibility),
                tooltip,
            ))
        }

        Some(highlighting)
    }
}
