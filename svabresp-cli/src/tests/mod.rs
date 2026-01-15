use std::str::FromStr;
use svabresp::num_rational::BigRational;
use svabresp::shapley::{BruteForceAlgorithm, ResponsibilityValues};
use svabresp::state_based::grouping::{
    IndividualGroupExtractionScheme, LabelGroupExtractionScheme,
};
use svabresp::{CounterexampleFile, ModelFromString, ResponsibilityTask};

#[test]
fn small_network_explicit() {
    let task = ResponsibilityTask {
        model_description: ModelFromString::new(
            "small-network.prism",
            include_str!("files/small-network.prism"),
            "P=1 [F \"obj\"]",
        ),
        constants: "".to_string(),
        coop_game_type: svabresp::CoopGameType::<CounterexampleFile>::Forward,
        algorithm: BruteForceAlgorithm::new(),
        grouping_scheme: IndividualGroupExtractionScheme::new(),
    };
    let result = task.run();

    assert_res("(loc=1)", "1/12", &result);
    assert_res("(loc=2)", "1/12", &result);
    assert_res("(loc=3)", "1/4", &result);
    assert_res("(loc=4)", "7/12", &result);
}

#[test]
fn labelled_groups() {
    let task = ResponsibilityTask {
        model_description: ModelFromString::new(
            "labelled-groups.prism",
            include_str!("files/labelled-groups.prism"),
            "P=1 [F \"obj\"]",
        ),
        constants: "".to_string(),
        coop_game_type: svabresp::CoopGameType::<CounterexampleFile>::Forward,
        algorithm: BruteForceAlgorithm::new(),
        grouping_scheme: LabelGroupExtractionScheme::new(vec![
            "l3".to_string(),
            "l1".to_string(),
            "l2".to_string(),
            "dummy".to_string(),
        ]),
    };
    let result = task.run();

    assert_res("l1, l2", "1/12", &result);
    assert_res("l2", "1/12", &result);
    assert_res("l1", "1/4", &result);
    assert_res("no labels", "7/12", &result);
}

fn assert_res(name: &str, value: &str, result: &ResponsibilityValues<String>) {
    assert_eq!(
        result.get(&(name.to_string())).unwrap().value,
        BigRational::from_str(value).unwrap()
    );
}
