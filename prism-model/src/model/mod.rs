mod formula_substitution;

mod renamed_module_expansion;
pub use renamed_module_expansion::ModuleExpansionError;

use crate::formulas::FormulaManager;
use crate::module::RenamedModule;
use crate::rewards::RewardsManager;
use crate::{Expression, LabelManager, ModuleManager, VariableManager};
use std::fmt::{Display, Formatter};

pub struct Model<AM, A, V, S: Clone> {
    pub model_type: ModelType<S>,

    pub variable_manager: VariableManager<V, S>,
    pub formulas: FormulaManager<V, S>,

    pub action_manager: AM,

    pub modules: ModuleManager<A, V, S>,
    pub renamed_modules: Vec<RenamedModule<S>>,

    pub init_constraint: Option<Expression<V, S>>,

    pub labels: LabelManager<V, S>,
    pub rewards: RewardsManager<A, V, S>,

    pub span: S,
}

impl<AM: Default, A, V, S: Clone> Model<AM, A, V, S> {
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
        variable_manager: VariableManager<V, S>,
        formulas: FormulaManager<V, S>,
        action_manager: AM,
        modules: ModuleManager<A, V, S>,
        renamed_modules: Vec<RenamedModule<S>>,
        init_constraint: Option<Expression<V, S>>,
        labels: LabelManager<V, S>,
        rewards: RewardsManager<A, V, S>,
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

    pub fn map_span<S2: Clone, F: Fn(S) -> S2>(self, map: &F) -> Model<AM, A, V, S2> {
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

impl<AM, A: Display, V: Display, S: Clone> Display for Model<AM, A, V, S> {
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
