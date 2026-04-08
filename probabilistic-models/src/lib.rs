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

mod features;
pub use features::{ModelFeatures, Ownership};

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
            writeln!(
                f,
                "{}",
                state.valuation.displayable(&self.valuation_context)
            )?;
            for i in 0..self.atomic_proposition_count {
                if state.atomic_propositions.get_value(i) {
                    writeln!(f, "    Fulfils atomic proposition {}", i)?;
                }
            }
            for action in state.actions.iter() {
                writeln!(
                    f,
                    "    Action `{}`",
                    self.action_names[action.action_name_index]
                )?;
                for target in action.successors.iter() {
                    writeln!(
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

    fn compute_predecessors<P: Predecessors>(&self) -> Vec<P> {
        let mut new_predecessors = Vec::with_capacity(self.states.len());
        for _ in 0..self.states.len() {
            new_predecessors.push(P::Builder::create());
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

        new_predecessors.into_iter().map(|p| p.finish()).collect()
    }

    pub fn rebuild_predecessors(&mut self) {
        let new_predecessors = self.compute_predecessors();

        for (index, predecessors) in new_predecessors.into_iter().enumerate() {
            self.states[index].predecessors = predecessors;
        }
    }
    pub fn rebuild_and_transform_predecessors<
        P2: Predecessors,
        M2: ModelTypes<
                Valuation = M::Valuation,
                Distribution = M::Distribution,
                Owners = M::Owners,
                ActionCollection = M::ActionCollection,
                AtomicPropositions = M::AtomicPropositions,
                InitialStates = M::InitialStates,
                Predecessors = P2,
            >,
    >(
        self,
    ) -> ProbabilisticModel<M2> {
        let new_predecessors: Vec<P2> = self.compute_predecessors();

        ProbabilisticModel {
            states: self
                .states
                .into_iter()
                .zip(new_predecessors.into_iter())
                .map(|(s, p)| s.map_predecessors(|_| p))
                .collect(),
            initial_states: self.initial_states,
            valuation_context: self.valuation_context,
            atomic_proposition_count: self.atomic_proposition_count,
            action_names: self.action_names,
        }
    }

    pub fn get_model_features(&self) -> ModelFeatures {
        ModelFeatures::from_model(self)
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

    pub fn map_predecessors<
        P2: Predecessors,
        F: FnOnce(M::Predecessors) -> P2,
        M2: ModelTypes<
                Valuation = M::Valuation,
                Distribution = M::Distribution,
                Owners = M::Owners,
                ActionCollection = M::ActionCollection,
                AtomicPropositions = M::AtomicPropositions,
                InitialStates = M::InitialStates,
                Predecessors = P2,
            >,
    >(
        self,
        map: F,
    ) -> State<M2> {
        State {
            valuation: self.valuation,
            actions: self.actions,
            atomic_propositions: self.atomic_propositions,
            owner: self.owner,
            predecessors: map(self.predecessors),
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

pub type Mdp<
    P = NonTrackedPredecessors,
    V = ValuationVector,
    AP = BitFlagsAtomicPropositions,
    I = SingleInitialState,
> = ProbabilisticModel<MdpType<P, V, AP, I>>;
pub struct MdpType<
    P: Predecessors = NonTrackedPredecessors,
    V: Valuation = ValuationVector,
    AP: AtomicPropositions = BitFlagsAtomicPropositions,
    I: InitialStates = SingleInitialState,
> {
    _phantom_data: PhantomData<(P, V, AP, I)>,
}
impl<P: Predecessors, V: Valuation, AP: AtomicPropositions, I: InitialStates> ModelTypes
    for MdpType<P, V, AP, I>
{
    type Valuation = V;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<DistributionVector>;
    type AtomicPropositions = AP;
    type InitialStates = I;
    type Predecessors = P;
}

pub type Dtmc<
    P = NonTrackedPredecessors,
    V = ValuationVector,
    AP = BitFlagsAtomicPropositions,
    I = SingleInitialState,
> = ProbabilisticModel<DtmcType<P, V, AP, I>>;
pub struct DtmcType<
    P: Predecessors = NonTrackedPredecessors,
    V: Valuation = ValuationVector,
    AP: AtomicPropositions = BitFlagsAtomicPropositions,
    I: InitialStates = SingleInitialState,
> {
    _phantom_data: PhantomData<(P, V, AP, I)>,
}
impl<P: Predecessors, V: Valuation, AP: AtomicPropositions, I: InitialStates> ModelTypes
    for DtmcType<P, V, AP, I>
{
    type Valuation = V;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = SingleAction<DistributionVector>;
    type AtomicPropositions = AP;
    type InitialStates = I;
    type Predecessors = P;
}

pub type TransitionSystem<
    P = NonTrackedPredecessors,
    V = ValuationVector,
    AP = BitFlagsAtomicPropositions,
    I = SingleInitialState,
> = ProbabilisticModel<TransitionSystemType<P, V, AP, I>>;
pub struct TransitionSystemType<
    P: Predecessors = NonTrackedPredecessors,
    V: Valuation = ValuationVector,
    AP: AtomicPropositions = BitFlagsAtomicPropositions,
    I: InitialStates = SingleInitialState,
> {
    _phantom_data: PhantomData<(P, V, AP, I)>,
}
impl<P: Predecessors, V: Valuation, AP: AtomicPropositions, I: InitialStates> ModelTypes
    for TransitionSystemType<P, V, AP, I>
{
    type Valuation = V;
    type Distribution = SingleStateDistribution;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<SingleStateDistribution>;
    type AtomicPropositions = AP;
    type InitialStates = I;
    type Predecessors = P;
}

pub type TwoPlayerStochasticGame<
    P = NonTrackedPredecessors,
    V = ValuationVector,
    AP = BitFlagsAtomicPropositions,
    I = SingleInitialState,
> = ProbabilisticModel<TwoPlayerStochasticGameType<P, V, AP, I>>;
pub struct TwoPlayerStochasticGameType<
    P: Predecessors = NonTrackedPredecessors,
    V: Valuation = ValuationVector,
    AP: AtomicPropositions = BitFlagsAtomicPropositions,
    I: InitialStates = SingleInitialState,
> {
    _phantom_data: PhantomData<(P, V, AP, I)>,
}
impl<P: Predecessors, V: Valuation, AP: AtomicPropositions, I: InitialStates> ModelTypes
    for TwoPlayerStochasticGameType<P, V, AP, I>
{
    type Valuation = V;
    type Distribution = DistributionVector;
    type Owners = TwoPlayer;
    type ActionCollection = ActionVector<DistributionVector>;
    type AtomicPropositions = AP;
    type InitialStates = I;
    type Predecessors = P;
}

pub type TwoPlayerNonstochasticGame<
    P = NonTrackedPredecessors,
    V = ValuationVector,
    AP = BitFlagsAtomicPropositions,
    I = SingleInitialState,
> = ProbabilisticModel<TwoPlayerNonstochasticGameType<P, V, AP, I>>;
pub struct TwoPlayerNonstochasticGameType<
    P: Predecessors = NonTrackedPredecessors,
    V: Valuation = ValuationVector,
    AP: AtomicPropositions = BitFlagsAtomicPropositions,
    I: InitialStates = SingleInitialState,
> {
    _phantom_data: PhantomData<(P, V, AP, I)>,
}
impl<P: Predecessors, V: Valuation, AP: AtomicPropositions, I: InitialStates> ModelTypes
    for TwoPlayerNonstochasticGameType<P, V, AP, I>
{
    type Valuation = V;
    type Distribution = SingleStateDistribution;
    type Owners = TwoPlayer;
    type ActionCollection = ActionVector<SingleStateDistribution>;
    type AtomicPropositions = AP;
    type InitialStates = I;
    type Predecessors = P;
}
