use crate::formulas::FormulaManager;
use crate::module::RenamedModule;
use crate::rewards::{Rewards, RewardsManager};
use crate::{Expression, LabelManager, Module, VariableManager};

pub struct Model<AM, A, V, S> {
    pub model_type: ModelType<S>,

    pub global_variables: VariableManager<V, S>,
    pub global_constants: VariableManager<V, S>,
    pub formulas: FormulaManager<V, S>,

    pub action_manager: AM,

    pub modules: Vec<Module<A, V, S>>,
    pub renamed_modules: Vec<RenamedModule<S>>,

    pub init_constraint: Option<Expression<V, S>>,

    pub labels: LabelManager<V, S>,
    pub rewards: RewardsManager<A, V, S>, // todo: turn into rewards manager

    pub span: S,
}

impl<AM, A, V, S> Model<AM, A, V, S>
where
    AM: Default,
{
    pub fn new(model_type: ModelType<S>, span: S) -> Self {
        Self {
            model_type,
            global_variables: VariableManager::new(),
            global_constants: VariableManager::new(),
            formulas: FormulaManager::new(),
            action_manager: AM::default(),
            modules: Vec::new(),
            renamed_modules: Vec::new(),
            init_constraint: None,
            labels: LabelManager::new(),
            rewards: RewardsManager::new(),
            span,
        }
    }

    pub fn from_components(
        model_type: ModelType<S>,
        global_variables: VariableManager<V, S>,
        global_constants: VariableManager<V, S>,
        formulas: FormulaManager<V, S>,
        action_manager: AM,
        modules: Vec<Module<A, V, S>>,
        renamed_modules: Vec<RenamedModule<S>>,
        init_constraint: Option<Expression<V, S>>,
        labels: LabelManager<V, S>,
        rewards: RewardsManager<A, V, S>,
        span: S,
    ) -> Self {
        Self {
            model_type,
            global_variables,
            global_constants,
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
}
