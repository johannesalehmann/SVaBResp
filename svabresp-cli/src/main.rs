#[cfg(test)]
mod tests;

use svabresp::num_traits::ToPrimitive;
use svabresp::shapley::{BruteForceAlgorithm, ResponsibilityValues, ShapleyAlgorithm};

use clap::{Arg, Command, arg};
use env_logger::Target;
use log::{LevelFilter, info, trace};
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
    refinement_initial_partition: RefinementInitialPartition,
    refinement_block_selection: RefinementBlockSelection,
    refinement_splitting: RefinementSplitting,
    grouping: GroupingKind,
    output: OutputKind,
    logging_level: LoggingLevel,
}

enum AlgorithmKind {
    BruteForce,
    Stochastic,
    Refinement,
}

enum RefinementInitialPartition {
    Singleton,
    Random { block_count: usize },
}

enum RefinementBlockSelection {
    Random { block_count: usize },
}

enum RefinementSplitting {
    Random,
    FrontierRandom,
    FrontierPreferPotentiallyWinning,
    FrontierPreferPotentiallyLosing,
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
        let matches =
            Command::new("svabresp").about("Computes responsibility values")
            .arg(arg!(-a --algorithm <ALGORITHM> "The algorithm that is used to compute the responsibility values. Legal values are `brute-force`, `stochastic`, `refinement`.").default_value("brute-force"))
            .arg(arg!(-g --grouping <GROUPING> "The scheme that is used to group states. Legal values are `individual`, `labels([space-separated list of label names])`, `modules`, `actions`, `variables([space-separated list of variable names])`.").default_value("individual"))
            .arg(arg!(-o --output <OUTPUT> "How the output should be presented. Legal values are `human-readable`, `parsable` (simple format that can be processed by other tools) and `silent` (no output).").default_value("human-readable"))
            .arg(arg!(-c --constants <CONSTANTS> "Values for the undefined constants in the model").required(false))
            .arg(arg!(-l --logging <LEVEL> "The level of detail for the logs. Legal values are `error`, `warn`, `info`, `debug` and `trace`.").default_value("warn"))
            .arg(arg!(--initialpartition <HEURISTICS> "Refinement algorithm: The heuristics used to construct the initial partition. Legal values are `singleton` and `random(<INTEGER>)`, where <INTEGER> is a positive integer.").default_value("singleton"))
            .arg(arg!(--blockselection <HEURISTICS> "Refinement algorithm: The heuristics used to select a block for refinement. Legal values are `random`. Every value may be succeeded immediately by `(<INTEGER>)`, where <INTEGER> is a positive integer. This indicates how many blocks should be refined in a single iteration.").default_value("random(1)"))
            .arg(arg!(--splitting <HEURISTICS> "Refinement algorithm: The heuristics used to split a block. Legal values are `random`, `frontier(random)`, `frontier(prefer_winning)` and `frontier(prefer_losing)`.").default_value("frontier(random)"))
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

        let refinement_initial_partition = match matches
            .get_one::<String>("initialpartition")
            .unwrap()
            .as_str()
        {
            i if i.starts_with("random") => {
                let count_string = i["random".len()..].trim();

                if !count_string.starts_with("(") || !count_string.ends_with(")") {
                    panic!(
                        "Invalid argument `{}` for --initialpartition. A valid argument for the random heuristics must have form random(<INTEGER>), where <INTEGER> is a positive integer",
                        i
                    );
                }
                let count_string = count_string[1..count_string.len() - 1].trim();
                let blocks = match count_string.parse::<usize>() {
                    Ok(val) => val,
                    Err(err) => panic!("Could not parse `{}` as integer", count_string),
                };
                RefinementInitialPartition::Random {
                    block_count: blocks,
                }
            }
            "singleton" => RefinementInitialPartition::Singleton,
            i => panic!("Unknown initial partition option `{}`", i),
        };

        let refinement_block_selection = match matches
            .get_one::<String>("blockselection")
            .unwrap()
            .as_str()
        {
            "random" => RefinementBlockSelection::Random { block_count: 1 },
            b if b.starts_with("random") => {
                let count_string = b["random".len()..].trim();

                if !count_string.starts_with("(") || !count_string.ends_with(")") {
                    panic!(
                        "Invalid argument `{}` for --blockselection. A valid argument for the random heuristics must have form random(<INTEGER>), where <INTEGER> is a positive integer",
                        b
                    );
                }
                let count_string = count_string[1..count_string.len() - 1].trim();
                let blocks = match count_string.parse::<usize>() {
                    Ok(val) => val,
                    Err(err) => panic!("Could not parse `{}` as integer", count_string),
                };
                RefinementBlockSelection::Random {
                    block_count: blocks,
                }
            }
            b => panic!("Unknown block selection option `{}`", b),
        };

        let refinement_splitting = match matches.get_one::<String>("splitting").unwrap().as_str() {
            "random" => RefinementSplitting::Random,
            "frontier(random)" | "frontier" => RefinementSplitting::FrontierRandom,
            "frontier(prefer_winning)" => RefinementSplitting::FrontierPreferPotentiallyWinning,
            "frontier(prefer_losing)" => RefinementSplitting::FrontierPreferPotentiallyLosing,
            s => panic!("Unknown splitting heuristics `{}`", s),
        };

        Cli {
            model,
            property,
            constants,
            algorithm,
            refinement_initial_partition,
            refinement_block_selection,
            refinement_splitting,
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
    let start = std::time::Instant::now();
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
        OutputKind::HumanReadable => {
            info!(
                "Computed responsibility in {:?} (including the time for model building)",
                start.elapsed()
            );
            printer.print_human_readable(output)
        }
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
