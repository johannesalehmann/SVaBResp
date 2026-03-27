use probabilistic_models::{
    ActionVector, AtomicProposition, DistributionVector, ModelTypes, ProbabilisticModel, TwoPlayer,
    VectorPredecessors,
};

pub trait StochasticGameAlgorithm: Sized {
    type ModelContext;

    fn create_model_context<
        M: ModelTypes<
                Predecessors = VectorPredecessors,
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
                Owners = TwoPlayer,
            >,
    >(
        &self,
        model: &ProbabilisticModel<M>,
    ) -> Self::ModelContext;

    fn create_if_compatible(
        property: &probabilistic_models::probabilistic_properties::Query<
            i64,
            f64,
            AtomicProposition,
        >,
    ) -> Option<Self>;

    fn player_one_probability<
        M: ModelTypes<
                Predecessors = VectorPredecessors,
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
                Owners = TwoPlayer,
            >,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
    ) -> f64 {
        let mut context = self.create_model_context(model);
        self.player_one_probability_with_context(model, &mut context)
    }

    fn player_one_probability_with_context<
        M: ModelTypes<
                Predecessors = VectorPredecessors,
                Distribution = DistributionVector,
                ActionCollection = ActionVector<DistributionVector>,
                Owners = TwoPlayer,
            >,
    >(
        &mut self,
        model: &ProbabilisticModel<M>,
        context: &mut Self::ModelContext,
    ) -> f64;
}

pub trait SolvableStochasticGame {
    type ModelTypes: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        > + Sized;

    fn set_owner(&mut self, state: usize, owner: TwoPlayer);

    fn maximum_player_1_probability(&mut self) -> f64;

    fn get_game(&self) -> &ProbabilisticModel<Self::ModelTypes>;
}

pub struct StochasticGameAndSolver<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        >,
    A: StochasticGameAlgorithm,
> {
    game: ProbabilisticModel<M>,
    solver: A,
    context: A::ModelContext,
}

impl<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        >,
    A: StochasticGameAlgorithm,
> StochasticGameAndSolver<M, A>
{
    pub fn new(game: ProbabilisticModel<M>, solver: A) -> Self {
        let context = solver.create_model_context(&game);
        Self {
            game,
            solver,
            context,
        }
    }

    pub fn with_existing_context(
        game: ProbabilisticModel<M>,
        solver: A,
        context: A::ModelContext,
    ) -> Self {
        Self {
            game,
            solver,
            context,
        }
    }
}

impl<
    M: ModelTypes<
            Predecessors = VectorPredecessors,
            Distribution = DistributionVector,
            ActionCollection = ActionVector<DistributionVector>,
            Owners = TwoPlayer,
        >,
    A: StochasticGameAlgorithm,
> SolvableStochasticGame for StochasticGameAndSolver<M, A>
{
    type ModelTypes = M;

    fn set_owner(&mut self, state: usize, owner: TwoPlayer) {
        self.game.states[state].owner = owner;
    }

    fn maximum_player_1_probability(&mut self) -> f64 {
        self.solver
            .player_one_probability_with_context(&self.game, &mut self.context)
    }

    fn get_game(&self) -> &ProbabilisticModel<Self::ModelTypes> {
        &self.game
    }
}
