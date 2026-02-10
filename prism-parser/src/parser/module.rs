use super::{E, command_parser, identifier_parser, variable_declaration_parser};
use crate::{PrismParserError, Span, Token};
use chumsky::IterParser;
use chumsky::Parser;
use chumsky::input::ValueInput;
use chumsky::prelude::just;
use prism_model::{Expression, Identifier, VariableInfo};

pub fn module_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    (
        prism_model::Module<
            Identifier<Span>,
            Expression<Identifier<Span>, Span>,
            Identifier<Span>,
            Span,
        >,
        Vec<VariableInfo<Expression<Identifier<Span>, Span>, Span>>,
    ),
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Module)
        .ignore_then(identifier_parser())
        .then(module_element_parser().repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::EndModule))
        .validate(|(name, els), e, emitter| {
            create_module_from_name_and_elements(name, els, e.span(), emitter)
        })
        .labelled("module")
        .as_context()
}

fn create_module_from_name_and_elements(
    name: Identifier<Span>,
    module_elements: Vec<ModuleElement>,
    span: Span,
    _emitter: &mut chumsky::input::Emitter<PrismParserError<Span, Token>>,
) -> (
    prism_model::Module<
        Identifier<Span>,
        Expression<Identifier<Span>, Span>,
        Identifier<Span>,
        Span,
    >,
    Vec<VariableInfo<Expression<Identifier<Span>, Span>, Span>>,
) {
    let mut module = prism_model::Module::new(name, span);
    let mut variables = Vec::new();

    for element in module_elements {
        match element {
            ModuleElement::Variable(v) => {
                variables.push(v);
            }
            ModuleElement::Command(c) => {
                module.commands.push(c);
            }
        }
    }

    (module, variables)
}

pub fn renamed_module_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::RenamedModule<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Module)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Equal))
        .then(identifier_parser())
        .then_ignore(just(Token::LeftSqBracket))
        .then(
            rename_rule_parser()
                .separated_by(just(Token::Comma))
                .collect::<Vec<_>>(),
        )
        .then_ignore(just(Token::RightSqBracket))
        .then_ignore(just(Token::EndModule))
        .map_with(|((new_name, old_name), rename_rules), e| {
            prism_model::RenamedModule::new(
                old_name,
                new_name,
                prism_model::RenameRules {
                    rules: rename_rules,
                },
                e.span(),
            )
        })
        .labelled("renamed module")
        .as_context()
}

fn rename_rule_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::RenameRule<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    identifier_parser()
        .then_ignore(just(Token::Equal))
        .then(identifier_parser())
        .map_with(|(old_name, new_name), e| {
            prism_model::RenameRule::new(old_name, new_name, e.span())
        })
        .labelled("rename rule")
        .as_context()
}

enum ModuleElement {
    Command(
        prism_model::Command<
            Identifier<Span>,
            Expression<Identifier<Span>, Span>,
            Identifier<Span>,
            Span,
        >,
    ),
    Variable(prism_model::VariableInfo<Expression<Identifier<Span>, Span>, Span>),
}

fn module_element_parser<'a, 'b, I>() -> impl Parser<'a, I, ModuleElement, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    command_parser()
        .map(|c| ModuleElement::Command(c))
        .or(variable_declaration_parser().map(|v| ModuleElement::Variable(v)))
}
