use super::E;
use crate::{PrismParserValidationError, Span, Token};
use chumsky::input::ValueInput;
use chumsky::{Parser, select};

pub fn identifier_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Identifier<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    (select! {
        Token::Identifier(name) = e => prism_model::Identifier::new::<String>(name.clone(), e.span())
    })
        .try_map_with(|i, e|
            i.map_err(|reason| PrismParserValidationError::InvalidIdentifierName { span: e.span(), reason }.into()))
        .labelled("identifier")
}

pub fn identifier_parser_potentially_reserved<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Identifier<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    (select! {
        Token::Identifier(name) = e => prism_model::Identifier::new_potentially_reserved::<String>(name.clone(), e.span())
    })
        .try_map_with(|i, e|
            i.map_err(|reason| PrismParserValidationError::InvalidIdentifierName { span: e.span(), reason }.into()))
        .labelled("identifier")
}
