use crate::parser::{E, expression_parser, identifier_parser};
use crate::{Span, Token};
use chumsky::IterParser;
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier, RewardsTarget};

pub fn rewards_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::Rewards<Identifier<Span>, Expression<Identifier<Span>, Span>, Span>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let rewards_name_parser = just(Token::Quote)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Quote))
        .labelled("reward name")
        .as_context();

    just(Token::Rewards)
        .ignore_then(rewards_name_parser.or_not())
        .then(rewards_element_parser().repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::EndRewards))
        .map_with(|(name, entries), e| prism_model::Rewards::with_entries(name, entries, e.span()))
        .labelled("rewards structure")
        .as_context()
}
fn rewards_element_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    prism_model::RewardsElement<Identifier<Span>, Expression<Identifier<Span>, Span>, Span>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::LeftSqBracket)
        .ignore_then(identifier_parser().or_not())
        .then_ignore(just(Token::RightSqBracket))
        .or_not()
        .map(|a| match a {
            Some(a) => RewardsTarget::Action(a),
            None => RewardsTarget::State,
        })
        .then(expression_parser())
        .then_ignore(just(Token::Colon))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((action, guard), value), e| {
            prism_model::RewardsElement::with_target(guard, value, action, e.span())
        })
        .labelled("rewards structure entry")
        .as_context()
}
