mod into_iter;
mod map_owners;

pub use into_iter::{IteratedAction, IteratedProbabilisticModel, IteratedState};

use crate::iter::map_owners::MappedOwners;
use crate::{
    Action, ActionCollection, Builder, Distribution, DistributionBuilder, InitialStates,
    InitialStatesBuilder, ModelTypes, Predecessors, PredecessorsBuilder, ProbabilisticModel, State,
    Successor,
};

pub trait IterProbabilisticModel<M: ModelTypes, IA: IterAction<M>, IS: IterState<M, IA>> {
    fn next_initial_state(&mut self) -> Option<usize>;
    fn next_state(&mut self) -> Option<IS>;

    fn collect(&mut self) -> ProbabilisticModel<M> {
        let mut initial_state_builder = <M::InitialStates as InitialStates>::get_builder();
        while let Some(initial_state) = self.next_initial_state() {
            initial_state_builder.add_by_index(initial_state);
        }
        let initial_states = initial_state_builder.finish();

        let mut states = Vec::new();
        while let Some(mut state) = self.next_state() {
            states.push(state.collect())
        }
        ProbabilisticModel {
            states,
            initial_states,
        }
    }
}

pub trait IterState<M: ModelTypes, IA: IterAction<M>> {
    fn take_valuation(&mut self) -> M::Valuation;
    fn next_action(&mut self) -> Option<IA>;
    fn take_atomic_propositions(&mut self) -> M::AtomicPropositions;
    fn take_owner(&mut self) -> M::Owners;
    fn take_predecessors(&mut self) -> M::Predecessors;

    fn collect(&mut self) -> State<M> {
        let mut builder = <M::ActionCollection<M> as ActionCollection<M>>::get_builder();
        while let Some(mut action) = self.next_action() {
            builder.add_action(action.collect());
        }
        let valuation = self.take_valuation();
        let atomic_propositions = self.take_atomic_propositions();
        let owner = self.take_owner();
        let predecessors = self.take_predecessors();
        State {
            valuation,
            actions: builder.finish(),
            atomic_propositions,
            owner,
            predecessors,
        }
    }
}

pub trait IterAction<M: ModelTypes> {
    fn next_successor(&mut self) -> Option<Successor>;
    fn collect(&mut self) -> Action<M> {
        let mut builder = <M::Distribution as Distribution>::get_builder();
        while let Some(successor) = self.next_successor() {
            builder.add_successor(successor);
        }
        Action {
            successors: builder.finish(),
        }
    }
}

pub trait IterFunctions<M: ModelTypes, IA: IterAction<M>, IS: IterState<M, IA>>
where
    Self: IterProbabilisticModel<M, IA, IS> + Sized,
{
    fn map_owners<F: Fn(M::Owners) -> O2, O2: crate::Owners>(
        self,
        map: F,
    ) -> MappedOwners<M::Owners, O2, F, M, IA, IS, Self>;
}

impl<
        M: ModelTypes,
        IA: IterAction<M>,
        IS: IterState<M, IA>,
        IPM: IterProbabilisticModel<M, IA, IS>,
    > IterFunctions<M, IA, IS> for IPM
{
    fn map_owners<F: Fn(M::Owners) -> O2, O2: crate::Owners>(
        self,
        map: F,
    ) -> MappedOwners<M::Owners, O2, F, M, IA, IS, Self> {
        MappedOwners {
            map,
            base: self,
            _phantom_data: Default::default(),
        }
    }
}
