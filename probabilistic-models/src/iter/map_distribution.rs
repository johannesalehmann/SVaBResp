// use super::{IterAction, IterProbabilisticModel, IterState, ModelTypes};
// use crate::{Distribution, Successor};
// use std::marker::PhantomData;
//
// pub struct MappedDistributions<
//     D1: Distribution,
//     D2: Distribution,
//     M1: ModelTypes<Distribution = D1>,
//     IA1: IterAction<M1>,
//     IS1: IterState<M1, IA1>,
//     Base: IterProbabilisticModel<M1, IA1, IS1>,
// > {
//     pub(crate) base: Base,
//     pub(crate) _phantom_data: PhantomData<(D1, D2, M1, IA1, IS1)>,
// }
//
// impl<
//     D1: Distribution,
//     D2: Distribution,
//     M1: ModelTypes<Distribution = D1>,
//     M2: ModelTypes<
//             Owners = M1::Owners,
//             Distribution = D2,
//             AtomicPropositions = M1::AtomicPropositions,
//             Valuation = M1::Valuation,
//             ActionCollection = M1::ActionCollection,
//             InitialStates = M1::InitialStates,
//             Predecessors = M1::Predecessors,
//         >,
//     IA1: IterAction<M1>,
//     IS1: IterState<M1, IA1>,
//     Base: IterProbabilisticModel<M1, IA1, IS1>,
// >
//     IterProbabilisticModel<
//         M2,
//         MappedDistributionsAction<D1, D2, M1, IA1>,
//         MappedDistributionsState<D1, D2, M1, IA1, IS1>,
//     > for MappedDistributions<D1, D2, M1, IA1, IS1, Base>
// {
//     fn next_initial_state(&mut self) -> Option<usize> {
//         self.base.next_initial_state()
//     }
//
//     fn next_state(&mut self) -> Option<MappedDistributionsState<D1, D2, M1, IA1, IS1>> {
//         if let Some(next) = self.base.next_state() {
//             Some(MappedDistributionsState {
//                 base: next,
//                 _phantom_data: Default::default(),
//             })
//         } else {
//             None
//         }
//     }
// }
//
// pub struct MappedDistributionsState<
//     D1: Distribution,
//     D2: Distribution,
//     M1: ModelTypes<Distribution = D1>,
//     IA1: IterAction<M1>,
//     BaseIterState: IterState<M1, IA1>,
// > {
//     base: BaseIterState,
//     _phantom_data: PhantomData<(D1, D2, M1, IA1)>,
// }
// impl<
//     D1: Distribution,
//     D2: Distribution,
//     M1: ModelTypes<Distribution = D1>,
//     M2: ModelTypes<
//             Owners = M1::Owners,
//             Distribution = D2,
//             AtomicPropositions = M1::AtomicPropositions,
//             Valuation = M1::Valuation,
//             ActionCollection = M1::ActionCollection,
//             InitialStates = M1::InitialStates,
//             Predecessors = M1::Predecessors,
//         >,
//     IA1: IterAction<M1>,
//     IS1: IterState<M1, IA1>,
// > IterState<M2, MappedDistributionsAction<D1, D2, M1, IA1>>
//     for MappedDistributionsState<D1, D2, M1, IA1, IS1>
// {
//     fn take_valuation(&mut self) -> M2::Valuation {
//         self.base.take_valuation()
//     }
//
//     fn next_action(&mut self) -> Option<MappedDistributionsAction<D1, D2, M1, IA1>> {
//         if let Some(base) = self.base.next_action() {
//             Some(MappedDistributionsAction {
//                 base,
//                 _phantom_data: Default::default(),
//             })
//         } else {
//             None
//         }
//     }
//
//     fn take_atomic_propositions(&mut self) -> M2::AtomicPropositions {
//         self.base.take_atomic_propositions()
//     }
//
//     fn take_owner(&mut self) -> M2::Owners {
//         self.base.take_owner()
//     }
//
//     fn take_predecessors(&mut self) -> M2::Predecessors {
//         self.base.take_predecessors()
//     }
// }
//
// pub struct MappedDistributionsAction<
//     D1: Distribution,
//     D2: Distribution,
//     M1: ModelTypes<Distribution = D1>,
//     Base: IterAction<M1>,
// > {
//     base: Base,
//     _phantom_data: PhantomData<(D1, D2, M1)>,
// }
//
// impl<
//     D1: Distribution,
//     D2: Distribution,
//     M1: ModelTypes<Distribution = D1>,
//     M2: ModelTypes<
//             Owners = M1::Owners,
//             Distribution = D2,
//             AtomicPropositions = M1::AtomicPropositions,
//             Valuation = M1::Valuation,
//             ActionCollection = M1::ActionCollection,
//             InitialStates = M1::InitialStates,
//         >,
//     Base: IterAction<M1>,
// > IterAction<M2> for MappedDistributionsAction<D1, D2, M1, Base>
// {
//     fn next_successor(&mut self) -> Option<Successor> {
//         self.base.next_successor()
//     }
// }
