use prism_model::{Expression, Identifier, Model, VariableReference};
use std::collections::HashMap;

pub struct SynchronisedActions {
    actions: Vec<SynchronisedAction>,
}

impl<'a> IntoIterator for &'a SynchronisedActions {
    type Item = &'a SynchronisedAction;
    type IntoIter = core::slice::Iter<'a, SynchronisedAction>;

    fn into_iter(self) -> Self::IntoIter {
        self.actions.iter()
    }
}

pub struct SynchronisedAction {
    pub participating_modules: Vec<SynchronisedActionModule>,
    pub name: String,
}

pub struct SynchronisedActionModule {
    pub module_index: usize,
    pub command_indices: Vec<usize>,
}

impl SynchronisedActions {
    pub fn from_prism<S: Clone>(
        model: &Model<(), Identifier<S>, Expression<VariableReference, S>, VariableReference, S>,
    ) -> Self {
        let mut actions: HashMap<String, SynchronisedAction> = HashMap::new();

        for (module_index, module) in model.modules.modules.iter().enumerate() {
            let mut module_actions: HashMap<String, SynchronisedActionModule> = HashMap::new();
            for (command_index, command) in module.commands.iter().enumerate() {
                if let Some(action) = &command.action {
                    if let Some(module_action) = module_actions.get_mut(&action.name) {
                        module_action.command_indices.push(command_index);
                    } else {
                        module_actions.insert(
                            action.name.clone(),
                            SynchronisedActionModule {
                                module_index,
                                command_indices: vec![command_index],
                            },
                        );
                    }
                }
            }
            for (action_name, module_action) in module_actions {
                if let Some(action) = actions.get_mut(&action_name) {
                    action.participating_modules.push(module_action);
                } else {
                    let synchronised_action_into = SynchronisedAction {
                        name: action_name.clone(),
                        participating_modules: vec![module_action],
                    };
                    actions.insert(action_name, synchronised_action_into);
                }
            }
        }

        SynchronisedActions {
            actions: actions.into_iter().map(|(_, a)| a).collect(),
        }
    }
}
