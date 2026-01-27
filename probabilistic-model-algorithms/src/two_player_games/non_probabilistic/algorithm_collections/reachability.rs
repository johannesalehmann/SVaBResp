use super::super::attractor;
use super::AlgorithmCollection;
use crate::regions::FlagStateRegion;
use probabilistic_models::probabilistic_properties::{
    Path, ProbabilityConstraint, ProbabilityKind, ProbabilityOperator, Property,
};
use probabilistic_models::{
    AtomicProposition, InitialStates, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};

pub struct ReachabilityAlgorithmCollection {
    target_states: AtomicProposition,
}

impl AlgorithmCollection for ReachabilityAlgorithmCollection {
    type WinningRegionType = FlagStateRegion;
    type ModelContext = ReachabilityAlgorithmContext;

    fn create_model_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::ModelContext {
        assert_eq!(model.initial_states.count(), 1);
        let initial_state = model.initial_states.get(0);
        let target_states = model.get_states_with_ap(self.target_states);
        let mut buffer = attractor::AttractorBuffer::create(model);
        buffer.reset_owner_counts(model, TwoPlayer::PlayerOne);
        ReachabilityAlgorithmContext {
            target_states,
            buffer,
            initial_state,
        }
    }

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
        } else if let Property {
            operator:
                ProbabilityOperator {
                    kind: ProbabilityKind::P,
                    constraint: ProbabilityConstraint::GreaterOrEqual(1.0),
                },
            path: Path::Eventually(ap),
        } = property
        {
            Some(Self { target_states: *ap })
        } else {
            None
        }
    }

    fn winning_with_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> TwoPlayer {
        self.winning_from_state_with_context(model, context.initial_state, context)
    }

    fn winning_from_state_with_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        state: usize,
        context: &mut Self::ModelContext,
    ) -> TwoPlayer {
        match attractor::attractor_contains_state_with_buffer(
            model,
            context.target_states.iter().cloned(),
            state,
            &mut context.buffer,
        ) {
            true => TwoPlayer::PlayerOne,
            false => TwoPlayer::PlayerTwo,
        }
    }
    fn winning_region_with_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> Self::WinningRegionType {
        attractor::attractor_with_buffer(
            model,
            context.target_states.iter().cloned(),
            &mut context.buffer,
        )
    }
}

pub struct ReachabilityAlgorithmContext {
    target_states: Vec<usize>,
    buffer: attractor::AttractorBuffer,
    initial_state: usize,
}

impl super::ChangeableOwners for ReachabilityAlgorithmContext {
    fn set_owner(&mut self, index: usize, owner: TwoPlayer) {
        match owner {
            TwoPlayer::PlayerOne => self.buffer.reset_reaching_player(index),
            TwoPlayer::PlayerTwo => self.buffer.reset_avoiding_player(index),
        }
    }
}

impl super::AdaptableOwners for ReachabilityAlgorithmContext {
    fn adapt_to_owners<M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>>(
        &mut self,
        model: &ProbabilisticModel<M>,
    ) {
        self.buffer.reset_owner_counts(model, TwoPlayer::PlayerOne);
    }
}
