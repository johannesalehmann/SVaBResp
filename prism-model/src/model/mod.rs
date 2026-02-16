mod formula_substitution;

mod map_expression_cloned;
mod map_expression_into;
mod map_expressions;
mod renamed_module_expansion;
mod variable_indexing;

pub use formula_substitution::FormulaSubstitutionVisitor;
pub use renamed_module_expansion::ModuleExpansionError;

use crate::formulas::FormulaManager;
use crate::module::RenamedModule;
use crate::rewards::RewardsManager;
use crate::{Expression, LabelManager, ModuleManager, VariableManager};
use std::fmt::{Display, Formatter};

pub struct Model<AM, A, E, V, S: Clone> {
    pub model_type: ModelType<S>,

    pub variable_manager: VariableManager<E, S>,
    pub formulas: FormulaManager<E, S>,

    pub action_manager: AM,

    pub modules: ModuleManager<A, E, V, S>,
    pub renamed_modules: Vec<RenamedModule<S>>,

    pub init_constraint: Option<E>,

    pub labels: LabelManager<E, S>,
    pub rewards: RewardsManager<A, E, S>,

    pub span: S,
}

impl<AM: Default, A, E, V, S: Clone> Model<AM, A, E, V, S> {
    pub fn new(model_type: ModelType<S>, span: S) -> Self {
        Self {
            model_type,
            variable_manager: VariableManager::new(),
            formulas: FormulaManager::new(),
            action_manager: AM::default(),
            modules: ModuleManager::new(),
            renamed_modules: Vec::new(),
            init_constraint: None,
            labels: LabelManager::new(),
            rewards: RewardsManager::new(),
            span,
        }
    }

    pub fn from_components(
        model_type: ModelType<S>,
        variable_manager: VariableManager<E, S>,
        formulas: FormulaManager<E, S>,
        action_manager: AM,
        modules: ModuleManager<A, E, V, S>,
        renamed_modules: Vec<RenamedModule<S>>,
        init_constraint: Option<E>,
        labels: LabelManager<E, S>,
        rewards: RewardsManager<A, E, S>,
        span: S,
    ) -> Self {
        Self {
            model_type,
            variable_manager,
            formulas,
            action_manager,
            modules,
            renamed_modules,
            init_constraint,
            labels,
            rewards,
            span,
        }
    }
}
impl<AM, A, V, S: Clone> Model<AM, A, Expression<V, S>, V, S> {
    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(
        self,
        map: &F,
    ) -> Model<AM, A, Expression<V, S2>, V, S2> {
        Model {
            model_type: self.model_type.map_span(map),
            variable_manager: self.variable_manager.map_span(map),
            formulas: self.formulas.map_span(map),
            action_manager: self.action_manager,
            modules: self.modules.map_span(map),
            renamed_modules: self
                .renamed_modules
                .into_iter()
                .map(|m| m.map_span(map))
                .collect(),
            init_constraint: self.init_constraint.map(|i| i.map_span(map)),
            labels: self.labels.map_span(map),
            rewards: self.rewards.map_span(map),
            span: map(self.span),
        }
    }
}

#[derive(Copy, Clone)]
pub enum ModelType<S> {
    Dtmc(S),
    Ctmc(S),
    Mdp(S),
}
impl<S> ModelType<S> {
    pub fn get_span(&self) -> &S {
        match self {
            ModelType::Dtmc(s) => s,
            ModelType::Ctmc(s) => s,
            ModelType::Mdp(s) => s,
        }
    }

    pub fn map_span<S2, F: Fn(S) -> S2>(self, map: &F) -> ModelType<S2> {
        match self {
            ModelType::Dtmc(span) => ModelType::Dtmc(map(span)),
            ModelType::Ctmc(span) => ModelType::Ctmc(map(span)),
            ModelType::Mdp(span) => ModelType::Mdp(map(span)),
        }
    }
}

impl<S> Display for ModelType<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Dtmc(_) => {
                write!(f, "dtmc")
            }
            ModelType::Ctmc(_) => {
                write!(f, "ctmc")
            }
            ModelType::Mdp(_) => {
                write!(f, "mdp")
            }
        }
    }
}

impl<AM, A: Display, E: Display, V: Display, S: Clone> Display for Model<AM, A, E, V, S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.model_type)?;
        writeln!(f, "")?;
        write!(f, "{}", self.variable_manager.format_as_consts())?;
        write!(f, "{}", self.variable_manager.format_as_global_vars())?;
        write!(f, "{}", self.formulas)?;
        write!(f, "{}", self.labels)?;
        if let Some(init) = &self.init_constraint {
            writeln!(f, "init")?;
            writeln!(f, "    {}", init)?;
            writeln!(f, "endinit")?;
        }
        for (i, module) in self.modules.modules.iter().enumerate() {
            writeln!(f, "{}", module.format(&self.variable_manager, i))?;
        }
        for renamed_module in &self.renamed_modules {
            writeln!(f, "{}", renamed_module)?;
        }
        for rewards in &self.rewards.rewards {
            writeln!(f, "{}", rewards)?;
        }

        Ok(())
    }
}
