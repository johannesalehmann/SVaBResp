use ariadne::ReportBuilder;
use ariadne::{Label, Report, ReportKind, Source};
use chumsky::error::RichPattern;
use chumsky::prelude::SimpleSpan;
use chumsky::util::MaybeRef;
use prism_model::{Identifier, InvalidName, Model, ModuleExpansionError};
use prism_parser::{PrismParserError, PrismParserValidationError, Span};
use std::ops::Range;

pub fn parse_prism(
    file_name: Option<&str>,
    source: &str,
) -> Option<Model<(), Identifier<SimpleSpan>, Identifier<SimpleSpan>, SimpleSpan>> {
    let parse_result = prism_parser::parse_prism(source);

    let has_errors = !parse_result.errors.is_empty();
    for error in parse_result.errors {
        print_error(file_name, source, error);
    }
    if has_errors {
        None
    } else {
        parse_result.output
    }
}

fn print_error(file_name: Option<&str>, source: &str, error: PrismParserError<Span, String>) {
    let file_name = file_name.unwrap_or("input");

    // let mut cg = ColorGenerator::new();
    // let context_color = cg.next();

    let builder = match error {
        PrismParserError::ExpectedFound {
            span,
            expected,
            found,
            contexts,
            help,
        } => build_expected_found(file_name, span, &expected, found, &contexts, &help),

        PrismParserError::Validation(validation) => build_validation(file_name, validation),
    };
    builder
        .finish()
        .print((file_name, Source::from(source)))
        .unwrap();
}

fn build_expected_found<'a>(
    file_name: &'a str,
    span: Span,
    expected: &Vec<RichPattern<String>>,
    found: Option<MaybeRef<String>>,
    contexts: &Vec<(RichPattern<String>, Span)>,
    help: &Option<String>,
) -> ReportBuilder<'a, (&'a str, Range<usize>)> {
    let mut builder = Report::build(ReportKind::Error, (file_name, span.into_range()));
    builder.set_message(format!(
        "Unexpected character{}",
        context_message(&contexts)
    ));
    builder
        .add_label(Label::new((file_name, span.into_range())).with_message(found_message(found)));

    if !expected.is_empty() {
        builder.add_note(expected_message(&expected));
    }

    if let Some((_, location)) = contexts.first() {
        builder.add_label(Label::new((file_name, location.into_range())));
    }

    if let Some(help) = help {
        builder.add_help(help);
    }

    builder
}

fn expected_message(expected: &Vec<RichPattern<String>>) -> String {
    let mut message = Vec::new();
    message.push("Expected ".to_string());
    message.push(pattern_to_string(&expected[0]));
    if expected.len() > 1 {
        for pat in &expected[1..expected.len() - 1] {
            message.push(", ".to_string());
            message.push(pattern_to_string(&pat));
        }
        message.push(" or ".to_string());
        message.push(pattern_to_string(&expected[expected.len() - 1]));
    }
    message.push(".".to_string());
    let message = message.join("");
    message
}

fn found_message(found: Option<MaybeRef<String>>) -> String {
    match found {
        None => "Found end of file".to_string(),
        Some(found) => {
            format!("Found `{}`", found.to_string())
        }
    }
}

fn context_message(contexts: &Vec<(RichPattern<String>, Span)>) -> String {
    let context_message = contexts
        .iter()
        .map(|(pattern, _)| format!("\"{:?}\"", pattern))
        .collect::<Vec<_>>()
        .join(" in ");
    let context = if contexts.len() > 0 {
        format!(" while parsing {}", context_message)
    } else {
        "".to_string()
    };
    context
}

fn pattern_to_string(pattern: &RichPattern<String>) -> String {
    match pattern {
        RichPattern::Token(tok) => format!("`{}`", tok.to_string()),
        RichPattern::Label(l) => format!("{l}"),
        RichPattern::Identifier(i) => format!("'{i}'"),
        RichPattern::Any => "any".to_string(),
        RichPattern::SomethingElse => "something else".to_string(),
        RichPattern::EndOfInput => "end of input".to_string(),
    }
}

