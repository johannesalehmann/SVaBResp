use crate::{Command, Identifier, VariableManager};

pub struct Module<A, V, S> {
    pub name: Identifier<S>,
    pub variables: VariableManager<V, S>,
    pub commands: Vec<Command<A, V, S>>,
    pub span: S,
}

impl<A, V, S> Module<A, V, S> {
    pub fn new(name: Identifier<S>, span: S) -> Self {
        Self {
            name,
            variables: VariableManager::new(),
            commands: Vec::new(),
            span,
        }
    }
}

pub struct RenamedModule<S> {
    pub old_name: Identifier<S>,
    pub new_name: Identifier<S>,
    pub rename_rules: Vec<RenameRule<S>>,
    pub span: S,
}

impl<S> RenamedModule<S> {
    pub fn new(
        old_name: Identifier<S>,
        new_name: Identifier<S>,
        rename_rules: Vec<RenameRule<S>>,
        span: S,
    ) -> Self {
        Self {
            old_name,
            new_name,
            rename_rules,
            span,
        }
    }
}

pub struct RenameRule<S> {
    pub old_name: Identifier<S>,
    pub new_name: Identifier<S>,
    pub span: S,
}
impl<S> RenameRule<S> {
    pub fn new(old_name: Identifier<S>, new_name: Identifier<S>, span: S) -> Self {
        Self {
            old_name,
            new_name,
            span,
        }
    }
}
