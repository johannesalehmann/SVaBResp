mod compute_responsibility;
use compute_responsibility::ComputeResponsibilityCommand;

mod list_grouping_options;
use list_grouping_options::ListGroupingOptionsCommand;

#[cfg(test)]
mod tests;

enum SVaBRespCommand {
    ListGroupingOptions(ListGroupingOptionsCommand),
    ComputeResponsibility(ComputeResponsibilityCommand),
}

impl SVaBRespCommand {
    pub fn from_arguments() -> Self {
        let matches = ComputeResponsibilityCommand::get_command()
            .args_conflicts_with_subcommands(true)
            .subcommand(ListGroupingOptionsCommand::get_subcommand())
            .get_matches();

        if let Some(matches) = matches.subcommand_matches("list-grouping-options") {
            Self::ListGroupingOptions(ListGroupingOptionsCommand::from_matches(matches))
        } else {
            Self::ComputeResponsibility(ComputeResponsibilityCommand::from_matches(&matches))
        }
    }
}

fn main() {
    let command = SVaBRespCommand::from_arguments();
    match command {
        SVaBRespCommand::ListGroupingOptions(command) => command.execute(),
        SVaBRespCommand::ComputeResponsibility(command) => command.execute(),
    }
}
