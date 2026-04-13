use crate::shapley::{ShapleyAlgorithm, SwitchingPairCollector};
use crate::state_based::StateBasedOutput;
use crate::state_based::grouping::{GroupExtractionScheme, VectorStateGroups};
use crate::state_based::refinement::GroupBlockingProvider;
use crate::{PrismModel, PrismProperty};
use chumsky::text::Char;
use log::trace;
use prism_parser::CharacterToLineMap;

pub struct ResponsibilityTask<
    'a,
    M: ModelAndPropertySource,
    C: CounterexampleSource,
    A: ShapleyAlgorithm,
    G: GroupExtractionScheme,
    R: GroupBlockingProvider,
    SPC: SwitchingPairCollector,
> {
    pub model_description: M,
    pub constants: String,
    pub coop_game_type: CoopGameType<C>,
    pub algorithm: A,
    pub grouping_scheme: &'a mut G,
    pub refinement: R,
    pub switching_pair_collector: &'a mut SPC,
}

impl<
    'a,
    M: ModelAndPropertySource,
    C: CounterexampleSource,
    A: ShapleyAlgorithm,
    G: GroupExtractionScheme,
    R: GroupBlockingProvider,
    SPC: SwitchingPairCollector,
> ResponsibilityTask<'a, M, C, A, G, R, SPC>
{
    pub fn run(mut self) -> StateBasedOutput<A::Output<String>, VectorStateGroups> {
        trace!("Loading model and property");
        let (prism_model, property, character_to_line_map) =
            self.model_description.get_model_and_property();
        trace!("Parsing constants");
        let constants = tiny_pmc::parsing::parse_const_assignments(&self.constants)
            .expect("Failed to parse constants");

        let responsibility = crate::state_based::compute_for_prism(
            prism_model,
            &character_to_line_map,
            property,
            self.grouping_scheme,
            self.refinement,
            &mut self.algorithm,
            constants,
            self.switching_pair_collector,
        );

        responsibility
    }
}

pub trait ModelAndPropertySource {
    fn get_model_and_property(
        self,
    ) -> (super::PrismModel, super::PrismProperty, CharacterToLineMap);
    fn get_source_code(&self) -> String;
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

    fn read_file(&self) -> String {
        std::fs::read_to_string(&self.path).expect("Failed to read input model")
    }
}

impl ModelAndPropertySource for ModelFromFile {
    fn get_model_and_property(
        self,
    ) -> (super::PrismModel, super::PrismProperty, CharacterToLineMap) {
        let file = self.read_file();

        let model_from_string = ModelFromString {
            name: self.path,
            model: file,
            property: self.property,
        };

        model_from_string.get_model_and_property()
    }

    fn get_source_code(&self) -> String {
        self.read_file()
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
    fn get_model_and_property(self) -> (PrismModel, PrismProperty, CharacterToLineMap) {
        let (model, properties, character_to_line_map) =
            tiny_pmc::parsing::parse_prism_and_print_errors(
                Some(self.name.as_str()),
                self.model.as_str(),
                &[self.property.as_str()],
            )
            .expect("Failed to parse prism model or property");

        assert_eq!(properties.len(), 1);
        let property = properties.into_iter().nth(0).unwrap();

        (model, property, character_to_line_map)
    }

    fn get_source_code(&self) -> String {
        self.model.clone()
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
