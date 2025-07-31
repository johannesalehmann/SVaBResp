mod error;
mod lexer;
mod parser;

use chumsky::prelude::*;
pub use error::{PrismParserError, PrismParserValidationError};
pub use lexer::{Span, Token};

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

            for error in parse_errors {
                errors.push(error.map_token(|t| format!("{}", t)).into_owned())
            }
            let output = output.map(|(o, _)| o);

            ParseResult { output, errors }
        } else {
            ParseResult {
                output: None,
                errors,
            }
        }
    }
}
