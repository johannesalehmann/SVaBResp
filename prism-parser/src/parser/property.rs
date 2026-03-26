use super::{E, expression_parser, identifier_parser};
use crate::{Span, Token};
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::{Recursive, just};
use prism_model::{Expression, Identifier};
use probabilistic_properties::{
    Bound, BoundOperator, NonDeterminismKind, PathFormula, Query, RewardFormula, StateFormula,
};

pub fn query_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    Query<
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
    >,
    E<'a>,
> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let mut state_formula_parser = Recursive::declare();
    let mut path_formula_parser = Recursive::declare();

    state_formula_parser.define(define_state_formula_parser(path_formula_parser.clone()));
    path_formula_parser.define(define_path_formula_parser(state_formula_parser.clone()));

    let probability_value = probability_min_max()
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Questionmark))
        .then_ignore(just(Token::LeftSqBracket))
        .then(path_formula_parser)
        .then_ignore(just(Token::RightSqBracket))
        .map(|(non_determinism, path)| Query::ProbabilityValue {
            non_determinism,
            path,
        });

    let state_formula = state_formula_parser.clone().map(|s| Query::StateFormula(s));

    let reward_bound = reward_name_and_min_max()
        .then(bound_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |(((name, non_determinism), bound), reward)| Query::RewardBound {
                non_determinism,
                name: name.map(|i| i.name), // TODO: Store span instead of name in reward
                bound,
                reward,
            },
        );

    let reward_value = reward_name_and_min_max()
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Questionmark))
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(|((name, non_determinism), reward)| Query::RewardValue {
            non_determinism,
            name: name.map(|i| i.name), // TODO: Store span instead of name in reward
            reward,
        });

    let time_bound = time_min_max()
        .then(bound_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(|((non_determinism, bound), reward)| Query::TimeBound {
            non_determinism,
            bound,
            reward,
        });

    let time_value = time_min_max()
        .then_ignore(just(Token::Equal))
        .then_ignore(just(Token::Questionmark))
        .then_ignore(just(Token::LeftSqBracket))
        .then(reward_formula(state_formula_parser.clone()))
        .then_ignore(just(Token::RightSqBracket))
        .map(|(non_determinism, reward)| Query::TimeValue {
            non_determinism,
            reward,
        });

    probability_value
        .or(state_formula)
        .or(reward_bound)
        .or(reward_value)
        .or(time_bound)
        .or(time_value)
}

pub fn probability_min_max<'a, 'b, I>()
-> impl Parser<'a, I, Option<NonDeterminismKind>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::P)
        .map(|_| None)
        .or(just(Token::PMax).map(|_| Some(NonDeterminismKind::Maximise)))
        .or(just(Token::PMin).map(|_| Some(NonDeterminismKind::Minimise)))
}
pub fn reward_name_and_min_max<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    (
        Option<prism_model::Identifier<Span>>,
        Option<NonDeterminismKind>,
    ),
    E<'a>,
> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::R)
        .ignore_then(reward_name())
        .then(
            just(Token::Max)
                .map(|_| NonDeterminismKind::Maximise)
                .or(just(Token::Min).map(|_| NonDeterminismKind::Minimise))
                .or_not(),
        )
        .map(|(n, m)| (Some(n), m))
        .or(just(Token::R).map(|_| (None, None)))
        .or(just(Token::RMax).map(|_| (None, Some(NonDeterminismKind::Maximise))))
        .or(just(Token::RMin).map(|_| (None, Some(NonDeterminismKind::Minimise))))
}

pub fn reward_name<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Identifier<Span>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::LeftCurlyBracket).ignore_then(
        just(Token::Quote)
            .ignore_then(identifier_parser())
            .then_ignore(just(Token::Quote))
            .then_ignore(just(Token::RightCurlyBracket)),
    )
}

pub fn time_min_max<'a, 'b, I>() -> impl Parser<'a, I, Option<NonDeterminismKind>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::T)
        .map(|_| None)
        .or(just(Token::TMax).map(|_| Some(NonDeterminismKind::Maximise)))
        .or(just(Token::TMin).map(|_| Some(NonDeterminismKind::Minimise)))
}
pub fn lra_min_max<'a, 'b, I>() -> impl Parser<'a, I, Option<NonDeterminismKind>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::LRA)
        .map(|_| None)
        .or(just(Token::LRAMax).map(|_| Some(NonDeterminismKind::Maximise)))
        .or(just(Token::LRAMin).map(|_| Some(NonDeterminismKind::Minimise)))
}

