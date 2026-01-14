#[cfg(test)]
mod tests;

use svabresp::num_rational::BigRational;
use svabresp::num_traits::{ToPrimitive, Zero};
use svabresp::shapley::{BruteForceAlgorithm, ResponsibilityValues, ShapleyAlgorithm};

use clap::{Arg, Command, arg};
use svabresp::state_based::grouping::{GroupExtractionScheme, IndividualGroupExtractionScheme};
use svabresp::{
    CoopGameType, CounterexampleFile, ModelAndPropertySource, ModelFromFile, ResponsibilityTask,
};

struct Cli {
    model: String,
    property: String,
    constants: String,
    algorithm: AlgorithmKind,
    grouping: GroupingKind,
    output: OutputKind,
}

enum AlgorithmKind {
    BruteForce,
    Stochastic,
}

enum GroupingKind {
    Individual,
    Labels { labels: Vec<String> },
    Modules,
    Actions,
    Variables { variables: Vec<String> },
}

enum OutputKind {
    HumanReadable,
    Parsable,
    Silent,
}

impl Cli {
    pub fn from_arguments() -> Self {
        let matches =         Command::new("svabresp").about("Computes responsibility values")
            .arg(arg!(-a --algorithm <ALGORITHM> "The algorithm that is used to compute the responsibility values. Legal values are `brute-force`, `stochastic`, `refinement`. The default is `brute-force`").default_value("brute-force"))
            .arg(arg!(-g --grouping <GROUPING> "The scheme that is used to group states. Legal values are `individual`, `labels [space-separated list of label names]`, `modules`, `actions`, `variables [space-separated list of variable names]`. The default is `individual`").default_value("individual"))
            .arg(arg!(-o --output <OUTPUT> "How the output should be presented. Legal values are `human-readable`, `parsable` (simple format that can be processed by other tools) and `silent` (no output). The default is `human-readable`").default_value("human-readable"))
            .arg(arg!(-c --constants <CONSTANTS> "Values for the undefined constants in the model").required(false))
            .arg(Arg::new("model").required(true).help("File name of the PRISM model file"))
            .arg(Arg::new("property").required(true).help("Property to be checked, given in PRISM property language"))
            .get_matches();

        let model = matches
            .get_one::<String>("model")
            .expect("Model name must be specified")
            .clone();
        let property = matches
            .get_one::<String>("property")
            .expect("Model name must be specified")
            .clone();
        let algorithm = match matches.get_one::<String>("algorithm").unwrap().as_str() {
            "brute-force" => AlgorithmKind::BruteForce,
            a => panic!("Unknown algorithm `{}`", a),
        };
        let grouping = match matches.get_one::<String>("grouping").unwrap().as_str() {
            "individual" => GroupingKind::Individual,
            a => panic!("Unknown grouping scheme `{}`", a),
        };
        let output = match matches.get_one::<String>("output").unwrap().as_str() {
            "human-readable" => OutputKind::HumanReadable,
            a => panic!("Unknown output kind `{}`", a),
        };
        let constants = match matches.get_one::<String>("constants") {
            Some(c) => c.clone(),
            None => "".to_string(),
        };

        Cli {
            model,
            property,
            constants,
            algorithm,
            grouping,
            output,
        }
    }
}

fn main() {
    let cli = Cli::from_arguments();

    execute_cli(cli);
}

fn execute_cli(cli: Cli) {
    let model_description = ModelFromFile::new(cli.model.clone(), cli.property.clone());
    execute_with_model_description(cli, model_description);
}

fn execute_with_model_description<M: ModelAndPropertySource>(cli: Cli, model_description: M) {
    match cli.grouping {
        GroupingKind::Individual => execute_with_grouping_scheme(
            cli,
            model_description,
            IndividualGroupExtractionScheme::new(),
        ),
        GroupingKind::Labels { .. } => {
            unimplemented!()
        }
        GroupingKind::Modules => {
            unimplemented!()
        }
        GroupingKind::Actions => {
            unimplemented!()
        }
        GroupingKind::Variables { .. } => {
            unimplemented!()
        }
    }
}

