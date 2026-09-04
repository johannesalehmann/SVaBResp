use super::{Game, Objective, StateIdx, WinningRegion};
use probabilistic_models::owners::TwoPlayer;
use probabilistic_models::traits::{ReadInitialStates, StateSet};

pub struct SolvableGame<O: Objective> {
    game: Game,
    objective: O,
    context: O::Context,
    initial_state: StateIdx,
}

impl<O: Objective> SolvableGame<O> {
    pub fn new(game: Game, objective: O) -> Self {
        let initial_states = game.initial_states().iter().collect::<Vec<_>>();
        assert_eq!(
            initial_states.len(),
            1,
            "responsibility computation requires a model with exactly one initial state"
        );
        let initial_state = initial_states[0];
        let context = objective.create_context(&game);
        Self {
            game,
            objective,
            context,
            initial_state,
        }
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn initial_state(&self) -> StateIdx {
        self.initial_state
    }

    pub fn set_owner(&mut self, state: StateIdx, owner: TwoPlayer) {
        self.objective.set_owner(&mut self.context, state, owner);
    }

    pub fn winner(&mut self) -> TwoPlayer {
        // TODO: Ideally, we avoid a complete reset of the context here. For safety and
        //  reachability, this iterates over all states and checks their owners. We have already
        //  manually reset the owner counts via `set_owner`, so doing this again is unnecessary.
        //  However, we cannot entirely avoid this call, as e.g. Büchi depends on it. Perhaps the
        //  resetting can be more granular (this needs to be done in tiny-pmc).
        self.objective.reset(&mut self.context);
        self.objective
            .winner_from_state(&self.game, &mut self.context, self.initial_state)
    }

    pub fn winning_region(&mut self) -> WinningRegion {
        // TODO: See winner()
        self.objective.reset(&mut self.context);
        self.objective.winning_region(&self.game, &mut self.context)
    }
}
