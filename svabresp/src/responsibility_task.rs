use crate::shapley::ShapleyAlgorithm;
use crate::state_based::grouping::GroupExtractionScheme;

pub struct ResponsibilityTask<M: ModelAndPropertySource, C: CounterexampleSource, A: ShapleyAlgorithm, G: GroupExtractionScheme> {
    pub model_description: M,
    pub constants: String,
    pub coop_game_type: CoopGameType<C>,
    pub algorithm: A,
    pub grouping_scheme: G,
}

impl<M: ModelAndPropertySource, C: CounterexampleSource, A: ShapleyAlgorithm, G: GroupExtractionScheme> ResponsibilityTask<M, C, A, G> {
    pub fn run(mut self) -> A::Output<String> {
        let (mut prism_model, mut property) = self.model_description.get_model_and_property();
        let constants = tiny_pmc::parsing::parse_const_assignments(&self.constants).expect("Failed to parse constants");

        self.grouping_scheme.transform_prism(&mut prism_model, &mut property);

        let responsibility = crate::state_based::compute_for_prism(
            prism_model,
            property,
            self.grouping_scheme,
            &mut self.algorithm,
            constants,
        );

        responsibility
    }
}

pub trait ModelAndPropertySource {
    fn get_model_and_property(self) -> (super::PrismModel, super::PrismProperty);
}

pub struct ModelFromFile {
    path: String,
    property: String,
}

impl ModelFromFile {
    pub fn new(path: String, property: String) -> Self {
        Self {path, property}
    }
}

impl ModelAndPropertySource for ModelFromFile {
    fn get_model_and_property(self) ->  (super::PrismModel, super::PrismProperty) {
        let file = std::fs::read_to_string(&self.path).expect("Failed to read input model");

        let (model, properties) = tiny_pmc::parsing::parse_prism_and_print_errors(
            Some(self.path.as_str()),
            &file[..],
            &[self.property.as_str()],
        ).expect("Failed to parse prism model or property");

        assert_eq!(properties.len(), 1);
        let property =properties.into_iter().nth(0).unwrap();

        (model, property)
    }
}

pub enum CoopGameType<C: CounterexampleSource> {
    Forward,
    Backward {
        counterexample: C,
        kind: BackwardResponsibilityKind
    }
}

pub trait CounterexampleSource {

}

pub struct CounterexampleFile {
    file_name: String
}

impl CounterexampleSource for CounterexampleFile {

}

pub enum BackwardResponsibilityKind {
    Optimistic,
    Pessimistic
}