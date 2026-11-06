pub type Coalition = u64;

pub struct CooperativeGame {
    pub players: usize,
    pub values: Vec<f64>,
}

impl CooperativeGame {
    pub fn new(players: usize, values: Vec<f64>) -> Self {
        assert!(players < 64, "At most 63 players are supported");
        assert_eq!(values.len(), 1 << players);
        Self { players, values }
    }

    pub fn value(&self, coalition: Coalition) -> f64 {
        self.values[coalition as usize]
    }
}

pub struct SimpleGame {
    pub players: usize,
    pub minimal_winning: Vec<Coalition>,
}

impl SimpleGame {
    pub fn new(players: usize, minimal_winning: Vec<Coalition>) -> Self {
        assert!(players <= 64, "At most 64 players are supported");
        Self {
            players,
            minimal_winning,
        }
    }

    pub fn is_winning(&self, coalition: Coalition) -> bool {
        self.minimal_winning.iter().any(|&m| m & coalition == m)
    }
}
