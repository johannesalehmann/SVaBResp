use chumsky::error::{Error, LabelError, RichPattern};
use chumsky::input::Input;
use chumsky::util::MaybeRef;
use prism_model::Identifier;

#[derive(Debug, PartialEq)]
pub enum PrismParserError<'a, S, T> {
    ExpectedFound {
        span: S,
        expected: Vec<RichPattern<'a, T>>,
        found: Option<MaybeRef<'a, T>>,
        contexts: Vec<(RichPattern<'a, T>, S)>,
    },
    Validation(PrismParserValidationError<S>),
}

#[derive(Debug, PartialEq)]
pub enum PrismParserValidationError<S> {
    UnsupportedModelType {
        model_type: &'static str,
        span: S,
    },
    MissingModelType,
    DuplicateModelType {
        first_occurrence: S,
        duplicate_occurrence: S,
    },
    DuplicateInitConstraint {
        first_occurrence: S,
        first_occurrence_inner: S,
        duplicate_occurrence: S,
        duplicate_occurrence_inner: S,
    },
    IllegalConstType {
        illegal_type: prism_model::VariableRange<Identifier<S>, S>,
        span: S,
    },
    DuplicateElement {
        previous_occurrence: S,
        new_definition: S,
        kind: ElementKind,
    },
    InvalidIdentifierName {
        span: S,
        reason: prism_model::InvalidName,
    },
}

#[derive(Debug, PartialEq)]
pub enum ElementKind {
    GlobalVar,
    LocalVar,
    Const,
    Label,
    Formula,
    Reward,
}

impl<'a, S, T> Into<PrismParserError<'a, S, T>> for PrismParserValidationError<S> {
    fn into(self) -> PrismParserError<'a, S, T> {
        PrismParserError::Validation(self)
    }
}

impl<'a, S, T> PrismParserError<'a, S, T> {
    pub fn into_owned<'b>(self) -> PrismParserError<'b, S, T>
    where
        T: Clone,
    {
        match self {
            Self::ExpectedFound {
                found,
                expected,
                span,
                contexts,
            } => PrismParserError::ExpectedFound {
                expected: expected.into_iter().map(RichPattern::into_owned).collect(),
                found: found.map(MaybeRef::into_owned),
                span,
                contexts: contexts
                    .into_iter()
                    .map(|(p, s)| (p.into_owned(), s))
                    .collect(),
            },
            Self::Validation(validation) => PrismParserError::Validation(validation),
        }
    }

    pub fn map_token<U, F: FnMut(T) -> U>(self, mut f: F) -> PrismParserError<'a, S, U>
    where
        T: Clone,
    {
        match self {
            Self::ExpectedFound {
                expected,
                found,
                span,
                contexts,
            } => PrismParserError::ExpectedFound {
                expected: expected
                    .into_iter()
                    .map(|pat| pat.map_token(&mut f))
                    .collect(),
                span,
                found: found.map(|found| f(found.into_inner()).into()),
                contexts: contexts
                    .into_iter()
                    .map(|(pat, s)| (pat.map_token(&mut f), s))
                    .collect(),
            },
            Self::Validation(validation) => PrismParserError::Validation(validation),
        }
    }
}

impl<'a, I: Input<'a>> Error<'a, I> for PrismParserError<'a, I::Span, I::Token>
where
    I::Token: PartialEq + Clone,
{
    fn merge(mut self, mut other: Self) -> Self {
        if let (
            Self::ExpectedFound { expected, .. },
            Self::ExpectedFound {
                expected: expected_other,
                ..
            },
        ) = (&mut self, &mut other)
        {
            expected.append(expected_other);
        }
        self
    }
}

impl<'a, I: Input<'a>, L> LabelError<'a, I, L> for PrismParserError<'a, I::Span, I::Token>
where
    I::Token: PartialEq + Clone,
    L: Into<RichPattern<'a, I::Token>>,
{
    fn expected_found<Iter: IntoIterator<Item = L>>(
        expected: Iter,
        found: Option<MaybeRef<'a, I::Token>>,
        span: I::Span,
    ) -> Self {
        Self::ExpectedFound {
            span,
            expected: expected.into_iter().map(|e| e.into()).collect(),
            found,
            contexts: Vec::new(),
        }
    }

    fn label_with(&mut self, label: L) {
        match &mut *self {
            Self::ExpectedFound { expected, .. } => {
                expected.clear();
                expected.push(label.into());
            }
            _ => (),
        }
    }

    fn in_context(&mut self, label: L, span: I::Span) {
        if let Self::ExpectedFound { contexts, .. } = self {
            let label = label.into();
            contexts.push((label, span))
        }
    }
}
