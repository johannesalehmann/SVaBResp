mod error;
mod lexer;
mod parser;

use chumsky::prelude::*;
pub use error::{PrismParserError, PrismParserValidationError};
pub use lexer::{Span, Token};
use std::borrow::Cow;

pub struct ParseResult<'a, O> {
    pub output: Option<O>,
    pub errors: Vec<PrismParserError<'a, Span, String>>,
}

pub struct ParseResults<'a, 'b> {
    pub model: ParseResult<
        'a,
        prism_model::Model<
            (),
            prism_model::Identifier<Span>,
            prism_model::Expression<prism_model::VariableReference, Span>,
            prism_model::VariableReference,
            Span,
        >,
    >,
    pub properties: Vec<
        ParseResult<
            'b,
            probabilistic_properties::Property<
                prism_model::Expression<prism_model::VariableReference, Span>,
                prism_model::Expression<prism_model::VariableReference, Span>,
            >,
        >,
    >,
}

pub fn parse_prism<'a, 'b>(source: &'a str, properties: &[&'a str]) -> ParseResults<'b, 'b> {
    let mut model_errors = Vec::new();
    let mut property_errors = (0..properties.len())
        .map(|_| Vec::new())
        .collect::<Vec<_>>();

    if let Some(lexer_output) = lex(source, &mut model_errors) {
        let (output, parse_errors) = parser::program_parser()
            .map_with(|ast, e| (ast, e.span()))
            .parse(
                lexer_output
                    .as_slice()
                    .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
            )
            .into_output_errors();
        process_parser_errors(&mut model_errors, parse_errors);
        let output = output.map(|(o, _)| o);

        let lexed_properties = properties
            .iter()
            .zip(property_errors.iter_mut())
            .map(|(p, errs)| lex(p, errs))
            .collect::<Vec<_>>();
        let mut parsed_properties = lexed_properties
            .into_iter()
            .zip(properties)
            .zip(property_errors.iter_mut())
            .map(|((lexer_output, source), errs)| {
                lexer_output.map_or(None, |lexer_output| {
                    let (output, parse_errors) = parser::property_parser()
                        .map_with(|ast, e| (ast, e.span()))
                        .parse(
                            lexer_output
                                .as_slice()
                                .map((source.len()..source.len()).into(), |(t, s)| (t, s)),
                        )
                        .into_output_errors();

                    process_parser_errors(errs, parse_errors);
                    output.map(|(o, _)| o)
                })
            })
            .collect::<Vec<_>>();

        let (output, properties) = match output {
            Some(mut output) => {
                parsed_properties
                    .iter_mut()
                    .zip(property_errors.iter_mut())
                    .for_each(|(p_option, errs)| {
                        if let Some(p) = p_option {
                            use prism_model::SubstitutableProperty;
                            p.substitute_labels(SimpleSpan::new(0, 1), &output.labels);
                            let substitution =
                                p.substitute_formulas(SimpleSpan::new(0, 1), &output.formulas);
                            if let Err(err) = substitution {
                                errs.push(
                                    PrismParserValidationError::CyclicFormulaDependency {
                                        cycle: err,
                                    }
                                    .into(),
                                );
                                *p_option = None
                            }
                        }
                    });

                if let Err(err) = output.substitute_formulas(SimpleSpan::new(0, 1)) {
                    model_errors.push(
                        PrismParserValidationError::CyclicFormulaDependency { cycle: err }.into(),
                    )
                }

                if let Err(error) = output.expand_renamed_models() {
                    model_errors
                        .push(PrismParserValidationError::ModuleExpansionError { error }.into());
                    (None, vec![None; properties.len()])
                } else {
                    let properties = parsed_properties
                        .into_iter()
                        .zip(property_errors.iter_mut())
                        .map(|(p, errs)| {
                            p.map_or(None, |p| {
                                use prism_model::SubstitutableProperty;
                                match p.replace_identifiers_by_variable_indices(
                                    &output.variable_manager,
                                ) {
                                    Ok(p) => Some(p),
                                    Err(e) => {
                                        for err in e {
                                            errs.push(
                                                PrismParserValidationError::UnknownVariable {
                                                    identifier: err.identifier,
                                                }
                                                .into(),
                                            );
                                        }
                                        None
                                    }
                                }
                            })
                        })
                        .collect::<Vec<_>>();

                    (
                        match output.replace_identifiers_by_variable_indices() {
                            Ok(output) => Some(output),
                            Err(errs) => {
                                for err in errs {
                                    model_errors.push(
                                        PrismParserValidationError::UnknownVariable {
                                            identifier: err.identifier,
                                        }
                                        .into(),
                                    )
                                }
                                None
                            }
                        },
                        properties,
                    )
                }
            }
            None => (None, vec![None; properties.len()]),
        };

        let properties = properties
            .into_iter()
            .zip(property_errors.into_iter())
            .map(|(p, e)| ParseResult {
                output: p,
                errors: e,
            })
            .collect::<Vec<_>>();

        ParseResults {
            model: ParseResult {
                output,
                errors: model_errors,
            },
            properties: properties,
        }
    } else {
        let properties = property_errors
            .into_iter()
            .map(|e| ParseResult {
                output: None,
                errors: e,
            })
            .collect::<Vec<_>>();

        ParseResults {
            model: ParseResult {
                output: None,
                errors: model_errors,
            },
            properties: properties,
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