fn build_validation(
    file_name: &str,
    error: PrismParserValidationError<Span>,
) -> ReportBuilder<(&str, Range<usize>)> {
    match error {
        PrismParserValidationError::UnsupportedModelType { model_type, span } => {
            let mut builder = Report::build(ReportKind::Error, (file_name, span.into_range()));
            builder.set_message("Unsupported model type");
            builder.add_label(
                Label::new((file_name, span.into_range()))
                    .with_message(format!("Model type {} is not supported", model_type)),
            );
            builder.add_help("Supported model types are `dtmc`, `mdp` and `ctmc`.");
            builder
        }
        PrismParserValidationError::MissingModelType => {
            let mut builder = Report::build(ReportKind::Error, (file_name, 0..1));
            builder.set_message("Missing model type");
            builder.add_label(Label::new((file_name, 0..1)));
            builder.add_help("Add a line with `dtmc`, `mdp` or `ctmc` to your model.");
            builder
        }
        PrismParserValidationError::DuplicateModelType {
            first_occurrence,
            duplicate_occurrence,
        } => {
            let mut builder = Report::build(
                ReportKind::Error,
                (file_name, duplicate_occurrence.into_range()),
            );
            builder.set_message("Duplicate model type");
            builder.add_label(
                Label::new((file_name, first_occurrence.into_range()))
                    .with_message("Model type is first set here"),
            );
            builder.add_label(
                Label::new((file_name, duplicate_occurrence.into_range()))
                    .with_message("Model type is defined again here"),
            );
            builder
        }
        PrismParserValidationError::DuplicateInitConstraint {
            first_occurrence,
            first_occurrence_inner,
            duplicate_occurrence,
            duplicate_occurrence_inner,
        } => {
            let mut builder = Report::build(
                ReportKind::Error,
                (file_name, duplicate_occurrence.into_range()),
            );
            builder.set_message("Duplicate init constraint");
            builder.add_label(
                Label::new((file_name, first_occurrence.into_range()))
                    .with_message("The init constraint is first set here"),
            );
            builder.add_label(Label::new((file_name, first_occurrence_inner.into_range())));
            builder.add_label(
                Label::new((file_name, duplicate_occurrence.into_range()))
                    .with_message("The duplicate init construct is set here"),
            );
            builder.add_label(Label::new((
                file_name,
                duplicate_occurrence_inner.into_range(),
            )));
            builder
        }
        PrismParserValidationError::IllegalConstType { illegal_type, span } => {
            let mut builder = Report::build(ReportKind::Error, (file_name, span.into_range()));
            builder.set_message("Illegal const type");
            builder.add_label(Label::new((file_name, span.into_range())));
            builder.add_label(
                Label::new((file_name, illegal_type.span().into_range())).with_message(format!(
                    "Type `{}` is not legal for constants",
                    illegal_type.get_name()
                )),
            );
            builder.add_help("The types `int`, `bool` and `double` are legal types for constants.");
            builder
        }
        PrismParserValidationError::DuplicateElement {
            previous_occurrence,
            new_definition,
            ..
        } => {
            let mut builder =
                Report::build(ReportKind::Error, (file_name, new_definition.into_range()));
            builder.set_message("Duplicate name");
            builder.add_label(
                Label::new((file_name, previous_occurrence.into_range()))
                    .with_message("First defined here"),
            );
            builder.add_label(
                Label::new((file_name, new_definition.into_range()))
                    .with_message("Defined again here"),
            );

            builder
        }
        PrismParserValidationError::InvalidIdentifierName { span, reason } => {
            let mut builder = Report::build(ReportKind::Error, (file_name, span.into_range()));
            builder.set_message("Invalid name");
            let (message, span, help) = match reason {
                InvalidName::Empty => ("Identifier must not be empty", span.into_range(), None),
                InvalidName::StartsWithNumber { .. } => (
                    "Identifier must not start with number",
                    span.start..span.start + 1,
                    None,
                ),
                InvalidName::InvalidCharacter { location, .. } => (
                    "Invalid character",
                    span.start + location..span.start + location + 1,
                    Some("Valid characters are `A`..`Z`, `a`..`z`, `0`..`9` and `_`."),
                ),
                InvalidName::Reserved { .. } => ("Is a reserved keyword", span.into_range(), None),
            };
            builder.add_label(Label::new((file_name, span)).with_message(message));
            if let Some(help) = help {
                builder.add_help(help);
            }
            builder
        }

        PrismParserValidationError::CyclicFormulaDependency { cycle } => {
            let primary = cycle.entries[0].formula_span.into_range();
            let mut builder = Report::build(ReportKind::Error, (file_name, primary));
            builder.set_message("Cyclic dependency between formulas");

            for i in 0..cycle.entries.len() {
                let depends_on_index = match i {
                    0 => cycle.entries.len() - 1,
                    i => i - 1,
                };
                let depends_on = cycle.entries[depends_on_index].formula_name.name.clone();
                let entry = &cycle.entries[i];
                builder.add_label(
                    Label::new((file_name, entry.dependency_span.into_range())).with_message(
                        format!("{} depends on {} here", entry.formula_name.name, depends_on),
                    ),
                );
                builder.add_label(Label::new((file_name, entry.formula_span.into_range())));
            }

            builder
        }

        PrismParserValidationError::ModuleExpansionError {
            error:
                ModuleExpansionError::DuplicateModule {
                    name,
                    original_module,
                    renaming_rule,
                },
        } => {
            let mut builder =
                Report::build(ReportKind::Error, (file_name, renaming_rule.into_range()));
            builder.set_message("Duplicate module during renaming");
            builder.add_label(
                Label::new((file_name, renaming_rule.into_range())).with_message(format!(
                    "This renaming rule creates a module with name {}",
                    name
                )),
            );
            builder.add_label(
                Label::new((file_name, original_module.into_range()))
                    .with_message(format!("A module with name {} is first defined here", name)),
            );

            builder
        }
        PrismParserValidationError::ModuleExpansionError {
            error:
                ModuleExpansionError::MissingVariableRenaming {
                    variable_name,
                    original_definition,
                    renaming_rule,
                },
        } => {
            let mut builder =
                Report::build(ReportKind::Error, (file_name, renaming_rule.into_range()));
            builder.set_message("Missing variable renaming during module renaming");

            builder.add_label(
                Label::new((file_name, renaming_rule.into_range())).with_message(format!(
                    "This renaming rule does not rename variable {}",
                    variable_name
                )),
            );
            builder.add_label(
                Label::new((file_name, original_definition.into_range()))
                    .with_message(format!("{} is defined here", variable_name)),
            );

            builder.add_note(
                "When renaming a module, a new name must be given for every variable of the module.",
            );
            builder
        }
        PrismParserValidationError::ModuleExpansionError {
            error:
                ModuleExpansionError::RenamingSourceDoesNotExist {
                    old_name,
                    new_name,
                    renaming_rule,
                },
        } => {
            let mut builder =
                Report::build(ReportKind::Error, (file_name, renaming_rule.into_range()));
            builder.set_message("Renamed module does not exist");

            builder.add_label(
                Label::new((file_name, old_name.span.into_range()))
                    .with_message(format!("Cannot find module with name {}", old_name.name)),
            );

            builder
        }
    }
}
