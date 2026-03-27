use super::algorithm_collections::{AdaptableOwners, ChangeableOwners, NonstochasticGameAlgorithm};
use crate::regions::StateRegion;
use probabilistic_models::{ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors};

pub trait SolvableNonstochasticGame {
    type WinningRegionType: StateRegion;

    type ModelTypes: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer> + Sized;

    fn set_owner(&mut self, state: usize, owner: TwoPlayer);

    fn get_winner(&mut self) -> TwoPlayer;

    fn get_winning_region(&mut self) -> Self::WinningRegionType;

    fn get_game(&self) -> &ProbabilisticModel<Self::ModelTypes>;
}

pub struct NonstochasticGameAndSolver<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: NonstochasticGameAlgorithm<ModelContext: AdaptableOwners>,
> {
    game: probabilistic_models::ProbabilisticModel<M>,
    solver: AC,
    context: AC::ModelContext,
}

impl<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: NonstochasticGameAlgorithm<ModelContext: AdaptableOwners>,
> NonstochasticGameAndSolver<M, AC>
{
    pub fn new(game: ProbabilisticModel<M>, solver: AC) -> Self {
        let context = solver.create_model_context(&game);
        Self {
            game,
            solver,
            context,
        }
    }
}

impl<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: NonstochasticGameAlgorithm<ModelContext: AdaptableOwners>,
> SolvableNonstochasticGame for NonstochasticGameAndSolver<M, AC>
{
    type WinningRegionType = AC::WinningRegionType;
    type ModelTypes = M;

    fn set_owner(&mut self, state: usize, owner: TwoPlayer) {
        self.game.states[state].owner = owner;
    }

    fn get_winner(&mut self) -> TwoPlayer {
        self.context.adapt_to_owners(&self.game);
        self.solver
            .winning_with_context(&self.game, &mut self.context)
    }

    fn get_winning_region(&mut self) -> Self::WinningRegionType {
        self.context.adapt_to_owners(&self.game);
        self.solver
            .winning_region_with_context(&self.game, &mut self.context)
    }

    fn get_game(&self) -> &ProbabilisticModel<Self::ModelTypes> {
        &self.game
    }
}

pub struct NonstochasticGameAndSolverExternalOwners<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: NonstochasticGameAlgorithm<ModelContext: ChangeableOwners>,
> {
    game: probabilistic_models::ProbabilisticModel<M>,
    solver: AC,
    context: AC::ModelContext,
}

impl<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    A: NonstochasticGameAlgorithm<ModelContext: ChangeableOwners>,
> NonstochasticGameAndSolverExternalOwners<M, A>
{
    pub fn new(game: ProbabilisticModel<M>, solver: A) -> Self {
        let context = solver.create_model_context(&game);
        Self {
            game,
            solver,
            context,
        }
    }
}

impl<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: NonstochasticGameAlgorithm<ModelContext: ChangeableOwners>,
> SolvableNonstochasticGame for NonstochasticGameAndSolverExternalOwners<M, AC>
{
    type WinningRegionType = AC::WinningRegionType;
    type ModelTypes = M;

    fn set_owner(&mut self, state: usize, owner: TwoPlayer) {
        self.context.set_owner(state, owner);
    }

    fn get_winner(&mut self) -> TwoPlayer {
        self.solver
            .winning_with_context(&self.game, &mut self.context)
    }

    fn get_winning_region(&mut self) -> Self::WinningRegionType {
        self.solver
            .winning_region_with_context(&self.game, &mut self.context)
    }

    fn get_game(&self) -> &ProbabilisticModel<Self::ModelTypes> {
        &self.game
    }
}
