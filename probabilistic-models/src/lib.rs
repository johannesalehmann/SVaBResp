pub use probabilistic_properties;
use std::fmt::Formatter;
use std::marker::PhantomData;

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

mod iter;
pub use iter::*;

mod predecessors;
pub use predecessors::*;

pub trait ModelTypes: Sized {
    type Valuation: Valuation;
    type Distribution: Distribution;
    type Owners: Owners;
    type ActionCollection: ActionCollection<Self::Distribution>;
    type AtomicPropositions: AtomicPropositions;
    type InitialStates: InitialStates;
    type Predecessors: Predecessors;
}

pub struct ProbabilisticModel<M: ModelTypes> {
    pub states: Vec<State<M>>,
    pub initial_states: M::InitialStates,
    pub valuation_context: <M::Valuation as Valuation>::ContextType,
    pub atomic_proposition_count: usize,
    pub action_names: Vec<String>,
}

impl<M: ModelTypes> std::fmt::Debug for ProbabilisticModel<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for state in &self.states {
            write!(
                f,
                "{}",
                state.valuation.displayable(&self.valuation_context)
            )?;
            for i in 0..self.atomic_proposition_count {
                if state.atomic_propositions.get_value(i) {
                    write!(f, "    Fulfils atomic proposition {}", i)?;
                }
            }
            for action in state.actions.iter() {
                write!(
                    f,
                    "    Action `{}`",
                    self.action_names[action.action_name_index]
                )?;
                for target in action.successors.iter() {
                    write!(
                        f,
                        "        {} -> {}",
                        target.probability,
                        self.states[target.index]
                            .valuation
                            .displayable(&self.valuation_context)
                    )?;
                }
            }
        }
        Ok(())
    }
}

impl<M: ModelTypes> ProbabilisticModel<M> {
    pub fn new(
        initial_states: M::InitialStates,
        valuation_context: <M::Valuation as Valuation>::ContextType,
        atomic_proposition_count: usize,
    ) -> Self {
        Self {
            states: Vec::new(),
            initial_states,
            valuation_context,
            atomic_proposition_count,
            action_names: Vec::new(),
        }
    }

    pub fn into_iter(self) -> IteratedProbabilisticModel<M> {
        IteratedProbabilisticModel::new(self)
    }

    pub fn get_states_with_ap(&self, ap: AtomicProposition) -> Vec<usize> {
        let mut res = Vec::new();
        for (index, state) in self.states.iter().enumerate() {
            if state.atomic_propositions.get_value(ap.index) {
                res.push(index);
            }
        }
        res
    }

    pub fn get_states_without_ap(&self, ap: AtomicProposition) -> Vec<usize> {
        let mut res = Vec::new();
        for (index, state) in self.states.iter().enumerate() {
            if !state.atomic_propositions.get_value(ap.index) {
                res.push(index);
            }
        }
        res
    }

    pub fn get_states_with_aps(&self, aps: &[AtomicProposition]) -> Vec<usize> {
        let mut res = Vec::new();
        for (index, state) in self.states.iter().enumerate() {
            for &ap in aps {
                if state.atomic_propositions.get_value(ap.index) {
                    res.push(index);
                }
            }
        }
        res
    }

    pub fn get_action_index_or_add(&mut self, action_name: &str) -> usize {
        for (i, action) in self.action_names.iter().enumerate() {
            if action == action_name {
                return i;
            }
        }

        let res = self.action_names.len();
        self.action_names.push(action_name.to_string());
        res
    }

    pub fn rebuild_predecessors(&mut self) {
        let mut new_predecessors = Vec::with_capacity(self.states.len());
        for _ in 0..self.states.len() {
            new_predecessors.push(<<M::Predecessors as Predecessors>::Builder>::create());
        }

        for (state_index, state) in self.states.iter().enumerate() {
            for (action_index, action) in state.actions.iter().enumerate() {
                for target in action.successors.iter() {
                    new_predecessors[target.index].add(Predecessor {
                        from: state_index,
                        action_index,
                        probability: target.probability,
                    })
                }
            }
        }

        for (index, predecessors) in new_predecessors.into_iter().enumerate() {
            self.states[index].predecessors = predecessors.finish();
        }
    }
}

pub struct State<M: ModelTypes> {
    pub valuation: M::Valuation,
    pub actions: M::ActionCollection,
    pub atomic_propositions: M::AtomicPropositions,
    pub owner: M::Owners,
    pub predecessors: M::Predecessors, // TODO: Rename to "incoming transitions" to highlight that a state may be contained multiple times
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
    action_iterator:
        <<M as ModelTypes>::ActionCollection as ActionCollection<M::Distribution>>::Iter<'a>,
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

pub struct Action<D: Distribution> {
    pub successors: D,
    pub action_name_index: usize,
}

pub type Mdp<P = NonTrackedPredecessors> = ProbabilisticModel<MdpType<P>>;
pub struct MdpType<P: Predecessors = NonTrackedPredecessors> {
    _phantom_data: PhantomData<P>,
}
impl<P: Predecessors> ModelTypes for MdpType<P> {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<DistributionVector>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
    type Predecessors = P;
}

pub type Dtmc<P = NonTrackedPredecessors> = ProbabilisticModel<DtmcType<P>>;
pub struct DtmcType<P: Predecessors = NonTrackedPredecessors> {
    _phantom_data: PhantomData<P>,
}
impl<P: Predecessors> ModelTypes for DtmcType<P> {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = SingleAction<DistributionVector>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
    type Predecessors = P;
}

pub type TransitionSystem<P = NonTrackedPredecessors> = ProbabilisticModel<TransitionSystemType<P>>;
pub struct TransitionSystemType<P: Predecessors = NonTrackedPredecessors> {
    _phantom_data: PhantomData<P>,
}
impl<P: Predecessors> ModelTypes for TransitionSystemType<P> {
    type Valuation = ValuationVector;
    type Distribution = SingleStateDistribution;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<SingleStateDistribution>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
    type Predecessors = P;
}

pub type TwoPlayerStochasticGame<P = NonTrackedPredecessors> =
    ProbabilisticModel<TwoPlayerStochasticGameType<P>>;
pub struct TwoPlayerStochasticGameType<P: Predecessors = NonTrackedPredecessors> {
    _phantom_data: PhantomData<P>,
}
impl<P: Predecessors> ModelTypes for TwoPlayerStochasticGameType<P> {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = TwoPlayer;
    type ActionCollection = ActionVector<DistributionVector>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
    type Predecessors = P;
}

pub type TwoPlayerNonstochasticGame<P = NonTrackedPredecessors> =
    ProbabilisticModel<TwoPlayerNonstochasticGameType<P>>;
pub struct TwoPlayerNonstochasticGameType<P: Predecessors = NonTrackedPredecessors> {
    _phantom_data: PhantomData<P>,
}
impl<P: Predecessors> ModelTypes for TwoPlayerNonstochasticGameType<P> {
    type Valuation = ValuationVector;
    type Distribution = SingleStateDistribution;
    type Owners = TwoPlayer;
    type ActionCollection = ActionVector<SingleStateDistribution>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
    type InitialStates = SingleInitialState;
    type Predecessors = P;
}
