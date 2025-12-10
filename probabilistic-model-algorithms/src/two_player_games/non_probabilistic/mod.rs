use crate::regions::{
    InvertedStateRegion, OrderedVectorStateRegion, StateRegion, VectorStateRegion,
};
use probabilistic_models::probabilistic_properties::{
    Path, ProbabilityConstraint, ProbabilityKind, ProbabilityOperator, Property,
};
use probabilistic_models::{
    AtomicProposition, ModelTypes, Predecessors, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};

mod attractor;

pub trait AlgorithmCollection: Sized {
    type WinningRegionType: StateRegion;

    fn create_if_compatible(
        property: &probabilistic_models::probabilistic_properties::Property<AtomicProposition, f64>,
    ) -> Option<Self>;

    fn compute_winning_player<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> TwoPlayer;

    fn is_winning<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
        state: usize,
    ) -> bool;

    fn compute_winning_region<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::WinningRegionType;
}

pub struct SafetyAlgorithmCollection {
    bad_states: AtomicProposition,
}

impl AlgorithmCollection for SafetyAlgorithmCollection {
    type WinningRegionType = InvertedStateRegion<OrderedVectorStateRegion>;

    fn create_if_compatible(property: &Property<AtomicProposition, f64>) -> Option<Self> {
        if let Property {
            operator:
                ProbabilityOperator {
                    kind: ProbabilityKind::P,
                    constraint: ProbabilityConstraint::EqualTo(1.0),
                },
            path: Path::Never(ap),
        } = property
        {
            Some(Self { bad_states: *ap })
        } else {
            None
        }
    }

    fn compute_winning_player<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> TwoPlayer {
        todo!()
    }

    fn is_winning<M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>>(
        &self,
        model: &ProbabilisticModel<M>,
        state: usize,
    ) -> bool {
        let region = self.compute_winning_region(model);
        region.is_set(state)
    }

    fn compute_winning_region<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::WinningRegionType {
        let target = model.get_states_with_ap(self.bad_states);

        let attractor: VectorStateRegion = attractor::attractor(model, target.iter().cloned()); // TODO: Use more efficient region data structure

        attractor.sorted().inverted()
    }
}

pub struct ReachabilityAlgorithmCollection {
    target_states: AtomicProposition,
}

impl AlgorithmCollection for ReachabilityAlgorithmCollection {
    type WinningRegionType = VectorStateRegion;

    fn create_if_compatible(property: &Property<AtomicProposition, f64>) -> Option<Self> {
        if let Property {
            operator:
                ProbabilityOperator {
                    kind: ProbabilityKind::P,
                    constraint: ProbabilityConstraint::EqualTo(1.0),
                },
            path: Path::Eventually(ap),
        } = property
        {
            Some(Self { target_states: *ap })
        } else {
            None
        }
    }

    fn compute_winning_player<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> TwoPlayer {
        todo!()
    }

    fn is_winning<M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>>(
        &self,
        model: &ProbabilisticModel<M>,
        state: usize,
    ) -> bool {
        let region = self.compute_winning_region(model);
        region.is_set(state)
    }
    fn compute_winning_region<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::WinningRegionType {
        let target = model.get_states_with_ap(self.target_states);

        attractor::attractor(model, target.iter().cloned())
    }
}
