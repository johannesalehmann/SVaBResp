use crate::parser::{E, identifier_parser, identifier_parser_potentially_reserved};
use crate::{Span, Token};
use chumsky::IterParser;
use chumsky::input::ValueInput;
use chumsky::prelude::{just, recursive};
use chumsky::{Parser, select};
use prism_model::Identifier;

pub fn expression_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Expression<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    use prism_model::Expression;
    recursive(|expr| {
        let int = select! {Token::Integer(val) = e =>  Expression::Int(val.parse::<i64>().unwrap(), e.span())};
        let float = select! {Token::Float(val) = e => Expression::Float(val.parse::<f64>().unwrap(), e.span())};

        let true_exp = just(Token::True).map_with(|_, e| Expression::Bool(true, e.span()));
        let false_exp = just(Token::False).map_with(|_, e| Expression::Bool(false, e.span()));

        let identifier = identifier_parser().map_with(|i, e| Expression::VarOrConst(i, e.span()));

        let function = identifier_parser_potentially_reserved()
            .then_ignore(just(Token::LeftBracket))
            .then(
                expr.clone()
                    .separated_by(just(Token::Comma))
                    .collect::<Vec<_>>(),
            )
            .then_ignore(just(Token::RightBracket))
            .map_with(|(name, args), e| Expression::Function(name, args, e.span()));

        let label = just(Token::Quote).ignore_then(identifier_parser()).then_ignore(just(Token::Quote))
            .map_with(|name, e| Expression::Label(name, e.span()));

        let parenthesised = expr
            .clone()
            .delimited_by(just(Token::LeftBracket), just(Token::RightBracket));

        let atom = float
            .or(int)
            .or(true_exp)
            .or(false_exp)
            .or(parenthesised)
            .or(function)
            .or(identifier)
            .or(label);

        let unary = just(Token::Minus)
            .repeated()
            .foldr_with(atom, |_, exp, e| {
                Expression::Minus(Box::new(exp), e.span())
            });
        let unary = unary.boxed();

        let prod_div = unary.clone().foldl_with(
            just(Token::Multiply)
                .to(Expression::Multiplication as fn(_, _, _) -> _)
                .or(just(Token::Divide).to(Expression::Division as fn(_, _, _) -> _))
                .then(unary)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );

        let sum_diff = prod_div.clone().foldl_with(
            just(Token::Plus)
                .to(Expression::Addition as fn(_, _, _) -> _)
                .or(just(Token::Minus).to(Expression::Subtraction as fn(_, _, _) -> _))
                .then(prod_div)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );
        let sum_diff = sum_diff.boxed();

        let relational = sum_diff.clone().foldl_with(
            (just(Token::LessThan).to(Expression::LessThan as fn(_, _, _) -> _))
                .or(just(Token::LessOrEqual).to(Expression::LessOrEqual as fn(_, _, _) -> _))
                .or(just(Token::GreaterThan).to(Expression::GreaterThan as fn(_, _, _) -> _))
                .or(just(Token::GreaterOrEqual).to(Expression::GreaterOrEqual as fn(_, _, _) -> _))
                .then(sum_diff)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );

        let eq_neq = relational.clone().foldl_with(
            (just(Token::Equal).to(Expression::Equals as fn(_, _, _) -> _))
                .or(just(Token::NotEqual).to(Expression::NotEquals as fn(_, _, _) -> _))
                .then(relational)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );

        let eq_neq = eq_neq.boxed();

        let negation = just(Token::Negation)
            .repeated()
            .foldr_with(eq_neq, |_, exp, e| {
                Expression::Negation(Box::new(exp), e.span())
            });

        let conjunction = negation.clone().foldl_with(
            (just(Token::And).to(Expression::Conjunction as fn(_, _, _) -> _))
                .then(negation)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );

        let disjunction = conjunction.clone().foldl_with(
            (just(Token::Or).to(Expression::Disjunction as fn(_, _, _) -> _))
                .then(conjunction)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );
        let disjunction = disjunction.boxed();

        let iff = disjunction.clone().foldl_with(
            (just(Token::IfAndOnlyIf).to(Expression::IfAndOnlyIf as fn(_, _, _) -> _))
                .then(disjunction)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );

        let implication = iff.clone().foldl_with(
            (just(Token::Implies).to(Expression::Implies as fn(_, _, _) -> _))
                .then(iff)
                .repeated(),
            |lhs, (op, rhs), e| op(Box::new(lhs), Box::new(rhs), e.span()),
        );
        let implication = implication.boxed();

        let ternary = implication
            .clone()
            .then(
                just(Token::Questionmark)
                    .ignore_then(implication.clone())
                    .then_ignore(just(Token::Colon))
                    .then(implication)
                    .or_not(),
            )
            .map_with(|(lhs, rest), e| match rest {
                Some((a, b)) => {
                    Expression::Ternary(Box::new(lhs), Box::new(a), Box::new(b), e.span())
                }
                None => lhs,
            });
        let ternary = ternary.boxed();

        ternary.clone()
    })
        .labelled("expression")
        .as_context()
}
