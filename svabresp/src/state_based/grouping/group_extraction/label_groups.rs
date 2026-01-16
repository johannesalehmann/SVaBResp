use crate::state_based::grouping::GroupsAndAuxiliary;
use crate::{PrismModel, PrismProperty};
use probabilistic_models::{
    AtomicProposition, AtomicPropositions, ModelTypes, ProbabilisticModel, TwoPlayer,
    VectorPredecessors,
};
use probabilistic_properties::Property;
use std::collections::HashMap;

pub struct LabelGroupExtractionScheme {
    labels: Vec<String>,
    label_atomic_propositions: Option<Vec<AtomicProposition>>,
}

impl LabelGroupExtractionScheme {
    pub fn new(labels: Vec<String>) -> Self {
        if labels.len() >= 128 {
            panic!("Currently, at most 127 labels can be used for label-based state grouping");
        }
        Self {
            labels,
            label_atomic_propositions: None,
        }
    }
}

impl super::GroupExtractionScheme for LabelGroupExtractionScheme {
    type GroupType = crate::state_based::grouping::VectorStateGroups;

    fn transform_prism(&mut self, prism_model: &mut PrismModel, property: &mut PrismProperty) {
        let _ = property;
        let mut label_atomic_propositions = Vec::new();

        for label in &self.labels {
            let index = prism_model
                .labels
                .get_index(label.as_str())
                .unwrap_or_else(|| panic!("Could not find label with name `{}`", label));
            label_atomic_propositions.push(AtomicProposition::new(index));
            // This relies on the model builder creating an atomic proposition with matching index for each label. This is currently the case, but should perhaps be handled more generically
        }

        self.label_atomic_propositions = Some(label_atomic_propositions);
    }

    fn create_groups<M: ModelTypes<Owners = TwoPlayer, Predecessors = VectorPredecessors>>(
        &mut self,
        game: &mut ProbabilisticModel<M>,
        property: &Property<AtomicProposition, f64>,
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
                    name = "no labels".to_string();
                }

                groups.insert(index, (name, vec![i]));
            } else {
                groups.get_mut(&index).unwrap().1.push(i);
            }
        }

        let mut builder = Self::GroupType::get_builder();
        for (_, (group_name, states)) in groups {
            builder.create_group_from_vec(states, group_name);
        }

        GroupsAndAuxiliary::new(builder.finish())
    }
}
