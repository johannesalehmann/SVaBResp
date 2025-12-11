use super::{IterAction, IterProbabilisticModel, IterState, ModelTypes};
use crate::{
    ActionCollection, AtomicPropositions, Distribution, InitialStates, Owners, Predecessors,
    Successor, Valuation,
};
use std::marker::PhantomData;

pub struct MappedOwners<
    V: Valuation,
    O: Owners,
    O2: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    Map: Fn(O) -> O2 + Clone,
    BaseIterAction: IterAction,
    BaseIterState: IterState<V, O, AP, P, BaseIterAction>,
    Base: IterProbabilisticModel<V, O, AP, P, BaseIterAction, BaseIterState>,
> {
    pub(crate) map: Map,
    pub(crate) base: Base,
    pub(crate) _phantom_data: PhantomData<(V, O, O2, AP, P, BaseIterAction, BaseIterState)>,
}

impl<
    V: Valuation,
    O: Owners,
    O2: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    Map: Fn(O) -> O2 + Clone,
    BaseIterAction: IterAction,
    BaseIterState: IterState<V, O, AP, P, BaseIterAction>,
    Base: IterProbabilisticModel<V, O, AP, P, BaseIterAction, BaseIterState>,
>
    IterProbabilisticModel<
        V,
        O2,
        AP,
        P,
        BaseIterAction,
        MappedOwnersState<V, O, O2, AP, P, Map, BaseIterAction, BaseIterState>,
    > for MappedOwners<V, O, O2, AP, P, Map, BaseIterAction, BaseIterState, Base>
{
    fn next_initial_state(&mut self) -> Option<usize> {
        self.base.next_initial_state()
    }

    fn next_state(
        &mut self,
    ) -> Option<MappedOwnersState<V, O, O2, AP, P, Map, BaseIterAction, BaseIterState>> {
        if let Some(next) = self.base.next_state() {
            Some(MappedOwnersState {
                function: self.map.clone(),
                base: next,
                _phantom_data: Default::default(),
            })
        } else {
            None
        }
    }
}

pub struct MappedOwnersState<
    V: Valuation,
    O: Owners,
    O2: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    Map: Fn(O) -> O2,
    BaseIterAction: IterAction,
    BaseIterState: IterState<V, O, AP, P, BaseIterAction>,
> {
    function: Map,
    base: BaseIterState,
    _phantom_data: PhantomData<(V, O, O2, AP, P, BaseIterAction)>,
}
impl<
    V: Valuation,
    O: Owners,
    O2: Owners,
    AP: AtomicPropositions,
    P: Predecessors,
    Map: Fn(O) -> O2,
    BaseIterAction: IterAction,
    BaseIterState: IterState<V, O, AP, P, BaseIterAction>,
> IterState<V, O2, AP, P, BaseIterAction>
    for MappedOwnersState<V, O, O2, AP, P, Map, BaseIterAction, BaseIterState>
{
    fn take_valuation(&mut self) -> V {
        self.base.take_valuation()
    }

    fn next_action(&mut self) -> Option<BaseIterAction> {
        self.base.next_action()
    }

    fn take_atomic_propositions(&mut self) -> AP {
        self.base.take_atomic_propositions()
    }

    fn take_owner(&mut self) -> O2 {
        (self.function)(self.base.take_owner())
    }

    fn take_predecessors(&mut self) -> P {
        self.base.take_predecessors()
    }
}
