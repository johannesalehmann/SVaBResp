use super::super::attractor;
use super::AlgorithmCollection;
use crate::regions::{FlagStateRegion, InvertedStateRegion, StateRegion};
use probabilistic_models::probabilistic_properties::{
    Path, ProbabilityConstraint, ProbabilityKind, ProbabilityOperator, Property,
};
use probabilistic_models::{
    AtomicProposition, InitialStates, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};

pub struct SafetyAlgorithmCollection {
    good_states: AtomicProposition,
}

impl AlgorithmCollection for SafetyAlgorithmCollection {
    type WinningRegionType = InvertedStateRegion<FlagStateRegion>;
    type ModelContext = SafetyAlgorithmContext;

    fn create_model_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::ModelContext {
        assert_eq!(model.initial_states.count(), 1);
        let initial_state = model.initial_states.get(0);
        let bad_states = model.get_states_without_ap(self.good_states);
        let mut buffer = attractor::AttractorBuffer::create(model);
        buffer.reset_owner_counts(model, TwoPlayer::PlayerTwo);
        SafetyAlgorithmContext {
            bad_states,
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
            path: Path::Generally(ap),
        } = property
        {
            Some(Self { good_states: *ap })
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
            context.bad_states.iter().cloned(),
            state,
            &mut context.buffer,
        ) {
            false => TwoPlayer::PlayerOne,
            true => TwoPlayer::PlayerTwo,
        }
    }
    fn winning_region_with_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> Self::WinningRegionType {
        attractor::attractor_with_buffer::<_, _, FlagStateRegion>(
            model,
            context.bad_states.iter().cloned(),
            &mut context.buffer,
        )
        .inverted()
    }
}

pub struct SafetyAlgorithmContext {
    bad_states: Vec<usize>,
    buffer: attractor::AttractorBuffer,
    initial_state: usize,
}

impl super::ChangeableOwners for SafetyAlgorithmContext {
    fn set_owner(&mut self, index: usize, owner: TwoPlayer) {
        match owner {
            TwoPlayer::PlayerOne => self.buffer.reset_avoiding_player(index),
            TwoPlayer::PlayerTwo => self.buffer.reset_reaching_player(index),
        }
    }
}

impl super::AdaptableOwners for SafetyAlgorithmContext {
    fn adapt_to_owners<M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>>(
        &mut self,
        model: &ProbabilisticModel<M>,
    ) {
        self.buffer.reset_owner_counts(model, TwoPlayer::PlayerTwo);
    }
}
