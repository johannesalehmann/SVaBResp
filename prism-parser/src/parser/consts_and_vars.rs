use super::{E, expression_parser, identifier_parser};
use crate::{PrismParserValidationError, Span, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier, VariableInfo};

pub fn const_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableInfo<Expression<Identifier<Span>, Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Const)
        .ignore_then(variable_domain_parser().or_not().map_with(|t, e| {
            t.unwrap_or(prism_model::VariableRange::UnboundedInt { span: e.span() })
        }))
        .then(identifier_parser())
        .then(just(Token::Equal).ignore_then(expression_parser()).or_not())
        .then_ignore(just(Token::Semicolon))
        .try_map_with(|((const_type, name), value), e| {
            if !const_type.is_legal_for_constant() {
                Err(PrismParserValidationError::IllegalConstType {
                    span: e.span(),
                    illegal_type: const_type,
                }
                .into())
            } else {
                Ok(prism_model::VariableInfo::with_optional_initial_value(
                    name,
                    const_type,
                    true,
                    None,
                    value,
                    e.span(),
                ))
            }
        })
        .labelled("constant")
        .as_context()
}
pub fn global_variable_declaration_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableInfo<Expression<Identifier<Span>, Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let init_parser = just(Token::Init).ignore_then(expression_parser());
    just(Token::Global)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Colon))
        .then(variable_domain_parser())
        .then(init_parser.or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((name, domain), init), e| {
            prism_model::VariableInfo::with_optional_initial_value(
                name,
                domain,
                false,
                None,
                init,
                e.span(),
            )
        })
        .labelled("global variable declaration")
        .as_context()
}

pub fn variable_declaration_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableInfo<Expression<Identifier<Span>, Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let init_parser = just(Token::Init).ignore_then(expression_parser());
    identifier_parser()
        .then_ignore(just(Token::Colon))
        .then(variable_domain_parser())
        .then(init_parser.or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((name, domain), init), e| {
            VariableInfo::with_optional_initial_value(name, domain, false, None, init, e.span())
            // Module must be changed from None to Some(...) later on
        })
        .labelled("variable declaration")
        .as_context()
}

fn variable_domain_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableRange<Expression<Identifier<Span>, Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let range_parser = just(Token::LeftSqBracket)
        .ignore_then(
            expression_parser()
                .then_ignore(just(Token::DotDot))
                .then(expression_parser()),
        )
        .then_ignore(just(Token::RightSqBracket))
        .map_with(|(min, max), e| prism_model::VariableRange::BoundedInt {
            min,
            max,
            span: e.span(),
        });

    range_parser
        .or(just(Token::Int)
            .map_with(|_, e| prism_model::VariableRange::UnboundedInt { span: e.span() }))
        .or(just(Token::Bool)
            .map_with(|_, e| prism_model::VariableRange::Boolean { span: e.span() }))
        .or(just(Token::Double)
            .map_with(|_, e| prism_model::VariableRange::Float { span: e.span() }))
        .labelled("variable domain ([n..m], int, bool or double)")
        .as_context()
}
