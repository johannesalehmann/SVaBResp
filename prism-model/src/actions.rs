use crate::Identifier;

pub struct ActionManager<S: Clone> {
    actions: Vec<Action<S>>,
}

impl<S: Clone> ActionManager<S> {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
        }
    }

    pub fn add_action(&mut self, name: Identifier<S>) -> Result<ActionReference, AddActionError> {
        if let Some(existing_action) = self.get_reference(&name) {
            Err(AddActionError::ActionExists {
                reference: existing_action,
            })
        } else {
            let index = ActionReference::new(self.actions.len());
            let action = Action::new(name);
            self.actions.push(action);
            Ok(index)
        }
    }

    pub fn get_reference(&self, name: &Identifier<S>) -> Option<ActionReference> {
        for (index, var) in self.actions.iter().enumerate() {
            if &var.name == name {
                return Some(ActionReference::new(index));
            }
        }
        None
    }

    pub fn get(&self, reference: ActionReference) -> Option<&Action<S>> {
        self.actions.get(reference.index)
    }
}

pub enum AddActionError {
    ActionExists { reference: ActionReference },
}

pub struct Action<S: Clone> {
    pub name: Identifier<S>,
}

impl<S: Clone> Action<S> {
    fn new(name: Identifier<S>) -> Self {
        Self { name }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Action<S2> {
        Action::new(self.name.map_span(map))
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct ActionReference {
    index: usize,
}

impl ActionReference {
    fn new(index: usize) -> Self {
        Self { index }
    }
}
