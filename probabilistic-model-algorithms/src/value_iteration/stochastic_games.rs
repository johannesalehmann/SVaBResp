use crate::sccs::{SccList, SccWithDependencies};
use crate::value_iteration::StateData;
use probabilistic_models::probabilistic_properties::{
    NonDeterminismKind, PathFormula, Query, StateFormula,
};
use probabilistic_models::{
    ActionCollection, ActionVector, AtomicProposition, AtomicPropositions, DistributionVector,
    InitialStates, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};

pub struct StochasticGameValueIterationContext {
    data: Vec<StateData>,
    sccs: SccList<SccWithDependencies>,
    scc_reverse_order: Vec<usize>,
    eps: f64,
}

impl StochasticGameValueIterationContext {
    pub fn new<
        M: ModelTypes<
                Predecessors = VectorPredecessors,
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
                Owners = TwoPlayer,
            >,
    >(
        model: &ProbabilisticModel<M>,
        goal_states: AtomicProposition,
        eps: f64,
    ) -> Self {
        let mut data = vec![StateData::new(); model.states.len()];
        let mut excluded_states = Vec::new();
        for (state_index, state) in model.states.iter().enumerate() {
            if state.atomic_propositions.get_value(goal_states.index) {
                excluded_states.push(state_index);
                data[state_index].value = 1.0;
            } else if state.actions.get_number_of_actions() == 0 {
                excluded_states.push(state_index);
            }
        }

        let sccs: SccList = crate::sccs::compute_sccs(
            &model,
            &crate::sccs::ExclusionList::new(&excluded_states[..]),
        );
        let sccs = sccs.compute_dependencies(&model);
        let scc_reverse_order = sccs.get_reverse_topological_order();
        Self {
            data,
            sccs,
            scc_reverse_order,
            eps,
        }
    }

    fn reset(&mut self) {
        for scc in self.sccs.iter() {
            for &state in &scc.members {
                self.data[state].value = 0.0;
            }
        }
    }
}

pub fn value_iteration_stochastic_games<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        >,
>(
    model: &ProbabilisticModel<M>,
    goal_states: AtomicProposition,
    eps: f64,
) -> f64 {
    let mut context = StochasticGameValueIterationContext::new(&model, goal_states, eps);
    value_iteration_stochastic_games_with_context(model, &mut context)
}
pub fn value_iteration_stochastic_games_with_context<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        >,
>(
    model: &ProbabilisticModel<M>,
    context: &mut StochasticGameValueIterationContext,
) -> f64 {
    context.reset();

    super::value_iteration_internal(
        &model,
        &mut context.data,
        context.eps,
        &context.sccs,
        &context.scc_reverse_order,
        super::TwoPlayerMaxMin {},
    );
    context.data[*model.initial_states.iter().next().unwrap()].value
}

pub struct StochasticGameValueIterationAlgorithm {
    goal_states: AtomicProposition,
}

impl crate::traits::StochasticGameAlgorithm for StochasticGameValueIterationAlgorithm {
    type ModelContext = StochasticGameValueIterationContext;

    fn create_model_context<
        M: ModelTypes<
                Predecessors = VectorPredecessors,
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
                Owners = TwoPlayer,
            >,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::ModelContext {
        StochasticGameValueIterationContext::new(model, self.goal_states, 0.000_001)
    }

    fn create_if_compatible(property: &Query<i64, f64, AtomicProposition>) -> Option<Self> {
        if let Query::ProbabilityValue {
            non_determinism,
            path: PathFormula::Eventually { condition },
        } = property
        {
            // TODO: Properly support game formulas. Games always assume that player one maximises
            // and minimises, so we accept both no non-determinism and maximising non-determinism,
            // as both can be considered an accurate description (either of the game objective as a
            // whole or of player one's objective)
            if non_determinism.is_none() || non_determinism.unwrap() == NonDeterminismKind::Maximise
            {
                if let StateFormula::Expression(goal_states) = **condition {
                    return Some(Self { goal_states });
                }
            }
        }
        None
    }

    fn player_one_probability_with_context<
        M: ModelTypes<
                Predecessors = VectorPredecessors,
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
                Owners = TwoPlayer,
            >,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> f64 {
        value_iteration_stochastic_games_with_context(model, context)
    }
}
