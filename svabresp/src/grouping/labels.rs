use super::{Grouping, GroupingScheme, StateGroup};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::traits::ReadStateSpace;
use std::collections::HashMap;
use tiny_pmc::{API, OptionalModel};

/// Groups states by the set of given labels that hold in them: Two states are in the same group
/// iff they satisfy exactly the same labels.
pub struct LabelGrouping {
    labels: Vec<String>,
}

impl LabelGrouping {
    pub fn new(labels: Vec<String>) -> Self {
        Self { labels }
    }
}

impl GroupingScheme for LabelGrouping {
    fn create_groups(
        &mut self,
        model: &mut OptionalModel,
        _query: &ProcessedQuery<API>,
    ) -> Grouping {
        let atomic_propositions = model
            .atomic_propositions
            .as_ref()
            .expect("Grouping by labels requires a model with labels");

        // The model builder creates one atomic proposition per PRISM label, named after the label.
        let label_indices = self
            .labels
            .iter()
            .map(|label| {
                atomic_propositions
                    .index_by_name(label)
                    .unwrap_or_else(|| panic!("Cannot find label `{label}` for grouping"))
            })
            .collect::<Vec<_>>();

        let mut groups: Vec<StateGroup> = Vec::new();
        let mut group_indices: HashMap<Vec<bool>, usize> = HashMap::new();

        let states = probabilistic_models::for_concrete_base!(model.as_ref(), |m| m.states());
        for state in states {
            let values = label_indices
                .iter()
                .map(|&label| atomic_propositions[label][state])
                .collect::<Vec<_>>();

            match group_indices.get(&values) {
                Some(&index) => groups[index].states.push(state),
                None => {
                    let name = self
                        .labels
                        .iter()
                        .zip(&values)
                        .filter(|(_, value)| **value)
                        .map(|(label, _)| label.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    let name = if name.is_empty() {
                        "no labels".to_string()
                    } else {
                        name
                    };
                    group_indices.insert(values, groups.len());
                    groups.push(StateGroup {
                        name,
                        states: vec![state],
                    });
                }
            }
        }

        Grouping::new(groups)
    }
}
