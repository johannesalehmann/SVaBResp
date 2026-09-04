use super::{ApIdx, Game, StateIdx};
use probabilistic_model_algorithms::value_iteration;
use probabilistic_models::owners::TwoPlayer;
use probabilistic_properties::{NonDeterminismKind, PathFormula, Query, StateFormula};

pub trait ValueObjective: Sized {
    type Context;

    fn try_from_query(query: &Query<i64, f64, ApIdx>) -> Option<Self>;

    fn create_context(&self, game: &Game) -> Self::Context;

    fn set_owner(
        &self,
        game: &mut Game,
        context: &mut Self::Context,
        state: StateIdx,
        owner: TwoPlayer,
    );

    fn reset(&self, context: &mut Self::Context);

    fn value_from_state(&self, game: &Game, context: &mut Self::Context, state: StateIdx) -> f64;
}

pub struct ReachabilityValue {
    target_states: ApIdx,
    epsilon: f64,
}

impl ReachabilityValue {
    const DEFAULT_EPSILON: f64 = 0.000_001;
}

impl ValueObjective for ReachabilityValue {
    type Context = ();

    fn try_from_query(query: &Query<i64, f64, ApIdx>) -> Option<Self> {
        let Query::ProbabilityValue {
            non_determinism,
            path: PathFormula::Eventually { condition },
        } = query
        else {
            return None;
        };
        if !matches!(non_determinism, None | Some(NonDeterminismKind::Maximise)) {
            return None;
        }
        let StateFormula::Expression(target_states) = **condition else {
            return None;
        };
        Some(Self {
            target_states,
            epsilon: Self::DEFAULT_EPSILON,
        })
    }

    fn create_context(&self, _game: &Game) -> Self::Context {}

    fn set_owner(
        &self,
        game: &mut Game,
        _context: &mut Self::Context,
        state: StateIdx,
        owner: TwoPlayer,
    ) {
        game.base.owners[state] = owner;
    }

    fn reset(&self, _context: &mut Self::Context) {}

    // TODO: Once supported by tiny-pmc, reuse the value vector and pre-compute SCCs once instead of
    //  redoing both every iteration.
    fn value_from_state(&self, game: &Game, _context: &mut (), state: StateIdx) -> f64 {
        value_iteration::value_iteration_game(game, self.target_states, self.epsilon)[state]
    }
}
