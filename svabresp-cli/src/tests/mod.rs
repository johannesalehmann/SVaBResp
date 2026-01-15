use std::str::FromStr;
use svabresp::num_rational::BigRational;
use svabresp::shapley::{BruteForceAlgorithm, ResponsibilityValues};
use svabresp::state_based::grouping::{
    IndividualGroupExtractionScheme, LabelGroupExtractionScheme, ModuleExtractionScheme,
    ValueGroupExtractionScheme,
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

#[test]
fn value_groups() {
    let task = ResponsibilityTask {
        model_description: ModelFromString::new(
            "value-groups.prism",
            include_str!("files/value-groups.prism"),
            "P=1 [F \"obj\"]",
        ),
        constants: "".to_string(),
        coop_game_type: svabresp::CoopGameType::<CounterexampleFile>::Forward,
        algorithm: BruteForceAlgorithm::new(),
        grouping_scheme: ValueGroupExtractionScheme::new(vec![
            "x".to_string(),
            "z".to_string(),
            "w".to_string(),
        ]),
    };
    let result = task.run();

    assert_res("(x=-2, z=true, w=4)", "1/5", &result);
    assert_res("(x=1, z=true, w=4)", "1/5", &result);
    assert_res("(x=2, z=false, w=4)", "0/1", &result);
    assert_res("(x=0, z=true, w=5)", "0/1", &result);
}

#[test]
fn module_groups() {
    let task = ResponsibilityTask {
        model_description: ModelFromString::new(
            "module-groups.prism",
            include_str!("files/module-groups.prism"),
            "P=1 [G !\"obj\"]",
        ),
        constants: "".to_string(),
        coop_game_type: svabresp::CoopGameType::<CounterexampleFile>::Forward,
        algorithm: BruteForceAlgorithm::new(),
        grouping_scheme: ModuleExtractionScheme::new(),
    };
    let result = task.run();
    for res in result.players.iter() {
        println!("{}: {}", res.player_info, res.value);
    }

    assert_res("scheduler", "0", &result);
    assert_res("Window", "0", &result);
    assert_res("Rebeca", "2/3", &result);
    assert_res("Ada", "1/6", &result);
    assert_res("Julia", "1/6", &result);
    assert_res("install_window", "0", &result);
    assert_res("julia_throws", "0", &result);
    assert_res("ada_throws", "0", &result);
}

fn assert_res(name: &str, value: &str, result: &ResponsibilityValues<String>) {
    assert_eq!(
        result
            .get(&(name.to_string()))
            .unwrap_or_else(|| panic!("No responsibility value entry for `{}`", name))
            .value,
        BigRational::from_str(value).unwrap()
    );
}
