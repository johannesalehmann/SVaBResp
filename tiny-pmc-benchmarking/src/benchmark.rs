use prism_model_builder::ModelBuildingOutput;
use prism_model_builder::probabilistic_models::ModelTypes;
use std::time::Duration;

struct BenchmarkResult {
    parsing: Duration,
    building: Duration,
    total: Duration,
    state_count: usize,
}

struct BenchmarkPropertyResult {
    check_time: Duration,
    result: f64,
}

pub struct Benchmark {
    name: String,
    path: String,
    constants: String,
    reference_state_count: usize,
    reference_transition_count: usize,
    reference_build_time: Duration,
    properties: Vec<BenchmarkProperty>,
}

impl Benchmark {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        name: S1,
        path: S2,
        constants: S3,
        reference_state_count: usize,
        reference_transition_count: usize,
        reference_build_time: f64,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            constants: constants.into(),
            reference_state_count,
            reference_transition_count,
            reference_build_time: Duration::from_secs_f64(reference_build_time),
            properties: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_property<S: Into<String>>(
        mut self,
        property: S,
        reference_result: f64,
        reference_check_time: Option<f64>,
    ) -> Self {
        self.properties.push(BenchmarkProperty {
            property: property.into(),
            reference_result,
            reference_check_time: reference_check_time.map(Duration::from_secs_f64),
        });
        self
    }

    fn property_vector(&self) -> Vec<String> {
        self.properties.iter().map(|p| p.property.clone()).collect()
    }

    fn run_benchmark<M: ModelTypes>(&self) -> BenchmarkResult {
        let start = std::time::Instant::now();

        let source =
            std::fs::read_to_string(format!("tiny-pmc-benchmarking/files/{}", self.path)).unwrap();
        let constants = tiny_pmc::parsing::parse_const_assignments(&self.constants).unwrap();

        let parsed_model_and_objectives = tiny_pmc::parsing::parse_prism_and_print_errors(
            Some(&self.path),
            &source,
            &self.property_vector(),
        );
        let (mut prism_model, properties) = match parsed_model_and_objectives {
            None => panic!("Error parsing model"),
            Some((prism_model, properties, _)) => (prism_model, properties),
        };
        let parsing = start.elapsed();

        let mut atomic_propositions = Vec::new();
        let properties = tiny_pmc::building::prism_objectives_to_atomic_propositions(
            &mut atomic_propositions,
            properties,
        );
        let builder_output: ModelBuildingOutput<M> = prism_model_builder::build_model(
            &mut prism_model,
            &atomic_propositions[..],
            properties.clone().into_iter(),
            &constants,
        )
        .unwrap();

        for property in properties {
            // TODO
        }

        let total = start.elapsed();
        let building = total - parsing;
        let actual_state_count = builder_output.model.states.len();
        if actual_state_count != self.reference_state_count {
            println!(
                "State counts don't match. Got {} states, while the reference implementation got {} states",
                actual_state_count, self.reference_state_count
            );
        }
        BenchmarkResult {
            parsing,
            total,
            building,
            state_count: actual_state_count,
        }
    }
}

pub struct BenchmarkProperty {
    property: String,
    reference_result: f64,
    reference_check_time: Option<Duration>,
}

pub fn benchmarks() -> Vec<Benchmark> {
    vec![
        Benchmark::new(
            "consensus(N=4,K=2)",
            "mdp/consensus/consensus.4.prism",
            "N=4;K=2",
            22656,
            75232,
            0.032,
        )
        .with_property("P>=1 [ F \"finished\" ]", 1.0, None)
        .with_property(
            "Pmin=? [ F \"finished\"&\"all_coins_equal_1\" ]",
            0.3173828125,
            None,
        )
        .with_property(
            "Pmax=? [F (\"finished\" & !(\"agree\"))]",
            0.2944318543,
            None,
        )
        .with_property("R[exp]{\"steps\"}max=? [F \"finished\"] ", 363.0, None)
        .with_property("R[exp]{\"steps\"}min=? [F \"finished\"]", 192.0, None),
    ]
}