pub fn define_path_formula_parser<
    'a,
    'b,
    I,
    SF: Parser<
            'a,
            I,
            StateFormula<
                Expression<Identifier<Span>, Span>,
                Expression<Identifier<Span>, Span>,
                Expression<Identifier<Span>, Span>,
            >,
            E<'a>,
        > + Clone,
>(
    state_formula_parser: SF,
) -> impl Parser<
    'a,
    I,
    PathFormula<
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
    >,
    E<'a>,
> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let until = state_formula_parser
        .clone()
        .then_ignore(just(Token::Until))
        .then(state_formula_parser.clone())
        .map(|(before, after)| PathFormula::Until {
            before: Box::new(before),
            after: Box::new(after),
        });

    let eventually = just(Token::Finally)
        .ignore_then(state_formula_parser.clone())
        .map(|condition| PathFormula::Eventually {
            condition: Box::new(condition),
        });

    let bounded_until = state_formula_parser
        .clone()
        .then_ignore(just(Token::Until))
        .then(bound_parser())
        .then(state_formula_parser.clone())
        .map(|((before, bound), after)| PathFormula::BoundedUntil {
            before: Box::new(before),
            after: Box::new(after),
            bound,
        });

    let bounded_eventually = just(Token::Finally)
        .ignore_then(bound_parser())
        .then(state_formula_parser.clone())
        .map(|(bound, condition)| PathFormula::BoundedEventually {
            condition: Box::new(condition),
            bound,
        });

    let generally = just(Token::Generally)
        .ignore_then(state_formula_parser)
        .map(|condition| PathFormula::Generally {
            condition: Box::new(condition),
        });

    until
        .or(eventually)
        .or(bounded_until)
        .or(bounded_eventually)
        .or(generally)
}

pub fn define_state_formula_parser<
    'a,
    'b,
    I,
    PF: Parser<
            'a,
            I,
            PathFormula<
                Expression<Identifier<Span>, Span>,
                Expression<Identifier<Span>, Span>,
                Expression<Identifier<Span>, Span>,
            >,
            E<'a>,
        > + Clone,
>(
    path_formula_parser: PF,
) -> impl Parser<
    'a,
    I,
    StateFormula<
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
    >,
    E<'a>,
> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let expression = expression_parser().map(|e| StateFormula::Expression(e));

    let probability_bound = probability_min_max()
        .then(bound_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(path_formula_parser.clone())
        .then_ignore(just(Token::RightSqBracket))
        .map(
            |((non_determinism, bound), path)| StateFormula::ProbabilityBound {
                non_determinism,
                bound,
                path: Box::new(path),
            },
        );

    let long_run_average = lra_min_max()
        .then(bound_parser())
        .then(path_formula_parser)
        .map(
            |((non_determinism, bound), path)| StateFormula::LongRunAverage {
                non_determinism,
                bound,
                path: Box::new(path),
            },
        );

    expression.or(probability_bound).or(long_run_average)
}

pub fn bound_parser<'a, 'b, I>()
-> impl Parser<'a, I, Bound<Expression<Identifier<Span>, Span>>, E<'a>> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let operator = just(Token::LessThan)
        .map(|_| BoundOperator::LessThan)
        .or(just(Token::LessOrEqual).map(|_| BoundOperator::LessOrEqual))
        .or(just(Token::GreaterThan).map(|_| BoundOperator::GreaterThan))
        .or(just(Token::GreaterOrEqual).map(|_| BoundOperator::GreaterOrEqual));

    let value = expression_parser();

    operator
        .then(value)
        .map(|(operator, value)| Bound { operator, value })
}

pub fn reward_formula<
    'a,
    'b,
    I,
    SF: Parser<
            'a,
            I,
            StateFormula<
                Expression<Identifier<Span>, Span>,
                Expression<Identifier<Span>, Span>,
                Expression<Identifier<Span>, Span>,
            >,
            E<'a>,
        > + Clone,
>(
    state_formula_parser: SF,
) -> impl Parser<
    'a,
    I,
    RewardFormula<
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
    >,
    E<'a>,
> + Clone
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let instantaneous = just(Token::Instantaneous)
        .ignore_then(just(Token::Equal))
        .ignore_then(expression_parser())
        .map(|k| RewardFormula::Instantaneous { k });

    let cumulative = just(Token::Cumulative)
        .ignore_then(just(Token::LessOrEqual))
        .ignore_then(expression_parser())
        .map(|k| RewardFormula::Cumulative { k });

    let finally = just(Token::Finally)
        .ignore_then(state_formula_parser)
        .map(|states| RewardFormula::Finally { states });

    let long_run_average = just(Token::LRA).map(|_| RewardFormula::LongRunAverage);

    instantaneous
        .or(cumulative)
        .or(finally)
        .or(long_run_average)
}
