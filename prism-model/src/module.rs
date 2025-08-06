use crate::{Command, Identifier, VariableManager};
use std::fmt::{Display, Formatter};

pub struct ModuleManager<A, V, S: Clone> {
    pub modules: Vec<Module<A, V, S>>,
}

impl<A, V, S: Clone> ModuleManager<A, V, S> {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Module<A, V, S>> {
        self.modules.get(index)
    }

    pub fn get_by_name(&self, name: &Identifier<S>) -> Option<&Module<A, V, S>> {
        self.modules.iter().find(|m| &m.name == name)
    }

    pub fn add(&mut self, module: Module<A, V, S>) -> Result<(), AddModuleError> {
        for (index, other_module) in self.modules.iter().enumerate() {
            if other_module.name == module.name {
                return Err(AddModuleError::ModuleExists { index });
            }
        }
        self.modules.push(module);
        Ok(())
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> ModuleManager<A, V, S2> {
        let mut module_manager = ModuleManager {
            modules: Vec::with_capacity(self.modules.len()),
        };
        for module in self.modules {
            module_manager.modules.push(module.map_span(map))
        }

        module_manager
    }
}

#[derive(Debug)]
pub enum AddModuleError {
    ModuleExists { index: usize },
}

pub struct Module<A, V, S: Clone> {
    pub name: Identifier<S>,
    pub variables: VariableManager<V, S>,
    pub commands: Vec<Command<A, V, S>>,
    pub span: S,
}

impl<A, V, S: Clone> Module<A, V, S> {
    pub fn new(name: Identifier<S>, span: S) -> Self {
        Self {
            name,
            variables: VariableManager::new(),
            commands: Vec::new(),
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Module<A, V, S2> {
        Module {
            name: self.name.map_span(map),
            variables: self.variables.map_span(map),
            commands: self.commands.into_iter().map(|c| c.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<A: Display, V: Display, S: Clone> Display for Module<A, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module {}", self.name)?;
        write!(f, "{}", self.variables.format_as_local_vars())?;
        for command in &self.commands {
            writeln!(f, "    {}", command)?;
        }
        writeln!(f, "endmodule")
    }
}

pub struct RenamedModule<S: Clone> {
    pub old_name: Identifier<S>,
    pub new_name: Identifier<S>,
    pub rename_rules: RenameRules<S>,
    pub span: S,
}

impl<S: Clone> RenamedModule<S> {
    pub fn new(
        old_name: Identifier<S>,
        new_name: Identifier<S>,
        rename_rules: RenameRules<S>,
        span: S,
    ) -> Self {
        Self {
            old_name,
            new_name,
            rename_rules,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> RenamedModule<S2> {
        RenamedModule {
            old_name: self.old_name.map_span(map),
            new_name: self.new_name.map_span(map),
            rename_rules: RenameRules {
                rules: self
                    .rename_rules
                    .rules
                    .into_iter()
                    .map(|i| i.map_span(map))
                    .collect(),
            },
            span: map(self.span),
        }
    }
}

impl<S: Clone> Display for RenamedModule<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "module {} = {} [", self.new_name, self.old_name)?;
        let mut is_first = true;
        for rename_rule in &self.rename_rules.rules {
            if !is_first {
                write!(f, ", ")?;
            }
            is_first = false;
            write!(f, "{}", rename_rule)?;
        }
        write!(f, "] endmodule")
    }
}

pub struct RenameRules<S: Clone> {
    pub rules: Vec<RenameRule<S>>,
}

impl<S: Clone> RenameRules<S> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn get_renaming(&self, old_name: &Identifier<S>) -> Option<Identifier<S>> {
        for rule in &self.rules {
            if &rule.old_name == old_name {
                return Some(rule.new_name.clone());
            }
        }
        None
    }
}

pub struct RenameRule<S: Clone> {
    pub old_name: Identifier<S>,
    pub new_name: Identifier<S>,
    pub span: S,
}
impl<S: Clone> RenameRule<S> {
    pub fn new(old_name: Identifier<S>, new_name: Identifier<S>, span: S) -> Self {
        Self {
            old_name,
            new_name,
            span,
        }
    }

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> RenameRule<S2> {
        RenameRule {
            old_name: self.old_name.map_span(map),
            new_name: self.new_name.map_span(map),
            span: map(self.span),
        }
    }
}
impl<S: Clone> Display for RenameRule<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.old_name, self.new_name)
    }
}
