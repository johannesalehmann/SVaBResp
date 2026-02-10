use crate::parser::{E, expression_parser, identifier_parser};
use crate::{Span, Token};
use chumsky::IterParser;
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier};

pub fn command_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Command<
        Identifier<Span>,
        Expression<Identifier<Span>, Span>,
        Identifier<Span>,
        Span,
    >,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let action_parser = just(Token::LeftSqBracket)
        .ignore_then(identifier_parser().or_not())
        .then_ignore(just(Token::RightSqBracket))
        .labelled("action")
        .as_context();

    let no_updates_parser = just(Token::True).map(|_| Vec::new());

    let some_updates_parser = update_parser()
        .separated_by(just(Token::Plus))
        .collect::<Vec<_>>();

    let updates_parser = no_updates_parser
        .or(some_updates_parser)
        .labelled("updates")
        .as_context();

    action_parser
        .then(expression_parser())
        .then_ignore(just(Token::Arrow))
        .then(updates_parser)
        .then_ignore(just(Token::Semicolon))
        .map_with(|((action, guard), updates), e| {
            prism_model::Command::with_updates(action, guard, updates, e.span())
        })
        .labelled("command")
        .as_context()
}

fn update_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Update<Expression<Identifier<Span>, Span>, Identifier<Span>, Span>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    expression_parser()
        .then_ignore(just(Token::Colon))
        .or_not()
        .map_with(|exp, e| exp.unwrap_or(prism_model::Expression::Int(1, e.span())))
        .then(
            just(Token::True).map(|_| Vec::new()).or(assignment_parser()
                .separated_by(just(Token::And))
                .at_least(1)
                .collect::<Vec<_>>()),
        )
        .map_with(|(probability, assignments), e| {
            prism_model::Update::with_assignments(probability, assignments, e.span())
        })
        .labelled("update")
        .as_context()
}
fn assignment_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Assignment<Expression<Identifier<Span>, Span>, Identifier<Span>, Span>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::LeftBracket)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::AssignedTo))
        .then(expression_parser())
        .then_ignore(just(Token::RightBracket))
        .map_with(|(lhs, rhs), e| {
            let target_span = lhs.span;
            prism_model::Assignment::new(lhs, rhs, target_span, e.span())
        })
        .labelled("assignment")
        .as_context()
}
