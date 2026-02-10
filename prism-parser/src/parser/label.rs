use super::{E, expression_parser, identifier_parser};
use crate::{Span, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier};

pub fn label_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Label<Expression<Identifier<Span>, Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Label)
        .ignore_then(identifier_parser().delimited_by(just(Token::Quote), just(Token::Quote)))
        .then_ignore(just(Token::Equal))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|(name, expression), e| prism_model::Label::new(name, expression, e.span()))
        .labelled("label")
        .as_context()
}
