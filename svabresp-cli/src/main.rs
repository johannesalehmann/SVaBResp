use clap::Parser;
use svabresp::grouping::{GroupingScheme, IndividualGrouping};

#[derive(Parser)]
#[command(version, about = "Computes responsibility values")]
struct Arguments {
    #[arg(value_name = "MODEL_OR_PROPERTY", num_args = 1.., required = true)]
    files: Vec<String>,
    #[arg(short, long, default_value = "individual", value_parser = parse_grouping)]
    grouping: Grouping,
    #[arg(short, long, default_value_t = String::new())]
    constants: String,
}

#[derive(Clone, Debug)]
enum Grouping {
    Individual,
    Actions,
    Modules,
    Variables(Vec<String>),
}

fn parse_grouping(s: &str) -> Result<Grouping, String> {
    match s.trim() {
        "individual" => Ok(Grouping::Individual),
        "actions" => Ok(Grouping::Actions),
        "modules" => Ok(Grouping::Modules),
        s => s
            .strip_prefix("variables")
            .map(str::trim)
            .and_then(|s| s.strip_prefix('('))
            .and_then(|s| s.strip_suffix(')'))
            .map(|vars| Grouping::Variables(vars.split_whitespace().map(String::from).collect()))
            .ok_or_else(|| {
                format!("expected `individual`, `actions`, `modules` or `variables(x1 x2 ... xn)`, got `{s}`")
            }),
    }
}

impl Grouping {
    fn scheme(&self) -> Box<dyn GroupingScheme> {
        match self {
            Grouping::Individual => Box::new(IndividualGrouping),
            Grouping::Actions | Grouping::Modules | Grouping::Variables(_) => {
                unimplemented!("Grouping {self:?} is not yet supported")
            }
        }
    }
}

fn main() {
    let arguments = Arguments::parse();
    let mut grouping = arguments.grouping.scheme();
    let built = svabresp::build::build(&arguments.files, &arguments.constants, grouping.as_mut());
    println!("{}", built.model.info());
    println!("Groups: {}", built.groups.len());
}
