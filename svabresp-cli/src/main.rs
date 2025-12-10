fn main() {
    let file_name = "svabresp-cli/examples/test.prism";
    let file = std::fs::read_to_string(file_name).expect("Failed to read input model");

    let parsed = tiny_pmc::parsing::parse_prism_and_print_errors(
        Some("test.prism"),
        &file[..],
        &["PMax=? [F \"obj\"]"],
    );

    if parsed.is_none() {
        return;
    }
    let (model, properties) = parsed.unwrap();
    let property = properties.into_iter().nth(0).unwrap();

    let coop_game = svabresp::state_based::compute_for_prism(
        model,
        property,
        svabresp::state_based::grouping::IndividualGroupExtractionScheme::new(),
    );

    // TODO: Compute responsibility value here

    println!("We are done here");
}
