use crate::coop_game::{Coalition, CooperativeGame, SimpleGame};

fn shapley_values(players: usize, value: impl Fn(Coalition) -> f64) -> Vec<f64> {
    assert!(players < 64, "At most 63 players are supported");
    let weights = shapley_weights(players);
    let mut values = vec![0.0; players];

    for coalition in 0..(1 as Coalition) << players {
        let coalition_value = value(coalition);
        let size = coalition.count_ones() as usize;
        for player in 0..players {
            let extended = coalition | 1 << player;
            if extended != coalition {
                values[player] += weights[size] * (value(extended) - coalition_value);
            }
        }
    }

    values
}

/// Computes weights for the Shapley value.
///
/// Here, `weights[s]` is the weight given to a coalition of size `s` (not counting the player that
/// is added to the coalition as part of the Shapley check).
fn shapley_weights(players: usize) -> Vec<f64> {
    let factorials: Vec<f64> = (0..=players)
        .scan(1.0, |factorial, i| {
            if i > 0 {
                *factorial *= i as f64;
            }
            Some(*factorial)
        })
        .collect();

    (0..players)
        .map(|size| factorials[size] * factorials[players - size - 1] / factorials[players])
        .collect()
}

impl CooperativeGame {
    pub fn shapley_values(&self) -> Vec<f64> {
        shapley_values(self.players, |coalition| self.value(coalition))
    }
}

impl SimpleGame {
    pub fn shapley_values(&self) -> Vec<f64> {
        shapley_values(self.players, |coalition| {
            if self.is_winning(coalition) { 1.0 } else { 0.0 }
        })
    }
}