fn execute_with_grouping_scheme<M: ModelAndPropertySource, G: GroupExtractionScheme>(
    cli: Cli,
    model_description: M,
    grouping_scheme: G,
) {
    match cli.algorithm {
        AlgorithmKind::BruteForce => execute_with_algorithm(
            cli,
            model_description,
            grouping_scheme,
            BruteForceAlgorithm::new(),
            ResponsibilityValuesPrinter {},
        ),
        AlgorithmKind::Stochastic => {
            panic!("Stochastic algorithm not implemented in cli yet")
        }
    }
}

fn execute_with_algorithm<
    M: ModelAndPropertySource,
    G: GroupExtractionScheme,
    A: ShapleyAlgorithm,
    P: OutputPrinter<A::Output<String>>,
>(
    cli: Cli,
    model_description: M,
    grouping_scheme: G,
    algorithm: A,
    printer: P,
) {
    let task = ResponsibilityTask {
        model_description,
        constants: cli.constants,
        coop_game_type: CoopGameType::<CounterexampleFile>::Forward, // TODO: Make this configurable
        algorithm,
        grouping_scheme,
    };

    let output = task.run();

    match cli.output {
        OutputKind::HumanReadable => printer.print_human_readable(output),
        OutputKind::Parsable => printer.print_parsable(output),
        OutputKind::Silent => {
            // psst!
        }
    }
}

trait OutputPrinter<T> {
    fn print_human_readable(self, output: T);
    fn print_parsable(self, output: T);
}

struct ResponsibilityValuesPrinter {}

impl<PD: std::fmt::Display> OutputPrinter<ResponsibilityValues<PD>>
    for ResponsibilityValuesPrinter
{
    fn print_human_readable(self, output: ResponsibilityValues<PD>) {
        println!("Responsibility values:");
        for player in output.players {
            println!(
                " {}: {} ({})",
                player.player_info,
                player
                    .value
                    .to_f64()
                    .map(|f| format!("{:.6}", f))
                    .unwrap_or_else(|| "err".to_string()),
                player.value
            );
        }
    }

    fn print_parsable(self, output: ResponsibilityValues<PD>) {
        for player in output.players {
            println!(
                "{}:{}",
                player.player_info,
                player
                    .value
                    .to_f64()
                    .map(|f| f.to_string())
                    .unwrap_or_else(|| "err".to_string())
            );
        }
    }
}

fn old_main() {
    let mut start_time = std::time::Instant::now();
    // let file_name = "svabresp-cli/examples/small.prism"; // "/Users/johannes/repo/Work/BW-Responsibility/code/experiments/dresden_misrouted_train/dresden_railways.prism";
    // let file_name = "/Users/johannes/repo/Work/BW-Responsibility/code/experiments/dresden_misrouted_train/dresden_railways.prism";
    let file_name =
        "/Users/johannes/Documents/code/SVaBResp/tiny-pmc-cli/src/tests/files/pacman.v2.prism";
    let file = std::fs::read_to_string(file_name).expect("Failed to read input model");

    let parsed = tiny_pmc::parsing::parse_prism_and_print_errors(
        Some("pacman.v2.prism"),
        &file[..],
        // &["P=1 [G !\"obj\"]"],
        // &["P=1 [G !\"sbar\"]"],
        &["PMin=? [F \"Crash\"]"],
    );

    if parsed.is_none() {
        return;
    }
    let (model, properties) = parsed.unwrap();
    let property = properties.into_iter().nth(0).unwrap();
    println!("{:?}", property);

    let mut constants = std::collections::HashMap::new();
    constants.insert("MAXSTEPS".to_string(), svabresp::ConstValue::Int(5));

    let mut shapley = BruteForceAlgorithm::new();

    let responsibility = svabresp::state_based::compute_for_prism(
        model,
        property,
        svabresp::state_based::grouping::IndividualGroupExtractionScheme::new(),
        &mut shapley,
        constants,
    );

    println!("Responsibility values:");
    let mut sum = BigRational::zero();
    for (index, value) in responsibility.players.iter().enumerate() {
        println!(
            "  {}: {} ({})",
            value.player_info,
            value.value,
            value.value.to_f64().unwrap()
        );
        sum += &value.value;
    }
    println!("Total: {}", sum);
    println!("Finished in {:?}", start_time.elapsed());
}
