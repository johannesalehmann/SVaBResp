// Stores both whether a state is in the coalition (lowermost bit) and whether there is a
// scheduler defined for this state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Owner(u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Behaviour {
    Maximise,
    Minimise,
    FollowScheduler,
}

impl Owner {
    const IN_COALITION: u8 = 0b01;
    const SCHEDULED: u8 = 0b10;

    pub const MINIMISER: Self = Self(0);
    pub const MAXIMISER: Self = Self(Self::IN_COALITION);
    pub const SCHEDULER: Self = Self(Self::SCHEDULED);

    pub fn is_in_coalition(self) -> bool {
        self.0 & Self::IN_COALITION != 0
    }

    pub fn join_coalition(&mut self) {
        debug_assert!(!self.is_in_coalition(), "state is already in the coalition");
        self.0 |= Self::IN_COALITION;
    }

    pub fn leave_coalition(&mut self) {
        debug_assert!(self.is_in_coalition(), "state is not in the coalition");
        self.0 &= !Self::IN_COALITION;
    }

    pub fn behaviour(self) -> Behaviour {
        if self.is_in_coalition() {
            Behaviour::Maximise
        } else if self.0 & Self::SCHEDULED != 0 {
            Behaviour::FollowScheduler
        } else {
            Behaviour::Minimise
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving_the_coalition_restores_the_behaviour_outside_it() {
        for (initial, outside) in [
            (Owner::MINIMISER, Behaviour::Minimise),
            (Owner::SCHEDULER, Behaviour::FollowScheduler),
        ] {
            let mut owner = initial;
            assert_eq!(owner.behaviour(), outside);
            owner.join_coalition();
            assert_eq!(owner.behaviour(), Behaviour::Maximise);
            owner.leave_coalition();
            assert_eq!(owner, initial);
        }
    }

    #[test]
    fn maximiser_starts_in_the_coalition() {
        assert!(Owner::MAXIMISER.is_in_coalition());
        assert_eq!(Owner::MAXIMISER.behaviour(), Behaviour::Maximise);
    }
}
