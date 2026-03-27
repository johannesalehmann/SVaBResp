use super::super::AdaptableOwners;
use super::NonstochasticGameAlgorithm;
use crate::attractor;
use crate::regions::{FlagStateRegion, InvertedStateRegion, MutableStateRegion, StateRegion};
use probabilistic_models::probabilistic_properties::{Bound, BoundOperator, Query, StateFormula};
use probabilistic_models::{
    AtomicProposition, InitialStates, ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors,
};

pub struct BuechiAlgorithmCollection {
    buechi_states: AtomicProposition,
}

impl NonstochasticGameAlgorithm for BuechiAlgorithmCollection {
    type WinningRegionType = InvertedStateRegion<FlagStateRegion>;
    type ModelContext = BuechiAlgorithmContext;

    fn create_model_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::ModelContext {
        assert_eq!(model.initial_states.count(), 1);
        let initial_state = model.initial_states.get(0);
        let buechi_states = model.get_states_with_ap(self.buechi_states);
        let mut buffer = attractor::AttractorBuffer::create(model);
        buffer.reset_owner_counts(model, TwoPlayer::PlayerOne);
        let mut context = BuechiAlgorithmContext {
            buechi_states,
            buffer,
            owners: vec![TwoPlayer::PlayerOne; model.states.len()],
            unreachable: FlagStateRegion::create(model.states.len()),
            initial_state,
        };
        context.adapt_to_owners(&model);
        context
    }

    fn create_if_compatible(property: &Query<i64, f64, AtomicProposition>) -> Option<Self> {
        if let Query::StateFormula(StateFormula::ProbabilityBound {
            non_determinism: Option::None,
            bound:
                Bound {
                    operator: BoundOperator::GreaterOrEqual,
                    value: 1.0,
                },
            path,
        }) = property
            && let Some(StateFormula::ProbabilityBound {
                non_determinism: Option::None,
                bound:
                    Bound {
                        operator: BoundOperator::GreaterOrEqual,
                        value: 1.0,
                    },
                path,
            }) = path.generally_condition()
            && let Some(StateFormula::Expression(ap)) = path.eventually_condition()
        {
            Some(Self { buechi_states: *ap })
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
        let winning_region = self.winning_region_with_context(model, context);
        if winning_region.contains(state) {
            TwoPlayer::PlayerOne
        } else {
            TwoPlayer::PlayerTwo
        }
    }

    fn winning_region_with_context<
        M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> Self::WinningRegionType {
        let mut changed = true;

        context.unreachable.clear();

        while changed {
            context.reset_buffer(TwoPlayer::PlayerOne);

            let reachable = attractor::attractor_with_buffer::<_, _, FlagStateRegion>(
                model,
                context
                    .buechi_states
                    .iter()
                    .filter(|s| !context.unreachable.contains(**s))
                    .cloned(),
                &mut context.buffer,
            );

            context.reset_buffer(TwoPlayer::PlayerTwo);

            let player_2_reachable = attractor::attractor_with_buffer::<_, _, FlagStateRegion>(
                model,
                InvertedStateRegion::new(reachable).iter(),
                &mut context.buffer,
            );

            changed = context.add_new_unreachable_states(&player_2_reachable);
        }

        InvertedStateRegion::new(context.unreachable.clone())
    }
}
pub struct BuechiAlgorithmContext {
    buechi_states: Vec<usize>,
    buffer: attractor::AttractorBuffer,
    owners: Vec<TwoPlayer>,
    unreachable: FlagStateRegion,
    initial_state: usize,
}

impl BuechiAlgorithmContext {
    fn reset_buffer(&mut self, reaching_player: TwoPlayer) {
        for (index, &owner) in self.owners.iter().enumerate() {
            if self.unreachable.contains(index) {
                self.buffer.reset_sink_state(index);
            } else if reaching_player == owner {
                self.buffer.reset_reaching_player(index);
            } else {
                self.buffer.reset_avoiding_player(index);
            }
        }
    }

    fn add_new_unreachable_states(&mut self, new_unreachable: &FlagStateRegion) -> bool {
        let mut changed = false;
        for state in 0..self.unreachable.model_state_count() {
            if !self.unreachable.contains(state) && new_unreachable.contains(state) {
                self.unreachable.add_state(state);
                changed = true;
            }
        }
        changed
    }
}

impl super::ChangeableOwners for BuechiAlgorithmContext {
    fn set_owner(&mut self, index: usize, owner: TwoPlayer) {
        self.owners[index] = owner;
    }
}

impl super::AdaptableOwners for BuechiAlgorithmContext {
    fn adapt_to_owners<M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>>(
        &mut self,
        model: &ProbabilisticModel<M>,
    ) {
        for (index, state) in model.states.iter().enumerate() {
            self.owners[index] = state.owner;
        }
    }
}
