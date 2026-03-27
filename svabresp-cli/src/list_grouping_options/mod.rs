use clap::{Arg, ArgMatches, Command};
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
        let mut output = Vec::new();

        output.push("{");
        output.push("}");

        println!("{}", output.join(""));
    }
}
