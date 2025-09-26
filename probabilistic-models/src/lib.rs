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

pub trait ModelTypes: Sized {
    type Valuation: Valuation;
    type Distribution: Distribution;
    type Owners: Owners;
    type ActionCollection: ActionCollection<Self>;

    type AtomicPropositions: AtomicPropositions;
}

pub struct ProbabilisticModel<M: ModelTypes> {
    pub states: Vec<State<M>>,
}

impl<M: ModelTypes> ProbabilisticModel<M> {
    pub fn new() -> Self {
        Self { states: Vec::new() }
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

type Mdp = ProbabilisticModel<MdpType>;
pub struct MdpType {}
impl ModelTypes for MdpType {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<Self>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
}

type Dtmc = ProbabilisticModel<DtmcType>;
pub struct DtmcType {}
impl ModelTypes for DtmcType {
    type Valuation = ValuationVector;
    type Distribution = DistributionVector;
    type Owners = SinglePlayer;
    type ActionCollection = SingleAction<Self>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
}

type TransitionSystem = ProbabilisticModel<TransitionSystemType>;
pub struct TransitionSystemType {}
impl ModelTypes for TransitionSystemType {
    type Valuation = ValuationVector;
    type Distribution = SingleStateDistribution;
    type Owners = SinglePlayer;
    type ActionCollection = ActionVector<Self>;
    type AtomicPropositions = BitFlagsAtomicPropositions;
}
