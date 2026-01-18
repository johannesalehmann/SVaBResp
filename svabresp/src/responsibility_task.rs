use crate::shapley::ShapleyAlgorithm;
use crate::state_based::grouping::GroupExtractionScheme;
use crate::state_based::refinement::GroupBlockingProvider;
use crate::{PrismModel, PrismProperty};

pub struct ResponsibilityTask<
    M: ModelAndPropertySource,
    C: CounterexampleSource,
    A: ShapleyAlgorithm,
    G: GroupExtractionScheme,
    R: GroupBlockingProvider,
> {
    pub model_description: M,
    pub constants: String,
    pub coop_game_type: CoopGameType<C>,
    pub algorithm: A,
    pub grouping_scheme: G,
    pub refinement: R,
}

impl<
    M: ModelAndPropertySource,
    C: CounterexampleSource,
    A: ShapleyAlgorithm,
    G: GroupExtractionScheme,
    R: GroupBlockingProvider,
> ResponsibilityTask<M, C, A, G, R>
{
    pub fn run(mut self) -> A::Output<String> {
        let (prism_model, property) = self.model_description.get_model_and_property();
        let constants = tiny_pmc::parsing::parse_const_assignments(&self.constants)
            .expect("Failed to parse constants");

        let responsibility = crate::state_based::compute_for_prism(
            prism_model,
            property,
            self.grouping_scheme,
            self.refinement,
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
    pub fn new<S1: Into<String>, S2: Into<String>>(path: S1, property: S2) -> Self {
        Self {
            path: path.into(),
            property: property.into(),
        }
    }
}

impl ModelAndPropertySource for ModelFromFile {
    fn get_model_and_property(self) -> (super::PrismModel, super::PrismProperty) {
        let file = std::fs::read_to_string(&self.path).expect("Failed to read input model");

        let model_from_string = ModelFromString {
            name: self.path,
            model: file,
            property: self.property,
        };

        model_from_string.get_model_and_property()
    }
}

pub struct ModelFromString {
    name: String,
    model: String,
    property: String,
}

impl ModelFromString {
    pub fn new<S1: Into<String>, S2: Into<String>, S3: Into<String>>(
        name: S1,
        model: S2,
        property: S3,
    ) -> Self {
        Self {
            name: name.into(),
            model: model.into(),
            property: property.into(),
        }
    }
}

impl ModelAndPropertySource for ModelFromString {
    fn get_model_and_property(self) -> (PrismModel, PrismProperty) {
        let (model, properties) = tiny_pmc::parsing::parse_prism_and_print_errors(
            Some(self.name.as_str()),
            self.model.as_str(),
            &[self.property.as_str()],
        )
        .expect("Failed to parse prism model or property");

        assert_eq!(properties.len(), 1);
        let property = properties.into_iter().nth(0).unwrap();

        (model, property)
    }
}

pub enum CoopGameType<C: CounterexampleSource> {
    Forward,
    Backward {
        counterexample: C,
        kind: BackwardResponsibilityKind,
    },
}

pub trait CounterexampleSource {}

pub struct CounterexampleFile {
    #[allow(unused)]
    file_name: String,
}

impl CounterexampleSource for CounterexampleFile {}

pub enum BackwardResponsibilityKind {
    Optimistic,
    Pessimistic,
}
