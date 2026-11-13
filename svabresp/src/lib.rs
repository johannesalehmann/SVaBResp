use crate::build::build;
use crate::coop_game_solver::build_cooperative_game;
use crate::grouping::{Grouping, GroupingScheme};
use crate::preprocessing::potentially_responsible_states;
use crate::responsibility::Responsibility;
use crate::scheduler::PartialScheduler;
use prism_model_builder::To1;
use prism_model_builder::probabilistic_models::traits::{
    ReadAtomicPropositionsMaybe, ReadInitialStatesMaybe,
};
use prism_model_builder::queries::ProcessedQuery;
use probabilistic_models::traits::{ReadChoiceLabelsMaybe, ReadValuationsMaybe};
use probabilistic_properties::{NonDeterminismKind, PathFormula, Query, StateFormula};
use tiny_pmc::{API, OptionalModel};

pub mod build;
pub mod coop_game;
pub mod coop_game_solver;
pub mod grouping;
pub mod preprocessing;
pub mod responsibility;
pub mod scheduler;
pub mod shapley;

pub fn compute_responsibility_from_files(
    files: &[String],
    constants: &str,
    grouping: &mut dyn GroupingScheme,
    scheduler_file: Option<&str>,
) -> Responsibility {
    let mut built = build(files, constants, grouping);
    let scheduler = scheduler_file.map(|path| parse_scheduler(&mut built.model, path));
    compute_responsibility(
        built.model,
        &built.query,
        built.grouping,
        scheduler.as_ref(),
    )
}

/// `model` is only borrowed mutably to lend its choice labels to the parser; they are restored
/// before returning.
fn parse_scheduler(model: &mut OptionalModel, path: &str) -> PartialScheduler {
    let file = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read scheduler file `{path}`: {e}"));
    let model = model
        .as_ref()
        .base_into_mdp()
        .expect("Schedulers can only be used for MDPs")
        .try_with_choice_labels()
        .expect("Schedulers can only be used for models with choice labels")
        .try_with_valuations()
        .expect("Schedulers can only be used for models with state valuations");
    let scheduler = PartialScheduler::parse(&model, file);
    scheduler
}

pub fn compute_responsibility(
    model: OptionalModel,
    query: &ProcessedQuery<API>,
    grouping: Grouping,
    scheduler: Option<&PartialScheduler>,
) -> Responsibility {
    let model = model
        .try_with_atomic_propositions()
        .expect("Model must contain atomic propositions")
        .try_with_initial_states()
        .expect("Model must contain initial states")
        .try_convert_initial_states_to_single()
        .map_err(|(_, e)| e)
        .expect("Model must contain exactly one initial state");

    let goal = if let Query::ProbabilityValue {
        non_determinism: Some(NonDeterminismKind::Maximise),
        path: PathFormula::Eventually { condition },
    } = query
        && let StateFormula::Expression(goal) = condition.as_ref()
    {
        goal
    } else {
        panic!("Can only handle queries of form `Pmax=? [F some_expression]");
    };

    if model.base_is_mdp() {
        let mdp = model.base_into_mdp().unwrap();
        let goal_states = To1::with_entries(mdp.atomic_propositions[*goal].values().as_slice());

        let potentially_responsible = potentially_responsible_states(&mdp.base);
        let (restricted, link) =
            grouping.restrict_to_potentially_responsible(&potentially_responsible);
        let parallel_groups = std::thread::available_parallelism().map_or(0, |n| n.get().ilog2());
        let coop_game = build_cooperative_game(
            mdp.base,
            mdp.initial.index,
            &goal_states,
            &restricted,
            scheduler,
            parallel_groups as usize,
            1e-6,
        );

        let mut values = vec![0.0; grouping.groups.len()];
        for (index, value) in coop_game.shapley_values().into_iter().enumerate() {
            values[link.new_to_old_index(index)] = value;
        }
        Responsibility::new(grouping.groups, values)
    } else {
        panic!("Unsupported model type: {:?}", model.base.model_type())
    }
}
