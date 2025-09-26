mod error;
mod lexer;
mod parser;

use chumsky::prelude::*;
use chumsky::util::IntoMaybe;
pub use error::{PrismParserError, PrismParserValidationError};
pub use lexer::{Span, Token};
use prism_model::{Identifier, VariableManager, VariableReference};
use std::borrow::Cow;

pub struct ParseResult<'a, O> {
    pub output: Option<O>,
    pub errors: Vec<PrismParserError<'a, Span, String>>,
}

pub fn parse_expression<'a, 'b, R>(
    expression: &'a str,
    variable_manager: &VariableManager<R, Span>,
) -> ParseResult<'b, prism_model::Expression<VariableReference, Span>> {
    let mut errors = Vec::new();

    if let Some(lexer_output) = lex(expression, &mut errors) {
        let (output, parse_errors) = parser::expression_parser()
            .map_with(|ast, e| (ast, e.span()))
            .parse(
                lexer_output
                    .as_slice()
                    .map((expression.len()..expression.len()).into(), |(t, s)| (t, s)),
            )
            .into_output_errors();

        process_parser_errors(&mut errors, parse_errors);
        let mut output = output.map(|(o, _)| o);

        match output {
            Some(expression) => {
                match expression.replace_identifiers_by_variable_indices(variable_manager) {
                    Ok(expr) => {
                        return ParseResult {
                            output: Some(expr),
                            errors,
                        }
                    }
                    Err(errs) => {
                        for err in errs {
                            errors.push(
                                PrismParserValidationError::UnknownVariable {
                                    identifier: err.identifier,
                                }
                                .into(),
                            )
                        }
                    }
                }
            }
            None => {}
        }
    }
    ParseResult {
        output: None,
        errors,
    }
}

pub fn parse_prism<'a, 'b>(
    source: &'a str,
) -> ParseResult<
    'b,
    prism_model::Model<(), prism_model::Identifier<Span>, prism_model::VariableReference, Span>,
> {
    let mut errors = Vec::new();

    if let Some(lexer_output) = lex(source, &mut errors) {
        let (output, parse_errors) = parser::program_parser()
            .map_with(|ast, e| (ast, e.span()))
            .parse(
                lexer_output
                    .as_slice()
                    .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
            )
            .into_output_errors();

        process_parser_errors(&mut errors, parse_errors);
        let mut output = output.map(|(o, _)| o);

        let output = match output {
            Some(mut output) => {
                if let Err(err) = output.substitute_formulas(SimpleSpan::new(0, 1)) {
                    errors.push(
                        PrismParserValidationError::CyclicFormulaDependency { cycle: err }.into(),
                    )
                }
                if let Err(error) = output.expand_renamed_models() {
                    errors.push(PrismParserValidationError::ModuleExpansionError { error }.into());
                    None
                } else {
                    match output.replace_identifiers_by_variable_indices() {
                        Ok(output) => Some(output),
                        Err(errs) => {
                            for err in errs {
                                errors.push(
                                    PrismParserValidationError::UnknownVariable {
                                        identifier: err.identifier,
                                    }
                                    .into(),
                                )
                            }
                            None
                        }
                    }
                }
            }
            None => None,
        };

        ParseResult { output, errors }
    } else {
        ParseResult {
            output: None,
            errors,
        }
    }
}

fn process_parser_errors(
    errors: &mut Vec<PrismParserError<Span, String>>,
    parse_errors: Vec<PrismParserError<Span, Token>>,
) {
    for mut error in parse_errors {
        if let PrismParserError::ExpectedFound {
            expected,
            contexts,
            help,
            ..
        } = &mut error
        {
            // If a reserved keyword is used in a declaration, an understandable error is
            // emitted, but if the same keyword is used in an expression, this instead
            // produces the error "exected (, found ...)", because the reserved keyword is
            // treated as the first part of a function declaration. To make this error less
            // confusing, the add some context here:
            if expected.len() == 1
                && expected[0]
                    == chumsky::error::RichPattern::Token(chumsky::util::Maybe::Val(
                        Token::LeftBracket,
                    ))
                && !contexts.is_empty()
                && contexts.first().unwrap().0
                    == chumsky::error::RichPattern::Label(Cow::Borrowed("expression"))
            {
                *help = Some(
                    "This error is often caused by using variables with reserved names".to_string(),
                );
            }
        } else {
        }
        errors.push(error.map_token(|t| format!("{}", t)).into_owned())
    }
}

pub fn lex(
    source: &str,
    errors: &mut Vec<PrismParserError<Span, String>>,
) -> Option<Vec<lexer::Spanned<Token>>> {
    let (lexer_output, lexer_errors) = lexer::raw_lex(source).into_output_errors();
    if !lexer_errors.is_empty() {
        for error in lexer_errors {
            errors.push(error.map_token(|c| c.to_string()).into_owned())
        }
        None
    } else {
        lexer_output
    }
}
