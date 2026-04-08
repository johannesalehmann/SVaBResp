use crate::shapley::PlayerDescriptions;
use crate::state_based::grouping::StateGroups;

#[derive(Clone)]
pub struct GroupNames {
    names: Vec<String>,
}

impl GroupNames {
    pub fn from_grouping<G: StateGroups>(groups: &G) -> Self {
        let mut names = Vec::with_capacity(groups.get_count());
        for g in 0..groups.get_count() {
            names.push(groups.get_label(g))
        }
        Self { names }
    }
}

impl PlayerDescriptions for GroupNames {
    type IntoIter = std::vec::IntoIter<String>;
    type PlayerType = String;

    fn get_player_description(&self, index: usize) -> &Self::PlayerType {
        &self.names[index]
    }

    fn into_iterator(self) -> Self::IntoIter {
        IntoIterator::into_iter(self.names)
    }
}
