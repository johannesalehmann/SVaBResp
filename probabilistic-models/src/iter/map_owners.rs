use super::{IterAction, IterProbabilisticModel, IterState, ModelTypes};
use crate::{Owners, Predecessors, Successor};
use std::marker::PhantomData;

pub struct MappedOwners<
    O1: Owners,
    O2: Owners,
    Map: Fn(O1) -> O2,
    M1: ModelTypes<Owners = O1>,
    IA1: IterAction<M1>,
    IS1: IterState<M1, IA1>,
    Base: IterProbabilisticModel<M1, IA1, IS1>,
> {
    pub(crate) map: Map,
    pub(crate) base: Base,
    pub(crate) _phantom_data: PhantomData<(O1, O2, M1, IA1, IS1)>,
}

impl<
        O1: Owners,
        O2: Owners,
        Map: Fn(O1) -> O2 + Clone,
        M1: ModelTypes<Owners = O1>,
        M2: ModelTypes<
            Owners = O2,
            Distribution = M1::Distribution,
            AtomicPropositions = M1::AtomicPropositions,
            Valuation = M1::Valuation,
            ActionCollection<M2> = M1::ActionCollection<M2>,
            InitialStates = M1::InitialStates,
            Predecessors = M1::Predecessors,
        >,
        IA1: IterAction<M1>,
        IS1: IterState<M1, IA1>,
        Base: IterProbabilisticModel<M1, IA1, IS1>,
    >
    IterProbabilisticModel<
        M2,
        MappedOwnersAction<O1, O2, Map, M1, IA1>,
        MappedOwnersState<O1, O2, Map, M1, IA1, IS1>,
    > for MappedOwners<O1, O2, Map, M1, IA1, IS1, Base>
{
    fn next_initial_state(&mut self) -> Option<usize> {
        self.base.next_initial_state()
    }

    fn next_state(&mut self) -> Option<MappedOwnersState<O1, O2, Map, M1, IA1, IS1>> {
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
    O1: Owners,
    O2: Owners,
    Map: Fn(O1) -> O2 + Clone,
    M1: ModelTypes<Owners = O1>,
    IA1: IterAction<M1>,
    BaseIterState: IterState<M1, IA1>,
> {
    function: Map,
    base: BaseIterState,
    _phantom_data: PhantomData<(O1, O2, M1, IA1)>,
}
impl<
        O1: Owners,
        O2: Owners,
        Map: Fn(O1) -> O2 + Clone,
        M1: ModelTypes<Owners = O1>,
        M2: ModelTypes<
            Owners = O2,
            Distribution = M1::Distribution,
            AtomicPropositions = M1::AtomicPropositions,
            Valuation = M1::Valuation,
            ActionCollection<M2> = M1::ActionCollection<M2>,
            InitialStates = M1::InitialStates,
            Predecessors = M1::Predecessors,
        >,
        IA1: IterAction<M1>,
        IS1: IterState<M1, IA1>,
    > IterState<M2, MappedOwnersAction<O1, O2, Map, M1, IA1>>
    for MappedOwnersState<O1, O2, Map, M1, IA1, IS1>
{
    fn take_valuation(&mut self) -> M2::Valuation {
        self.base.take_valuation()
    }

    fn next_action(&mut self) -> Option<MappedOwnersAction<O1, O2, Map, M1, IA1>> {
        if let Some(base) = self.base.next_action() {
            Some(MappedOwnersAction {
                map: self.function.clone(),
                base,
                _phantom_data: Default::default(),
            })
        } else {
            None
        }
    }

    fn take_atomic_propositions(&mut self) -> M2::AtomicPropositions {
        self.base.take_atomic_propositions()
    }

    fn take_owner(&mut self) -> M2::Owners {
        (self.function)(self.base.take_owner())
    }

    fn take_predecessors(&mut self) -> M2::Predecessors {
        self.base.take_predecessors()
    }
}

pub struct MappedOwnersAction<
    O1: Owners,
    O2: Owners,
    Map: Fn(O1) -> O2 + Clone,
    M1: ModelTypes<Owners = O1>,
    Base: IterAction<M1>,
> {
    map: Map,
    base: Base,
    _phantom_data: PhantomData<(O1, O2, M1)>,
}

impl<
        O1: Owners,
        O2: Owners,
        Map: Fn(O1) -> O2 + Clone,
        M1: ModelTypes<Owners = O1>,
        M2: ModelTypes<
            Owners = O2,
            Distribution = M1::Distribution,
            AtomicPropositions = M1::AtomicPropositions,
            Valuation = M1::Valuation,
            ActionCollection<M2> = M1::ActionCollection<M2>,
            InitialStates = M1::InitialStates,
        >,
        Base: IterAction<M1>,
    > IterAction<M2> for MappedOwnersAction<O1, O2, Map, M1, Base>
{
    fn next_successor(&mut self) -> Option<Successor> {
        self.base.next_successor()
    }
}
