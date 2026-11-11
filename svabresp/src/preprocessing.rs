use crate::grouping::StateIdx;
use prism_model_builder::To1;
use probabilistic_models::DefaultIx;
use probabilistic_models::traits::ReadStateSpace;

pub fn potentially_responsible_states<M: ReadStateSpace<Ix = DefaultIx>>(
    model: &M,
) -> To1<StateIdx, bool> {
    To1::with_entries(
        model
            .states()
            .into_iter()
            .map(|state| model.choices_of_state(state).len() >= 2)
            .collect::<Vec<_>>(),
    )
}
