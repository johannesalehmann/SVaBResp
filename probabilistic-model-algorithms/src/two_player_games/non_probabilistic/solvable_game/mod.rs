use crate::two_player_games::non_probabilistic::algorithm_collections::{
    AdaptableOwners, AlgorithmCollection, ChangeableOwners,
};
use probabilistic_models::{ModelTypes, ProbabilisticModel, TwoPlayer, VectorPredecessors};

pub trait SolvableGame {
    fn set_owner(&mut self, state: usize, owner: TwoPlayer);

    fn get_winner(&mut self) -> TwoPlayer;
}

pub struct GameAndSolver<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: AlgorithmCollection<ModelContext: AdaptableOwners>,
> {
    game: probabilistic_models::ProbabilisticModel<M>,
    solver: AC,
    context: AC::ModelContext,
}

impl<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: AlgorithmCollection<ModelContext: AdaptableOwners>,
> GameAndSolver<M, AC>
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
    AC: AlgorithmCollection<ModelContext: AdaptableOwners>,
> SolvableGame for GameAndSolver<M, AC>
{
    fn set_owner(&mut self, state: usize, owner: TwoPlayer) {
        self.game.states[state].owner = owner;
    }

    fn get_winner(&mut self) -> TwoPlayer {
        self.context.adapt_to_owners(&self.game);
        self.solver
            .winning_with_context(&self.game, &mut self.context)
    }
}

pub struct GameAndSolverExternalOwners<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: AlgorithmCollection<ModelContext: ChangeableOwners>,
> {
    game: probabilistic_models::ProbabilisticModel<M>,
    solver: AC,
    context: AC::ModelContext,
}

impl<
    M: ModelTypes<Predecessors = VectorPredecessors, Owners = TwoPlayer>,
    AC: AlgorithmCollection<ModelContext: ChangeableOwners>,
> GameAndSolverExternalOwners<M, AC>
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
    AC: AlgorithmCollection<ModelContext: ChangeableOwners>,
> SolvableGame for GameAndSolverExternalOwners<M, AC>
{
    fn set_owner(&mut self, state: usize, owner: TwoPlayer) {
        self.context.set_owner(state, owner);
    }

    fn get_winner(&mut self) -> TwoPlayer {
        self.solver
            .winning_with_context(&self.game, &mut self.context)
    }
}
