use crate::{
    Action, ActionCollection, Distribution, InitialStates, ModelTypes, ProbabilisticModel, State,
    Successor, Valuation,
};
use std::collections::VecDeque;

pub struct IteratedProbabilisticModel<M: ModelTypes> {
    initial_states: M::InitialStates,
    current_initial_state_index: usize,
    states: VecDeque<State<M>>,
    valuation_context: Option<<M::Valuation as Valuation>::ContextType>,
}

impl<M: ModelTypes> IteratedProbabilisticModel<M> {
    pub(crate) fn new(model: ProbabilisticModel<M>) -> Self {
        Self {
            initial_states: model.initial_states,
            current_initial_state_index: 0,
            states: VecDeque::from(model.states),
            valuation_context: Some(model.valuation_context),
        }
    }
}

impl<'a, M: ModelTypes>
    super::IterProbabilisticModel<
        M::Valuation,
        M::Owners,
        M::AtomicPropositions,
        M::Predecessors,
        IteratedAction<M>,
        IteratedState<M>,
    > for IteratedProbabilisticModel<M>
{
    fn next_initial_state(&mut self) -> Option<usize> {
        if self.current_initial_state_index < self.initial_states.count() {
            let initial_state = self.initial_states.get(self.current_initial_state_index);
            self.current_initial_state_index += 1;
            Some(initial_state)
        } else {
            None
        }
    }

    fn next_state(&mut self) -> Option<IteratedState<M>> {
        if let Some(state) = self.states.pop_front() {
            Some(IteratedState::new(state))
        } else {
            None
        }
    }

    fn take_valuation_context(&mut self) -> <M::Valuation as Valuation>::ContextType {
        self.valuation_context.take().unwrap()
    }
}

pub struct IteratedState<M: ModelTypes> {
    actions: <<M as ModelTypes>::ActionCollection as ActionCollection<M::Distribution>>::IntoIter,
    valuation: Option<M::Valuation>,
    atomic_propositions: Option<M::AtomicPropositions>,
    owner: Option<M::Owners>,
    predecessors: Option<M::Predecessors>,
}
impl<M: ModelTypes> IteratedState<M> {
    fn new(state: State<M>) -> Self {
        let actions = state.actions.into_iter();
        Self {
            actions,
            valuation: Some(state.valuation),
            atomic_propositions: Some(state.atomic_propositions),
            owner: Some(state.owner),
            predecessors: Some(state.predecessors),
        }
    }
}

impl<M: ModelTypes>
    super::IterState<
        M::Valuation,
        M::Owners,
        M::AtomicPropositions,
        M::Predecessors,
        IteratedAction<M>,
    > for IteratedState<M>
{
    fn take_valuation(&mut self) -> M::Valuation {
        self.valuation.take().unwrap()
    }

    fn next_action(&mut self) -> Option<IteratedAction<M>> {
        self.actions.next().map(IteratedAction::new)
    }

    fn take_atomic_propositions(&mut self) -> M::AtomicPropositions {
        self.atomic_propositions.take().unwrap()
    }

    fn take_owner(&mut self) -> M::Owners {
        self.owner.take().unwrap()
    }

    fn take_predecessors(&mut self) -> M::Predecessors {
        self.predecessors.take().unwrap()
    }
}

pub struct IteratedAction<M: ModelTypes> {
    distribution: M::Distribution,
    current_index: usize,
}

impl<M: ModelTypes> IteratedAction<M> {
    fn new(action: Action<M::Distribution>) -> Self {
        Self {
            distribution: action.successors,
            current_index: 0,
        }
    }
}

impl<M: ModelTypes> super::IterAction for IteratedAction<M> {
    fn next_successor(&mut self) -> Option<Successor> {
        if self.current_index < self.distribution.number_of_successors() {
            let successor = self.distribution.get_successor(self.current_index);
            self.current_index += 1;
            Some(successor)
        } else {
            None
        }
    }
}
