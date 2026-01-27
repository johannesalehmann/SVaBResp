#[cfg(test)]
mod tests;

use svabresp::num_traits::ToPrimitive;
use svabresp::shapley::{BruteForceAlgorithm, ResponsibilityValues, ShapleyAlgorithm};

use clap::{Arg, Command, arg};
use env_logger::Target;
use log::{LevelFilter, trace};
use svabresp::state_based::grouping::{
    ActionGroupExtractionScheme, GroupExtractionScheme, IndividualGroupExtractionScheme,
    LabelGroupExtractionScheme, ModuleGroupExtractionScheme, ValueGroupExtractionScheme,
};
use svabresp::state_based::refinement::{
    FrontierSplittingHeuristics, GroupBlockingProvider, IdentityGroupBlockingProvider,
    RandomBlockSelectionHeuristics, RefinementGroupBlockingProvider, SingletonInitialPartition,
};
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
    logging_level: LoggingLevel,
}

enum AlgorithmKind {
    BruteForce,
    Stochastic,
    Refinement,
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

enum LoggingLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Cli {
    pub fn from_arguments() -> Self {
        let matches =         Command::new("svabresp").about("Computes responsibility values")
            .arg(arg!(-a --algorithm <ALGORITHM> "The algorithm that is used to compute the responsibility values. Legal values are `brute-force`, `stochastic`, `refinement`. The default is `brute-force`").default_value("brute-force"))
            .arg(arg!(-g --grouping <GROUPING> "The scheme that is used to group states. Legal values are `individual`, `labels([space-separated list of label names])`, `modules`, `actions`, `variables([space-separated list of variable names])`. The default is `individual`").default_value("individual"))
            .arg(arg!(-o --output <OUTPUT> "How the output should be presented. Legal values are `human-readable`, `parsable` (simple format that can be processed by other tools) and `silent` (no output). The default is `human-readable`").default_value("human-readable"))
            .arg(arg!(-c --constants <CONSTANTS> "Values for the undefined constants in the model").required(false))
            .arg(arg!(-l --logging <LEVEL> "The level of detail for the logs. Legal values are `error`, `warn`, `info`, `debug` and `trace`.").default_value("warn"))
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
            "stochastic" => AlgorithmKind::Stochastic,
            "refinement" => AlgorithmKind::Refinement,
            a => panic!("Unknown algorithm `{}`", a),
        };
        let grouping = match matches.get_one::<String>("grouping").unwrap().as_str() {
            "individual" => GroupingKind::Individual,
            g if g.starts_with("labels") => {
                let labels = Self::parse_space_separated_names(
                    &g["labels".len()..],
                    "--grouping labels must include a parenthesised list of label names, e.g. --grouping labels(foo bar)",
                );

                GroupingKind::Labels { labels }
            }
            "modules" => GroupingKind::Modules,
            "actions" => GroupingKind::Actions,
            g if g.starts_with("variables") => {
                let variables = Self::parse_space_separated_names(
                    &g["labels".len()..],
                    "--grouping variables must include a parenthesised list of variable names, e.g. --grouping variables(x y timer)",
                );

                GroupingKind::Variables { variables }
            }
            g => panic!("Unknown grouping scheme `{}`", g),
        };
        let output = match matches.get_one::<String>("output").unwrap().as_str() {
            "human-readable" => OutputKind::HumanReadable,
            "parsable" => OutputKind::Parsable,
            "silent" => OutputKind::Silent,
            o => panic!("Unknown output kind `{}`", o),
        };
        let constants = match matches.get_one::<String>("constants") {
            Some(c) => c.clone(),
            None => "".to_string(),
        };
        let logging_level = match matches.get_one::<String>("logging").unwrap().as_str() {
            "error" => LoggingLevel::Error,
            "warn" => LoggingLevel::Warn,
            "info" => LoggingLevel::Info,
            "debug" => LoggingLevel::Debug,
            "trace" => LoggingLevel::Trace,
            l => panic!("Unknown logging level `{}`", l),
        };

        Cli {
            model,
            property,
            constants,
            algorithm,
            grouping,
            output,
            logging_level,
        }
    }

    fn parse_space_separated_names(a: &str, error_no_parentheses: &str) -> Vec<String> {
        let names = a.trim();
        if !names.starts_with("(") || names.ends_with(")") {
            panic!("{}", error_no_parentheses);
        }
        let names = &names[1..names.len() - 1];
        let names = names
            .split(' ')
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        names
    }
}

fn main() {
    let cli = Cli::from_arguments();

    execute_cli(cli);
}

fn execute_cli(cli: Cli) {
    let mut logging_builder = env_logger::Builder::from_default_env();
    logging_builder.target(Target::Stdout);
    match cli.logging_level {
        LoggingLevel::Error => {
            logging_builder.filter(None, LevelFilter::Error);
        }
        LoggingLevel::Warn => {
            logging_builder.filter(None, LevelFilter::Warn);
        }
        LoggingLevel::Info => {
            logging_builder.filter(None, LevelFilter::Info);
        }
        LoggingLevel::Debug => {
            logging_builder.filter(None, LevelFilter::Debug);
        }
        LoggingLevel::Trace => {
            logging_builder.filter(None, LevelFilter::Trace);
        }
    }
    logging_builder.init();
    trace!("Trace-level logging enabled");

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
        GroupingKind::Labels { ref labels } => {
            let labels = labels.clone();
            execute_with_grouping_scheme(
                cli,
                model_description,
                LabelGroupExtractionScheme::new(labels),
            )
        }
        GroupingKind::Modules => {
            execute_with_grouping_scheme(cli, model_description, ModuleGroupExtractionScheme::new())
        }
        GroupingKind::Actions => {
            execute_with_grouping_scheme(cli, model_description, ActionGroupExtractionScheme::new())
        }
        GroupingKind::Variables { ref variables } => {
            let variables = variables.clone();
            execute_with_grouping_scheme(
                cli,
                model_description,
                ValueGroupExtractionScheme::new(variables),
            )
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
            IdentityGroupBlockingProvider::new(),
        ),
        AlgorithmKind::Stochastic => {
            panic!("Stochastic algorithm not implemented in cli yet")
        }
        AlgorithmKind::Refinement => execute_with_algorithm(
            cli,
            model_description,
            grouping_scheme,
            BruteForceAlgorithm::new(),
            ResponsibilityValuesPrinter {},
            RefinementGroupBlockingProvider::new(
                SingletonInitialPartition::new(),
                RandomBlockSelectionHeuristics::new(1),
                FrontierSplittingHeuristics::new(),
            ),
        ),
    }
}

fn execute_with_algorithm<
    M: ModelAndPropertySource,
    G: GroupExtractionScheme,
    A: ShapleyAlgorithm,
    P: OutputPrinter<A::Output<String>>,
    B: GroupBlockingProvider,
>(
    cli: Cli,
    model_description: M,
    grouping_scheme: G,
    algorithm: A,
    printer: P,
    refinement: B,
) {
    let task = ResponsibilityTask {
        model_description,
        constants: cli.constants,
        coop_game_type: CoopGameType::<CounterexampleFile>::Forward, // TODO: Make this configurable
        algorithm,
        grouping_scheme,
        refinement,
    };

    trace!("Finished preparing responsibility task");
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
