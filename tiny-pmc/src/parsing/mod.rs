use crate::PrismModel;
use chumsky::span::SimpleSpan;
use prism_parser::PrismParserError;

pub enum ErrorSource {
    Model,
    Property(usize),
}

pub fn parse_model_from_source<'a>(
    source: &str,
    properties: &[&str],
) -> Result<
    (PrismModel, Vec<crate::Property>),
    Vec<(ErrorSource, PrismParserError<'a, SimpleSpan, String>)>,
> {
    let parse_results = prism_parser::parse_prism(source, properties);
    let mut attributed_errors = Vec::new();

    for error in parse_results.model.errors {
        attributed_errors.push((ErrorSource::Model, error));
    }
    let mut properties = Vec::new();
    for (i, property) in parse_results.properties.into_iter().enumerate() {
        for error in property.errors {
            attributed_errors.push((ErrorSource::Property(i), error));
        }
        if let Some(property) = property.output {
            properties.push(property)
        }
    }
    if attributed_errors.is_empty() {
        if let Some(model) = parse_results.model.output {
            Ok((model, properties))
        } else {
            Err(vec![])
        }
    } else {
        Err(attributed_errors)
    }
}
