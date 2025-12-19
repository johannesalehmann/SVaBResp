mod reachability;
pub use reachability::{ReachabilityAlgorithmCollection, ReachabilityAlgorithmContext};

mod safety;
pub use safety::{SafetyAlgorithmCollection, SafetyAlgorithmContext};

use crate::regions::StateRegion;
use probabilistic_models::{AtomicProposition, ProbabilisticModel, TwoPlayer, VectorPredecessors};

pub trait AlgorithmCollection: Sized {
    type WinningRegionType: StateRegion;
    type ModelContext;

    fn create_model_context<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &self,
        model: &probabilistic_models::ProbabilisticModel<M>,
    ) -> Self::ModelContext;

    fn create_if_compatible(
        property: &probabilistic_models::probabilistic_properties::Property<AtomicProposition, f64>,
    ) -> Option<Self>;

    fn winning<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
    ) -> TwoPlayer {
        let mut context = self.create_model_context(model);
        self.winning_with_context(model, &mut context)
    }

    fn winning_with_context<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> TwoPlayer;

    fn winning_from_state<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        state: usize,
    ) -> TwoPlayer {
        let mut context = self.create_model_context(model);
        self.winning_from_state_with_context(model, state, &mut context)
    }
    fn winning_from_state_with_context<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        state: usize,
        context: &mut Self::ModelContext,
    ) -> TwoPlayer;

    fn winning_region<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
    ) -> Self::WinningRegionType {
        let mut context = self.create_model_context(model);
        self.winning_region_with_context(model, &mut context)
    }

    fn winning_region_with_context<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> Self::WinningRegionType;
}

pub trait ChangeableOwners {
    fn set_owner(&mut self, index: usize, owner: TwoPlayer);
}

pub trait AdaptableOwners {
    fn adapt_to_owners<
        M: probabilistic_models::ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    >(
        &mut self,
        model: &probabilistic_models::ProbabilisticModel<M>,
    );
}
