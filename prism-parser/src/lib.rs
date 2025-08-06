mod error;
mod lexer;
mod parser;

use chumsky::prelude::*;
pub use error::{PrismParserError, PrismParserValidationError};
pub use lexer::{Span, Token};
use std::borrow::Cow;

pub struct ParseResult<'a> {
    pub output: Option<
        prism_model::Model<(), prism_model::Identifier<Span>, prism_model::Identifier<Span>, Span>,
    >,
    pub errors: Vec<PrismParserError<'a, Span, String>>,
}

pub fn parse_prism<'a, 'b>(source: &'a str) -> ParseResult<'b> {
    let mut errors = Vec::new();

    let (lexer_output, lexer_errors) = lexer::raw_lex(source).into_output_errors();
    if !lexer_errors.is_empty() {
        for error in lexer_errors {
            errors.push(error.map_token(|c| c.to_string()).into_owned())
        }
        ParseResult {
            output: None,
            errors,
        }
    } else {
        if let Some(lexer_output) = lexer_output {
            let (output, parse_errors) = parser::program_parser()
                .map_with(|ast, e| (ast, e.span()))
                .parse(
                    lexer_output
                        .as_slice()
                        .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
                )
                .into_output_errors();

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
                            "This error is often caused by using variables with reserved names"
                                .to_string(),
                        );
                    }
                } else {
                }
                errors.push(error.map_token(|t| format!("{}", t)).into_owned())
            }
            let mut output = output.map(|(o, _)| o);

            if let Some(output) = &mut output {
                if let Err(err) = output.substitute_formulas(SimpleSpan::new(0, 1)) {
                    errors.push(
                        PrismParserValidationError::CyclicFormulaDependency { cycle: err }.into(),
                    )
                }
                if let Err(error) = output.expand_renamed_models() {
                    errors.push(PrismParserValidationError::ModuleExpansionError { error }.into())
                }
            }

            ParseResult { output, errors }
        } else {
            ParseResult {
                output: None,
                errors,
            }
        }
    }
}
