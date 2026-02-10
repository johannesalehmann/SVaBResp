use super::{E, expression_parser};
use crate::{Span, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier};
use probabilistic_properties::ProbabilityOperator;

pub fn property_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    probabilistic_properties::Property<
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
    >,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    property_operator_parser()
        .then_ignore(just(Token::LeftSqBracket))
        .then(path_parser())
        .then_ignore(just(Token::RightSqBracket))
        .map(|(o, p)| probabilistic_properties::Property {
            operator: o,
            path: p,
        })
}
pub fn property_operator_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    probabilistic_properties::ProbabilityOperator<Expression<Identifier<Span>, Span>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    probability_kind_parser()
        .then(probability_constraint_parser())
        .map(
            |(probability_kind, probability_constraint)| ProbabilityOperator {
                kind: probability_kind,
                constraint: probability_constraint,
            },
        )
}
pub fn probability_kind_parser<'a, 'b, I>()
-> impl Parser<'a, I, probabilistic_properties::ProbabilityKind, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::PMax)
        .map(|_| probabilistic_properties::ProbabilityKind::PMax)
        .or(just(Token::PMin).map(|_| probabilistic_properties::ProbabilityKind::PMin))
        .or(just(Token::P).map(|_| probabilistic_properties::ProbabilityKind::P))
}

pub fn probability_constraint_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    probabilistic_properties::ProbabilityConstraint<Expression<Identifier<Span>, Span>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Equal)
        .then_ignore(just(Token::Questionmark))
        .map(|_| probabilistic_properties::ProbabilityConstraint::ValueOf)
        .or(just(Token::Equal)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::EqualTo(e)))
        .or(just(Token::GreaterThan)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::GreaterThan(e)))
        .or(just(Token::GreaterOrEqual)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::GreaterOrEqual(e)))
        .or(just(Token::LessThan)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::LessThan(e)))
        .or(just(Token::LessOrEqual)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::LessOrEqual(e)))
}

pub fn path_parser<'a, 'b, I>()
-> impl Parser<'a, I, probabilistic_properties::Path<Expression<Identifier<Span>, Span>>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Finally)
        .ignore_then(expression_parser())
        .map(probabilistic_properties::Path::Eventually)
        .or(just(Token::Generally)
            .ignore_then(expression_parser())
            .map(probabilistic_properties::Path::Generally))
        .or(just(Token::Generally)
            .ignore_then(just(Token::Finally))
            .ignore_then(expression_parser())
            .map(probabilistic_properties::Path::InfinitelyOften))
}
