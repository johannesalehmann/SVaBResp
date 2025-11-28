mod distributions;
pub use distributions::*;

mod owners;
pub use owners::*;

mod actions;
pub use actions::*;
mod state_valuations;
pub use state_valuations::*;

mod atomic_propositions;
pub use atomic_propositions::*;

mod initial_states;
pub use initial_states::*;

pub trait ModelTypes: Sized {
    type Valuation: Valuation + std::hash::Hash + Eq;
    type Distribution: Distribution;
    type Owners: Owners;
    type ActionCollection: ActionCollection<Self>;
    type AtomicPropositions: AtomicPropositions;
    type InitialStates: InitialStates;
}

pub struct ProbabilisticModel<M: ModelTypes> {
    pub states: Vec<State<M>>,
    pub initial_states: M::InitialStates,
}

impl<M: ModelTypes> ProbabilisticModel<M> {
    pub fn new(initial_states: M::InitialStates) -> Self {
        Self {
            states: Vec::new(),
            initial_states,
        }
    }
}

pub struct State<M: ModelTypes> {
    pub valuation: M::Valuation,
    pub actions: M::ActionCollection,
    pub atomic_propositions: M::AtomicPropositions,
}

impl<M: ModelTypes> State<M> {
    pub fn get_all_successors(&self) -> StateSuccessorIterator<'_, '_, M> {
        let mut action_iterator = self.actions.iter();
        let transition_iterator = if let Some(action) = action_iterator.next() {
            Some(action.successors.iter())
        } else {
            None
        };

        StateSuccessorIterator {
            action_iterator,
            transition_iterator,
            action_index: 0,
        }
    }
}

pub struct StateSuccessorIterator<'a: 'b, 'b, M: ModelTypes + 'a + 'b> {
    action_index: usize,
    action_iterator: <<M as ModelTypes>::ActionCollection as ActionCollection<M>>::Iter<'a>,
    transition_iterator: Option<<<M as ModelTypes>::Distribution as Distribution>::Iter<'b>>,
}

impl<'a: 'b, 'b, M: ModelTypes> Iterator for StateSuccessorIterator<'a, 'b, M> {
    type Item = StateSuccessor;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(transition_iterator) = &mut self.transition_iterator {
                if let Some(transition) = transition_iterator.next() {
                    return Some(StateSuccessor {
                        action_index: self.action_index,
                        target_index: transition.index,
                        probability: transition.probability,
                    });
                } else {
                    if let Some(action) = self.action_iterator.next() {
                        self.action_index += 1;
                        self.transition_iterator = Some(action.successors.iter());
                    } else {
                        self.transition_iterator = None;
                    }
                }
            } else {
                return None;
            }
        }
    }
}

pub struct StateSuccessor {
    pub action_index: usize,
    pub target_index: usize,
    pub probability: f64,
}

pub struct Action<M: ModelTypes> {
    pub successors: M::Distribution,
}

pub type Mdp = ProbabilisticModel<MdpType>;
pub struct MdpType {}
impl ModelTypes for MdpType {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<Self>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
}

pub type Dtmc = ProbabilisticModel<DtmcType>;
pub struct DtmcType {}
impl ModelTypes for DtmcType {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = SingleAction<Self>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
}

pub type TransitionSystem = ProbabilisticModel<TransitionSystemType>;
pub struct TransitionSystemType {}
impl ModelTypes for TransitionSystemType {
    type Valuation = ValuationVector;
    type Distribution = SingleStateDistribution;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<Self>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
}
