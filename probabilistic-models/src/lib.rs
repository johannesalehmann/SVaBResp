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
