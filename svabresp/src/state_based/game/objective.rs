use super::{ApIdx, Game, StateIdx, WinningRegion};
use probabilistic_model_algorithms::nonstochastic_games as solvers;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_properties::{Bound, BoundOperator, Query, StateFormula};

pub trait Objective: Sized {
    type Context;

    fn try_from_query(query: &Query<i64, f64, ApIdx>) -> Option<Self>;

    fn create_context(&self, game: &Game) -> Self::Context;

    fn set_owner(&self, context: &mut Self::Context, state: StateIdx, owner: TwoPlayer);

    fn reset(&self, context: &mut Self::Context);

    fn winning_region(&self, game: &Game, context: &mut Self::Context) -> WinningRegion;

    fn winner_from_state(
        &self,
        game: &Game,
        context: &mut Self::Context,
        state: StateIdx,
    ) -> TwoPlayer;
}

pub struct Reachability {
    target_states: ApIdx,
}

pub struct Safety {
    good_states: ApIdx,
}

pub struct Buechi {
    buechi_states: ApIdx,
}

// TODO: These functions could perhaps be migrated to probabilistic-properties (in a more
//  comprehensive way).
fn almost_sure_path(
    query: &Query<i64, f64, ApIdx>,
) -> Option<&probabilistic_properties::PathFormula<i64, f64, ApIdx>> {
    match query {
        Query::StateFormula(formula) => almost_sure_path_of_state_formula(formula),
        _ => None,
    }
}

fn almost_sure_path_of_state_formula(
    formula: &StateFormula<i64, f64, ApIdx>,
) -> Option<&probabilistic_properties::PathFormula<i64, f64, ApIdx>> {
    if let StateFormula::ProbabilityBound {
        non_determinism: None,
        bound:
            Bound {
                operator: BoundOperator::GreaterOrEqual,
                value: 1.0,
            },
        path,
    } = formula
    {
        Some(path)
    } else {
        None
    }
}

impl Objective for Reachability {
    type Context = solvers::ReachabilityContext<StateIdx>;

    fn try_from_query(query: &Query<i64, f64, ApIdx>) -> Option<Self> {
        let path = almost_sure_path(query)?;
        if let Some(StateFormula::Expression(ap)) = path.eventually_condition() {
            Some(Self { target_states: *ap })
        } else {
            None
        }
    }

    fn create_context(&self, game: &Game) -> Self::Context {
        solvers::create_reachability_context(game, self.target_states)
    }

    fn set_owner(&self, context: &mut Self::Context, state: StateIdx, owner: TwoPlayer) {
        context.set_owner(state, owner);
    }

    fn reset(&self, context: &mut Self::Context) {
        context.reset();
    }

    fn winning_region(&self, game: &Game, context: &mut Self::Context) -> WinningRegion {
        WinningRegion::new(solvers::solve_reachability_raw(game, context))
    }

    fn winner_from_state(
        &self,
        game: &Game,
        context: &mut Self::Context,
        state: StateIdx,
    ) -> TwoPlayer {
        solvers::reachability_winner_from_state_raw(game, context, state)
    }
}

impl Objective for Safety {
    type Context = solvers::SafetyContext<StateIdx>;

    fn try_from_query(query: &Query<i64, f64, ApIdx>) -> Option<Self> {
        let path = almost_sure_path(query)?;
        if let Some(StateFormula::Expression(ap)) = path.generally_condition() {
            Some(Self { good_states: *ap })
        } else {
            None
        }
    }

    fn create_context(&self, game: &Game) -> Self::Context {
        solvers::create_safety_context(game, self.good_states)
    }

    fn set_owner(&self, context: &mut Self::Context, state: StateIdx, owner: TwoPlayer) {
        context.set_owner(state, owner);
    }

    fn reset(&self, context: &mut Self::Context) {
        context.reset();
    }

    fn winning_region(&self, game: &Game, context: &mut Self::Context) -> WinningRegion {
        WinningRegion::new(solvers::solve_safety_raw(game, context))
    }

    fn winner_from_state(
        &self,
        game: &Game,
        context: &mut Self::Context,
        state: StateIdx,
    ) -> TwoPlayer {
        solvers::safety_winner_from_state_raw(game, context, state)
    }
}

impl Objective for Buechi {
    type Context = solvers::BuechiContext<StateIdx>;

    fn try_from_query(query: &Query<i64, f64, ApIdx>) -> Option<Self> {
        let path = almost_sure_path(query)?;
        let inner = almost_sure_path_of_state_formula(path.generally_condition()?)?;
        if let Some(StateFormula::Expression(ap)) = inner.eventually_condition() {
            Some(Self { buechi_states: *ap })
        } else {
            None
        }
    }

    fn create_context(&self, game: &Game) -> Self::Context {
        solvers::create_buechi_context(game, self.buechi_states)
    }

    fn set_owner(&self, context: &mut Self::Context, state: StateIdx, owner: TwoPlayer) {
        context.set_owner(state, owner);
    }

    fn reset(&self, context: &mut Self::Context) {
        context.reset();
    }

    fn winning_region(&self, game: &Game, context: &mut Self::Context) -> WinningRegion {
        WinningRegion::new(solvers::solve_buechi_raw(game, context))
    }

    fn winner_from_state(
        &self,
        game: &Game,
        context: &mut Self::Context,
        state: StateIdx,
    ) -> TwoPlayer {
        solvers::buechi_winner_from_state_raw(game, context, state)
    }
}
