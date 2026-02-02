mod table;

use crate::table::Table;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

use wait_timeout::ChildExt;

#[tokio::main]
async fn main() {
    evaluate_initial_partition_heuristics().await;
    evaluate_block_selection_heuristics().await;
    evaluate_block_splitting_heuristics().await;
}

fn model_base_path() -> &'static str {
    "svabresp-benchmarking/src/models/"
}

fn get_heuristics_models() -> Vec<ModelSource> {
    vec![
        // ModelSource::new("dekker", "dekker.prism", "P=1 [G F \"obj\"]"),
        ModelSource::new("generals", "generals_3.prism", "P=1 [G !\"obj\"]"),
        ModelSource::new("railway", "railway.prism", "P=1 [F \"obj\"]"),
        ModelSource::new("station", "station.prism", "P=1 [G !\"obj\"]"),
        ModelSource::new(
            "philosophers",
            "dining_philosophers.prism",
            "P=1 [G !\"obj\"]",
        ),
        //ModelSource::new("clouds", "clouds.prism", "P=1 [F \"obj\"]"),
    ]
}

fn get_timeout() -> Duration {
    Duration::from_secs(30)
}

fn get_initial_partition_refinements() -> (Table, Vec<Vec<String>>) {
    let ks = vec![1, 2, 3, 4, 5];

    let mut table = Table::new();
    table.start_new_header();
    table.add_to_header("\\emph{{no refinement}}", 1);

    let mut blocking_providers = Vec::with_capacity(ks.len() + 1);
    blocking_providers.push(vec!["--algorithm".to_string(), "brute-force".to_string()]);
    for &k in &ks {
        table.add_to_header(format!("$n={}$", k), 1);
        blocking_providers.push(vec![
            "--algorithm".to_string(),
            "refinement".to_string(),
            "--initialpartition".to_string(),
            format!("random({})", k),
            "--blockselection".to_string(),
            "random".to_string(),
            "--splitting".to_string(),
            "frontier(random)".to_string(),
        ]);
    }
    (table, blocking_providers)
}
fn get_block_selection_refinements() -> (Table, Vec<Vec<String>>) {
    let mut table = Table::new();
    table.start_new_header();
    table.add_to_header("\\emph{{random}}", 1);
    table.add_to_header("\\emph{{max-$\\Delta$}}", 1);
    table.add_to_header("\\emph{{min-$\\Delta$}}", 1);
    table.add_to_header("\\emph{{min-frontier}}", 1);

    let mut blocking_providers = Vec::new();

    let block_selections = ["random", "max-delta", "min-delta", "min-frontier"];

    for block_selection in block_selections {
        blocking_providers.push(vec![
            "--algorithm".to_string(),
            "refinement".to_string(),
            "--initialpartition".to_string(),
            "singleton".to_string(),
            "--blockselection".to_string(),
            block_selection.to_string(),
            "--splitting".to_string(),
            "frontier(random)".to_string(),
        ]);
    }

    (table, blocking_providers)
}
fn get_splitting_heuristics_refinements() -> (Table, Vec<Vec<String>>) {
    let mut table = Table::new();
    table.start_new_header();
    table.add_to_header("\\emph{{random}}", 1);
    table.add_to_header("\\emph{{frontier-any}}", 1);
    table.add_to_header("\\emph{{frontier-to-losing}}", 1);
    table.add_to_header("\\emph{{frontier-to-winning}}", 1);

    let mut blocking_providers = Vec::new();

    let splitting_heuristicses = [
        "random",
        "frontier(random)",
        "frontier(prefer_losing)",
        "frontier(prefer_winning)",
    ];

    for splitting_heuristics in splitting_heuristicses {
        blocking_providers.push(vec![
            "--algorithm".to_string(),
            "refinement".to_string(),
            "--initialpartition".to_string(),
            "singleton".to_string(),
            "--blockselection".to_string(),
            "random".to_string(),
            "--splitting".to_string(),
            splitting_heuristics.to_string(),
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
        get_initial_partition_refinements,
    )
    .await;
}

async fn evaluate_block_selection_heuristics() {
    produce_table(
        "block selection benchmark",
        get_heuristics_models,
        get_block_selection_refinements,
    )
    .await;
}

async fn evaluate_block_splitting_heuristics() {
    produce_table(
        "block splitting benchmark",
        get_heuristics_models,
        get_splitting_heuristics_refinements,
    )
    .await;
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
