mod into_iter;
pub use into_iter::{IteratedAction, IteratedProbabilisticModel, IteratedState};

mod map_owners;
use map_owners::MappedOwners;

use crate::{
    Action, ActionCollection, AtomicPropositions, Builder, Distribution, DistributionBuilder,
    InitialStates, InitialStatesBuilder, ModelTypes, Owners, Predecessors, PredecessorsBuilder,
    ProbabilisticModel, State, Successor, Valuation,
};

pub trait IterProbabilisticModel<
    V: Valuation,
    O: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    IA: IterAction,
    IS: IterState<V, O, AP, P, IA>,
>
{
    fn next_initial_state(&mut self) -> Option<usize>;
    fn next_state(&mut self) -> Option<IS>;

    fn collect<
        M: ModelTypes<Valuation = V, Owners = O, AtomicPropositions = AP, Predecessors = P>,
    >(
        &mut self,
    ) -> ProbabilisticModel<M> {
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

pub trait IterState<
    V: Valuation,
    O: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    IA: IterAction,
>
{
    fn take_valuation(&mut self) -> V;
    fn next_action(&mut self) -> Option<IA>;
    fn take_atomic_propositions(&mut self) -> AP;
    fn take_owner(&mut self) -> O;
    fn take_predecessors(&mut self) -> P;

    fn collect<
        M: ModelTypes<Valuation = V, Owners = O, AtomicPropositions = AP, Predecessors = P>,
    >(
        &mut self,
    ) -> State<M> {
        let mut builder = <M::ActionCollection as ActionCollection<M::Distribution>>::get_builder();
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

pub trait IterAction {
    fn next_successor(&mut self) -> Option<Successor>;
    fn collect<D: Distribution>(&mut self) -> Action<D> {
        let mut builder = D::get_builder();
        while let Some(successor) = self.next_successor() {
            builder.add_successor(successor);
        }
        Action {
            successors: builder.finish(),
        }
    }
}

pub trait IterFunctions<
    V: Valuation,
    O: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    IA: IterAction,
    IS: IterState<V, O, AP, P, IA>,
> where
    Self: IterProbabilisticModel<V, O, AP, P, IA, IS> + Sized,
{
    fn map_owners<F: Fn(O) -> O2 + Clone, O2: Owners>(
        self,
        map: F,
    ) -> MappedOwners<V, O, O2, AP, P, F, IA, IS, Self>;
}

impl<
    V: Valuation,
    O: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    IA: IterAction,
    IS: IterState<V, O, AP, P, IA>,
    IPM: IterProbabilisticModel<V, O, AP, P, IA, IS> + Sized,
> IterFunctions<V, O, AP, P, IA, IS> for IPM
{
    fn map_owners<F: Fn(O) -> O2 + Clone, O2: Owners>(
        self,
        map: F,
    ) -> MappedOwners<V, O, O2, AP, P, F, IA, IS, IPM> {
        MappedOwners {
            map,
            base: self,
            _phantom_data: Default::default(),
        }
    }
}
