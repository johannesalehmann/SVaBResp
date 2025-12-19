use svabresp::num_rational::BigRational;
use svabresp::num_traits::{ToPrimitive, Zero};
use svabresp::shapley::BruteForceAlgorithm;

fn main() {
    let file_name = "svabresp-cli/examples/small.prism"; // "/Users/johannes/repo/Work/BW-Responsibility/code/experiments/dresden_misrouted_train/dresden_railways.prism";
    // let file_name = "/Users/johannes/repo/Work/BW-Responsibility/code/experiments/dresden_misrouted_train/dresden_railways.prism";
    let file = std::fs::read_to_string(file_name).expect("Failed to read input model");

    let parsed = tiny_pmc::parsing::parse_prism_and_print_errors(
        Some("small.prism"),
        &file[..],
        &["P=1 [G !\"obj\"]"],
        // &["P=1 [G !\"sbar\"]"],
    );

    if parsed.is_none() {
        return;
    }
    let (model, properties) = parsed.unwrap();
    let property = properties.into_iter().nth(0).unwrap();
    println!("{:?}", property);

    let mut shapley = BruteForceAlgorithm::new();

    let responsibility = svabresp::state_based::compute_for_prism(
        model,
        property,
        svabresp::state_based::grouping::IndividualGroupExtractionScheme::new(),
        &mut shapley,
    );

    println!("Responsibility values:");
    let mut sum = BigRational::zero();
    for (index, value) in responsibility.states.iter().enumerate() {
        println!(
            "  {}: {} ({})",
            value.player_info,
            value.value,
            value.value.to_f32().unwrap()
        );
        sum += &value.value;
    }
    println!("Total: {}", sum);
}
