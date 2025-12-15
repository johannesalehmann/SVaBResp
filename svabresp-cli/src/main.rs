use svabresp::shapley::BruteForceAlgorithm;

fn main() {
    let file_name = "/Users/johannes/repo/Work/BW-Responsibility/code/experiments/dining_philosophers/dining_philosophers.prism"; //"svabresp-cli/examples/test.prism";
    let file = std::fs::read_to_string(file_name).expect("Failed to read input model");

    let parsed = tiny_pmc::parsing::parse_prism_and_print_errors(
        Some("dining_philosophers.prism"),
        &file[..],
        &["P=1 [F \"sbar\"]"],
    );

    if parsed.is_none() {
        return;
    }
    let (model, properties) = parsed.unwrap();
    let property = properties.into_iter().nth(0).unwrap();

    let mut shapley = BruteForceAlgorithm::new();

    let responsibility = svabresp::state_based::compute_for_prism(
        model,
        property,
        svabresp::state_based::grouping::IndividualGroupExtractionScheme::new(),
        &mut shapley,
    );

    println!("Responsibility values:");
    for (index, value) in responsibility.states.iter().enumerate() {
        println!("  {}: {}", index, value.value);
    }
}
