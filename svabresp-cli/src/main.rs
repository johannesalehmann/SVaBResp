use clap::Parser;
use svabresp::compute_responsibility_from_files;
use svabresp::grouping::{
    ActionGrouping, GroupingScheme, IndividualGrouping, LabelGrouping, ModuleGrouping,
    VariableGrouping,
};

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
    Labels(Vec<String>),
}

fn parse_grouping(s: &str) -> Result<Grouping, String> {
    match s.trim() {
        "individual" => Ok(Grouping::Individual),
        "actions" => Ok(Grouping::Actions),
        "modules" => Ok(Grouping::Modules),
        s if s.starts_with("variables") => {
            parse_list(s, "variables", "x1 x2 ... xn").map(Grouping::Variables)
        }
        s if s.starts_with("labels") => {
            parse_list(s, "labels", "l1 l2 ... ln").map(Grouping::Labels)
        }
        s => Err(format!(
            "expected `individual`, `actions`, `modules`, `variables(x1 x2 ... xn)` or `labels(l1 l2 ... ln)`, got `{s}`"
        )),
    }
}

/// Parses `name(a b ... z)` into `[a, b, ..., z]`. `placeholder` describes the list in errors.
fn parse_list(s: &str, name: &str, placeholder: &str) -> Result<Vec<String>, String> {
    s.strip_prefix(name)
        .map(str::trim)
        .and_then(|s| s.strip_prefix('('))
        .and_then(|s| s.strip_suffix(')'))
        .map(|list| list.split_whitespace().map(String::from).collect())
        .ok_or_else(|| format!("expected `{name}({placeholder})`, got `{s}`"))
}

impl Grouping {
    fn scheme(&self) -> Box<dyn GroupingScheme> {
        match self {
            Grouping::Individual => Box::new(IndividualGrouping),
            Grouping::Actions => Box::new(ActionGrouping),
            Grouping::Modules => Box::new(ModuleGrouping::new()),
            Grouping::Variables(variables) => Box::new(VariableGrouping::new(variables.clone())),
            Grouping::Labels(labels) => Box::new(LabelGrouping::new(labels.clone())),
        }
    }
}

fn main() {
    let arguments = Arguments::parse();
    let mut grouping = arguments.grouping.scheme();
    let responsibility = compute_responsibility_from_files(
        &arguments.files,
        &arguments.constants,
        grouping.as_mut(),
    );
    println!("{responsibility}");
}
