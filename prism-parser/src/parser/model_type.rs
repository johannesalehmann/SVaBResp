use super::E;
use crate::{PrismParserValidationError, Span, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;

pub fn model_type_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::ModelType<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Dtmc)
        .map_with(|_, e| prism_model::ModelType::Dtmc(e.span()))
        .or(just(Token::Ctmc).map_with(|_, e| prism_model::ModelType::Ctmc(e.span())))
        .or(just(Token::Mdp).map_with(|_, e| prism_model::ModelType::Mdp(e.span())))
        .or(just(Token::Pta).try_map(|_, span: Span| {
            Err(PrismParserValidationError::UnsupportedModelType {
                model_type: "pta",
                span,
            }
            .into())
        }))
        .or(just(Token::Pomdp).try_map(|_, span: Span| {
            Err(PrismParserValidationError::UnsupportedModelType {
                model_type: "pomdp",
                span,
            }
            .into())
        }))
        .or(just(Token::Popta).try_map(|_, span: Span| {
            Err(PrismParserValidationError::UnsupportedModelType {
                model_type: "popta",
                span,
            }
            .into())
        }))
        .labelled("model type")
}
