use crate::{Command, Expression, Identifier, VariableManager};
use std::fmt::{Display, Formatter};

pub struct ModuleManager<A, E, V, S: Clone> {
    pub modules: Vec<Module<A, E, V, S>>,
}

impl<A, E, V, S: Clone> ModuleManager<A, E, V, S> {
    pub fn new() -> Self {
        Self {
            modules: Vec::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&Module<A, E, V, S>> {
        self.modules.get(index)
    }

    pub fn get_index_by_name(&self, name: &Identifier<S>) -> Option<usize> {
        self.modules
            .iter()
            .enumerate()
            .find(|(_, m)| &m.name == name)
            .map(|(i, _)| i)
    }

    pub fn get_by_name(&self, name: &Identifier<S>) -> Option<&Module<A, E, V, S>> {
        self.modules.iter().find(|m| &m.name == name)
    }

    pub fn add(&mut self, module: Module<A, E, V, S>) -> Result<usize, AddModuleError> {
        for (index, other_module) in self.modules.iter().enumerate() {
            if other_module.name == module.name {
                return Err(AddModuleError::ModuleExists { index });
            }
        }
        let index = self.modules.len();
        self.modules.push(module);
        Ok(index)
    }
}
impl<A, V, S: Clone> ModuleManager<A, Expression<V, S>, V, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> ModuleManager<A, Expression<V, S2>, V, S2> {
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

pub struct Module<A, E, V, S: Clone> {
    pub name: Identifier<S>,
    pub commands: Vec<Command<A, E, V, S>>,
    pub span: S,
}

impl<A, E, V, S: Clone> Module<A, E, V, S> {
    pub fn new(name: Identifier<S>, span: S) -> Self {
        Self {
            name,
            commands: Vec::new(),
            span,
        }
    }
}
impl<A, V, S: Clone> Module<A, Expression<V, S>, V, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Module<A, Expression<V, S2>, V, S2> {
        Module {
            name: self.name.map_span(map),
            commands: self.commands.into_iter().map(|c| c.map_span(map)).collect(),
            span: map(self.span),
        }
    }
}

impl<A: Display, E: Display, V: Display, S: Clone> Module<A, E, V, S> {
    pub fn format<'a, 'b>(
        &'a self,
        variable_manager: &'b VariableManager<E, S>,
        own_index: usize,
    ) -> PrintableModule<'a, 'b, A, E, V, S> {
        PrintableModule {
            module: self,
            variable_manager,
            own_index,
        }
    }
}

pub struct PrintableModule<'a, 'b, A: Display, E: Display, V: Display, S: Clone> {
    module: &'a Module<A, E, V, S>,
    variable_manager: &'b VariableManager<E, S>,
    own_index: usize,
}

impl<'a, 'b, A: Display, E: Display, V: Display, S: Clone> Display
    for PrintableModule<'a, 'b, A, E, V, S>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "module {}", self.module.name)?;
        write!(
            f,
            "{}",
            self.variable_manager.format_as_local_vars(self.own_index)
        )?;
        for command in &self.module.commands {
            writeln!(f, "    {}", command)?;
        }
        writeln!(f, "endmodule")
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
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
