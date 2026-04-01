use clap::{Arg, ArgMatches, Command};
use std::fmt::{Display, Formatter};
use svabresp::{ModelAndPropertySource, ModelFromFile};

pub struct ListGroupingOptionsCommand {
    model: String,
}

impl ListGroupingOptionsCommand {
    pub fn get_subcommand() -> Command {
        Command::new("list-grouping-options").arg(
            Arg::new("model")
                .required(true)
                .help("File name of the PRISM model file"),
        )
    }

    pub fn from_matches(matches: &ArgMatches) -> Self {
        let model = matches
            .get_one::<String>("model")
            .expect("Model name must be specified")
            .clone();
        Self { model }
    }

    pub fn execute(self) {
        // To reuse the ModelFromFile struct, we need a property. Because we do not have one (the
        // available grouping options don't depend on it), we use the following formula, which works
        // for any model.
        let property = "P=? [F true]";
        let model_description = ModelFromFile::new(self.model.clone(), property);

        let (prism_model, _) = model_description.get_model_and_property();

        let mut variables = Vec::new();
        for variable in prism_model.variable_manager.variables {
            if !variable.is_constant {
                variables.push(GroupingOptionValue {
                    name: format!(
                        "{} ({}{})",
                        variable.name.name,
                        variable.range,
                        variable
                            .initial_value
                            .map_or("".to_string(), |i| format!(" init {}", i))
                    ),
                    value: variable.name.name.clone(),
                });
            }
        }

        let mut labels = Vec::new();
        for label in prism_model.labels.labels {
            labels.push(GroupingOptionValue {
                name: format!("{} ({})", label.name.name, label.condition),
                value: label.name.name.clone(),
            });
        }

        let mut grouping_options = Vec::new();
        grouping_options.push(GroupingOption {
            name: "by module".to_string(),
            args: "-g modules".to_string(),
            kind: GroupingOptionKind::Simple,
            values: None,
        });
        grouping_options.push(GroupingOption {
            name: "by action".to_string(),
            args: "-g actions".to_string(),
            kind: GroupingOptionKind::Simple,
            values: None,
        });
        grouping_options.push(GroupingOption {
            name: "by variable".to_string(),
            args: "-g variables($options)".to_string(),
            kind: GroupingOptionKind::Multiselect,
            values: Some(variables),
        });
        grouping_options.push(GroupingOption {
            name: "by label".to_string(),
            args: "-g labels($options)".to_string(),
            kind: GroupingOptionKind::Multiselect,
            values: Some(labels),
        });

        let mut output = Vec::new();

        let indent = "    ";

        output.push("[\n".to_string());
        let mut first = true;
        for grouping_option in grouping_options {
            if !first {
                output.push(",\n\n".to_string());
            }
            first = false;
            output.push(indent.to_string());
            output.push(grouping_option.to_json(format!("\n{indent}"), indent))
        }
        output.push("\n]".to_string());

        println!("{}", output.join(""));
    }
}

enum GroupingOptionKind {
    Simple,
    Multiselect,
}

impl Display for GroupingOptionKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupingOptionKind::Simple => {
                write!(f, "simple")
            }
            GroupingOptionKind::Multiselect => {
                write!(f, "multiselect")
            }
        }
    }
}

struct GroupingOptionValue {
    name: String,
    value: String,
}

impl GroupingOptionValue {
    fn to_json<S: Display, S2: Display>(&self, new_line: S, indent: S2) -> String {
        format!(
            "{{{new_line}{indent}\"name\": \"{}\",{new_line}{indent}\"value\": \"{}\"{new_line}}}",
            self.name, self.value
        )
    }
}

struct GroupingOption {
    name: String,
    args: String,
    kind: GroupingOptionKind,
    values: Option<Vec<GroupingOptionValue>>,
}

impl GroupingOption {
    pub fn to_json<S: Display, S2: Display>(&self, new_line: S, indent: S2) -> String {
        let options = match &self.values {
            Some(options) => format!(
                ",{new_line}{indent}\"options\": [{new_line}{indent}{indent}{}{new_line}{indent}]",
                options
                    .iter()
                    .map(|o| o.to_json(format!("{new_line}{indent}{indent}"), &indent))
                    .collect::<Vec<_>>()
                    .join(&format!(",{new_line}{indent}{indent}"))
            ),
            None => "".to_string(),
        };
        format!(
            "{{{new_line}{indent}\"name\": \"{}\",{new_line}{indent}\"cli_args\": \"{}\",{new_line}{indent}\"kind\": \"{}\"{}{new_line}}}",
            self.name, self.args, self.kind, options
        )
    }
}
