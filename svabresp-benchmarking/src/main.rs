mod table;

use crate::table::Table;
use std::time::Duration;
use svabresp::shapley::{BruteForceAlgorithm, ResponsibilityValues};
use svabresp::state_based::grouping::IndividualGroupExtractionScheme;
use svabresp::state_based::refinement::{
    FrontierSplittingHeuristics, GroupBlockingProvider, IdentityGroupBlockingProvider,
    RandomBlockSelectionHeuristics, RandomInitialPartition, RefinementGroupBlockingProvider,
};
use svabresp::{CounterexampleFile, ModelFromFile, ResponsibilityTask};

use indicatif::{ProgressBar, ProgressStyle};

use wait_timeout::ChildExt;

#[tokio::main]
async fn main() {
    evaluate_initial_partition_heuristics().await;
}

fn model_base_path() -> &'static str {
    "svabresp-benchmarking/src/models/"
}

fn get_heuristics_models() -> Vec<ModelSource> {
    vec![
        ModelSource::new("dekker", "dekker.prism", "P=1 [G F \"obj\"]"),
        ModelSource::new("generals", "generals_3.prism", "P=1 [G !\"obj\"]"),
        ModelSource::new("railway", "railway.prism", "P=1 [F \"obj\"]"),
        ModelSource::new("station", "railway.prism", "P=1 [G !\"obj\"]"),
        ModelSource::new("philosophers", "railway.prism", "P=1 [G !\"obj\"]"),
        ModelSource::new("clouds", "railway.prism", "P=1 [F \"obj\"]"),
    ]
}

fn get_timeout() -> Duration {
    Duration::from_secs(1)
}

fn get_initial_partition_refinement_groups() -> (Table, Vec<Vec<String>>) {
    let ks = vec![1, 2, 3, 4, 5];

    let mut table = Table::new();
    table.start_new_header();
    table.add_to_header("\\emph{{no refinement}}", 1);

    let mut blocking_providers = Vec::with_capacity(ks.len() + 1);
    blocking_providers.push(vec![
        "--refinement".to_string(),
        "--initialpartition".to_string(),
        "singleton".to_string(),
        "--blockselection".to_string(),
        "random".to_string(),
        "--splitting".to_string(),
        "frontier(random)".to_string(),
    ]);
    for &k in &ks {
        table.add_to_header(format!("$n={}$", k), 1);
        blocking_providers.push(vec![
            "--refinement".to_string(),
            "--initialpartition".to_string(),
            format!("n({})", k),
            "--blockselection".to_string(),
            "random".to_string(),
            "--splitting".to_string(),
            "frontier(random)".to_string(),
        ]);
    }
    (table, blocking_providers)
}

async fn produce_table<MF: Fn() -> Vec<ModelSource>, RF: Fn() -> (Table, Vec<Vec<String>>)>(
    benchmark_name: &'static str,
    models: MF,
    refinement: RF,
) {
    let models = models();
    let (mut table, refinements) = refinement(); //get_initial_partition_refinement_groups();
    println!("Running {}", benchmark_name);
    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>4}/{len:4} {msg}",
    )
    .unwrap()
    .progress_chars("#>-");
    let pb = ProgressBar::new((models.len() * refinements.len()) as u64);
    pb.set_style(style);

    for model in models {
        table.start_new_row(&model.name);
        for (refinement_index, refinement) in refinement() //get_initial_partition_refinement_groups()
            .1
            .into_iter()
            .enumerate()
        {
            pb.set_message(format!(
                "{} ({}/{})",
                model.name,
                refinement_index,
                refinements.len()
            ));
            let start = std::time::Instant::now();
            let mut child = std::process::Command::new("./target/release/svabresp-cli")
                .arg(model.file.as_str())
                .arg(model.property.as_str())
                .args(refinement.iter())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();

            match child.wait_timeout(get_timeout()).unwrap() {
                Some(status) => {
                    table.add_runtime(start.elapsed().as_secs_f64());
                    if status.code() != Some(0) {
                        println!(
                            "There was an error for configuration \"{}\" \"{}\" {}",
                            model.file,
                            model.property,
                            refinement.join(" ")
                        );
                    }
                }
                None => {
                    child.kill().unwrap();
                    table.add_timeout();
                }
            };
            pb.inc(1);
        }
    }
    pb.finish_with_message("done");

    table.print_latex();
}

async fn evaluate_initial_partition_heuristics() {
    produce_table(
        "initial partition benchmark",
        get_heuristics_models,
        get_initial_partition_refinement_groups,
    )
    .await;
}

async fn run<G: GroupBlockingProvider>(
    model_source: ModelSource,
    refinement: G,
) -> ResponsibilityValues<String> {
    let task = ResponsibilityTask {
        model_description: ModelFromFile::new(
            model_source.file.as_str(),
            model_source.property.as_str(),
        ),
        constants: "".to_string(),
        coop_game_type: svabresp::CoopGameType::<CounterexampleFile>::Forward,
        algorithm: BruteForceAlgorithm::new(),
        grouping_scheme: IndividualGroupExtractionScheme::new(),
        refinement,
    };
    let result = task.run();

    result
}

#[derive(Clone)]
struct ModelSource {
    name: String,
    file: String,
    property: String,
}
impl ModelSource {
    pub fn new(name: &'static str, file: &'static str, property: &'static str) -> Self {
        Self {
            name: name.into(),
            file: format!("{}{}", model_base_path(), file),
            property: property.into(),
        }
    }
}
