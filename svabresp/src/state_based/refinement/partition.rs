use crate::state_based::grouping::{StateGroups, VectorStateGroups};
use log::trace;
use std::collections::HashMap;

pub struct PlayerPartition {
    pub entries: Vec<PlayerPartitionEntry>,
}

impl PlayerPartition {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn with_entries(entries: Vec<PlayerPartitionEntry>) -> Self {
        Self { entries }
    }

    pub fn add_entry(&mut self, entry: PlayerPartitionEntry) {
        self.entries.push(entry);
    }

    pub fn split_entry<F: Fn(usize) -> usize>(&mut self, entry_index: usize, predicate: F) {
        let mut new_groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for &state in &self.entries[entry_index].players {
            let new_group = predicate(state);
            if let Some(group) = new_groups.get_mut(&new_group) {
                group.push(state);
            } else {
                new_groups.insert(new_group, vec![state]);
            }
        }

        let mut replaced_initial = false;
        // By preferentially assigning group 0 to the original entry, we ensure that the order of
        // entries behaves as deterministically as possible, i.e. does not primarily depend on the
        // ordering within the HashMap.
        if let Some(remaining_group) = new_groups.remove(&0) {
            self.entries[entry_index].players = remaining_group;
            replaced_initial = true;
        }
        for (_, group) in new_groups {
            if replaced_initial {
                self.entries.push(PlayerPartitionEntry::with_players(group));
            } else {
                self.entries[entry_index] = PlayerPartitionEntry::with_players(group);
                replaced_initial = true;
            }
        }
    }

    pub fn merge_non_singletons(&mut self) {
        let mut first_non_singleton_index = None;
        let mut entries = Vec::new();
        std::mem::swap(&mut entries, &mut self.entries);
        for mut entry in entries {
            if entry.players.len() == 1 {
                self.entries.push(entry);
            } else {
                if let Some(first_singleton_index) = first_non_singleton_index {
                    let first_non_singleton: &mut PlayerPartitionEntry =
                        &mut self.entries[first_singleton_index];
                    first_non_singleton.players.append(&mut entry.players);
                } else {
                    first_non_singleton_index = Some(self.entries.len());
                    self.entries.push(entry);
                }
            }
        }
    }

    pub fn apply_to_grouping<G: StateGroups>(&self, groups: G) -> VectorStateGroups {
        trace!("Applying group blocking to grouping");
        let mut builder = VectorStateGroups::get_builder();

        for entry in &self.entries {
            for &player in &entry.players {
                for state in groups.get_states(player) {
                    builder.add_state(state);
                }
            }

            let label = if entry.players.len() == 1 {
                groups.get_label(entry.players[0])
            } else {
                "[unnamed group of states]".to_string()
            };
            builder.finish_group(label)
        }

        for dummy_state in groups.get_dummy_states() {
            builder.add_dummy_state(dummy_state);
        }

        builder.finish()
    }
}

pub struct PlayerPartitionEntry {
    pub players: Vec<usize>,
}

impl PlayerPartitionEntry {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }

    pub fn with_players(players: Vec<usize>) -> Self {
        Self { players }
    }
}
