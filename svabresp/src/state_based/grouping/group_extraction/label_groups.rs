use crate::shapley::{ResponsibilityValues, SwitchingPairCollection};
use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::state_based::game::{ApIdx, Game};
use crate::{PrismModel, PrismProperty};
use prism_model::{FullSpan, Span};
use probabilistic_models::traits::{ReadAtomicPropositions, ReadStateSpace};
use probabilistic_properties::Query;
use std::collections::HashMap;

struct LabelDetails {
    label_name: String,
    definition_span: FullSpan,
    contained_in_players: Vec<String>,
    label_index: Option<usize>,
}

pub struct LabelGroupExtractionScheme {
    labels: Vec<String>,
    label_atomic_propositions: Option<Vec<ApIdx>>,
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
        character_to_line: &prism_parser::CharacterToLineMap,
    ) {
        let _ = (property, character_to_line);

        self.label_details.push(LabelDetails {
            label_name: Self::no_labels_text().to_string(),
            definition_span: prism_model.model_type.span().clone(),
            contained_in_players: vec![],
            label_index: None,
        });

        for (label_index, label) in self.labels.iter().enumerate() {
            let prism_label = prism_model
                .labels
                .by_name(label.as_str())
                .unwrap_or_else(|| panic!("Could not find label with name `{}`", label));

            self.label_details.push(LabelDetails {
                label_name: label.to_string(),
                definition_span: prism_label.name.span.clone(),
                contained_in_players: Vec::new(),
                label_index: Some(label_index),
            })
        }
    }

    fn create_groups(
        &mut self,
        game: Game,
        property: &Query<i64, f64, ApIdx>,
    ) -> (Game, GroupsAndAuxiliary<Self::GroupType>) {
        let _ = property;

        // The model builder creates one atomic proposition per PRISM label, named after the label,
        // so the atomic propositions for the grouping labels can be looked up on the built model.
        let label_atomic_propositions: Vec<ApIdx> = self
            .labels
            .iter()
            .map(|label| {
                game.atomic_propositions
                    .index_by_name(label.as_str())
                    .unwrap_or_else(|| {
                        panic!("The built model has no atomic proposition for label `{}`", label)
                    })
            })
            .collect();
        self.label_atomic_propositions = Some(label_atomic_propositions.clone());

        let mut groups = HashMap::new();
        for state in game.states() {
            let mut index = 0u128;
            for (j, &label) in label_atomic_propositions.iter().enumerate() {
                if game.is_atomic_proposition_set(state, label) {
                    index += 1u128 << j;
                }
            }
            if !groups.contains_key(&index) {
                // This string concatenation is not very pretty, but for n<128, this should be plenty fast
                let mut name = String::new();
                for (label, &ap_index) in self.labels.iter().zip(label_atomic_propositions.iter()) {
                    if game.is_atomic_proposition_set(state, ap_index) {
                        if name.len() > 0 {
                            name += ", ";
                        }
                        name += label;
                    }
                }
                if name.is_empty() {
                    name = Self::no_labels_text().to_string();
                }

                groups.insert(index, (name, vec![state]));
            } else {
                groups.get_mut(&index).unwrap().1.push(state);
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

        (game, GroupsAndAuxiliary::new(builder.finish()))
    }

    fn get_syntax_elements<S: AsRef<str>>(
        &self,
        values: &ResponsibilityValues<String, f64, f64>,
        switching_pairs: &SwitchingPairCollection,
        player_names: &[S],
    ) -> Option<crate::syntax_highlighting::SyntaxHighlighting> {
        use crate::syntax_highlighting::*;
        let mut highlighting = SyntaxHighlighting::new();
        let colour_ramp_index = 2;

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
                    "Label",
                    colour_ramp_index,
                    group_name,
                    values,
                    player_names,
                    is_probabilistic,
                    true,
                );
                total_responsibility += value;
                let overview = format!(
                    "`{}`: <ColoredNumber>{}, {}</ColoredNumber>",
                    group_name, value, colour_ramp_index
                );
                let details = format!("### Switching pairs for `{}`:\n\n{}", group_name, details);
                group_details.push((value, overview, details))
            }

            tooltip.push(format!(
                "Label responsibility for `{}`: <ColoredNumber>{}, {}</ColoredNumber>",
                label_details.label_name, total_responsibility, colour_ramp_index,
            ));

            group_details.sort_unstable_by(|(v1, _, _), (v2, _, _)| {
                v1.partial_cmp(v2)
                    .expect("Encountered NaN while sorting label groups by responsibility value")
            });

            tooltip.push("\n\n## Responsibility of label groups:".to_string());
            for (_, overview, _) in &group_details {
                tooltip.push(format!("\n- {}", overview));
            }
            tooltip.push("\n\n## Details:".to_string());
            for (_, _, details) in &group_details {
                tooltip.push(format!("\n\n{}", details));
            }

            let tooltip = tooltip.join("");

            if let Some(range) = label_details.definition_span.range() {
                highlighting.add_highlight(Highlight::new(
                    range.start,
                    range.end,
                    Colour::new(2, total_responsibility),
                    tooltip,
                ))
            }
        }

        Some(highlighting)
    }
}
