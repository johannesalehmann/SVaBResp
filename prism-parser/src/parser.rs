pub use super::{Span, Token};
use crate::error::ElementKind;
use crate::{PrismParserError, PrismParserValidationError};
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use prism_model::{
    Expression, Identifier, ModelType, ModuleManager, RewardsTarget, VariableAddError,
    VariableInfo, VariableManager,
};
use probabilistic_properties::ProbabilityOperator;

pub type E<'a> = extra::Err<crate::PrismParserError<'a, Span, Token>>; // Rich<'a, Token, Span>

pub fn property_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    probabilistic_properties::Property<
        Expression<Identifier<Span>, Span>,
        Expression<Identifier<Span>, Span>,
    >,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    property_operator_parser()
        .then_ignore(just(Token::LeftSqBracket))
        .then(path_parser())
        .then_ignore(just(Token::RightSqBracket))
        .map(|(o, p)| probabilistic_properties::Property {
            operator: o,
            path: p,
        })
}
pub fn property_operator_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    probabilistic_properties::ProbabilityOperator<Expression<Identifier<Span>, Span>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    probability_kind_parser()
        .then(probability_constraint_parser())
        .map(
            |(probability_kind, probability_constraint)| ProbabilityOperator {
                kind: probability_kind,
                constraint: probability_constraint,
            },
        )
}
pub fn probability_kind_parser<'a, 'b, I>()
-> impl Parser<'a, I, probabilistic_properties::ProbabilityKind, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::PMax)
        .map(|_| probabilistic_properties::ProbabilityKind::PMax)
        .or(just(Token::PMin).map(|_| probabilistic_properties::ProbabilityKind::PMin))
        .or(just(Token::P).map(|_| probabilistic_properties::ProbabilityKind::P))
}

pub fn probability_constraint_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    probabilistic_properties::ProbabilityConstraint<Expression<Identifier<Span>, Span>>,
    E<'a>,
>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Equal)
        .then_ignore(just(Token::Questionmark))
        .map(|_| probabilistic_properties::ProbabilityConstraint::ValueOf)
        .or(just(Token::Equal)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::EqualTo(e)))
        .or(just(Token::GreaterThan)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::GreaterThan(e)))
        .or(just(Token::GreaterOrEqual)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::GreaterOrEqual(e)))
        .or(just(Token::LessThan)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::LessThan(e)))
        .or(just(Token::LessOrEqual)
            .ignore_then(expression_parser())
            .map(|e| probabilistic_properties::ProbabilityConstraint::LessOrEqual(e)))
}

pub fn path_parser<'a, 'b, I>()
-> impl Parser<'a, I, probabilistic_properties::Path<Expression<Identifier<Span>, Span>>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Finally)
        .ignore_then(expression_parser())
        .map(probabilistic_properties::Path::Eventually)
        .or(just(Token::Generally)
            .ignore_then(expression_parser())
            .map(probabilistic_properties::Path::Generally))
        .or(just(Token::Generally)
            .ignore_then(just(Token::Finally))
            .ignore_then(expression_parser())
            .map(probabilistic_properties::Path::InfinitelyOften))
}

pub fn program_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Model<(), Identifier<Span>, Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    program_element_parser()
        .repeated()
        .collect::<Vec<_>>()
        .validate(|els, e, emitter| build_program_from_type_and_elements(els, e.span(), emitter))
}

fn add_or_emit_variable(
    manager: &mut VariableManager<Identifier<Span>, Span>,
    variable: VariableInfo<Identifier<Span>, Span>,
    kind: crate::error::ElementKind,
    emitter: &mut chumsky::input::Emitter<PrismParserError<Span, Token>>,
) {
    let span = variable.span;
    match manager.add_variable(variable) {
        Ok(_) => {}
        Err(err) => match err {
            VariableAddError::VariableExists { reference } => emitter.emit(
                PrismParserValidationError::DuplicateElement {
                    previous_occurrence: manager.get(&reference).unwrap().span,
                    new_definition: span,
                    kind,
                }
                .into(),
            ),
        },
    }
}

fn build_program_from_type_and_elements<'a>(
    elements: Vec<ProgramElement>,
    span: SimpleSpan,
    emitter: &mut chumsky::input::Emitter<PrismParserError<Span, Token>>,
) -> prism_model::Model<(), Identifier<Span>, Identifier<Span>, Span> {
    let mut model_type = Option::None;
    let mut modules = ModuleManager::new();
    let mut renamed_modules = Vec::new();
    let mut variables = VariableManager::new();
    let mut labels = prism_model::LabelManager::new();
    let mut formulas = prism_model::FormulaManager::new();
    let mut init_constraint = None;
    let mut rewards = prism_model::RewardsManager::new();

    for element in elements {
        match element {
            ProgramElement::Module(m, m_vars) => {
                let span = m.span.clone();

                match modules.add(m) {
                    Ok(module_index) => {
                        for mut variable in m_vars {
                            variable.scope = Some(module_index);
                            add_or_emit_variable(
                                &mut variables,
                                variable,
                                ElementKind::LocalVar,
                                emitter,
                            );
                        }
                    }
                    Err(prism_model::AddModuleError::ModuleExists { index }) => emitter.emit(
                        PrismParserValidationError::DuplicateElement {
                            previous_occurrence: modules.get(index).unwrap().span,
                            new_definition: span,
                            kind: ElementKind::Module,
                        }
                        .into(),
                    ),
                }
            }
            ProgramElement::RenamedModule(m) => renamed_modules.push(m),
            ProgramElement::Const(c) => {
                add_or_emit_variable(&mut variables, c, ElementKind::Const, emitter);
            }
            ProgramElement::GlobalVariable(v) => {
                add_or_emit_variable(&mut variables, v, ElementKind::GlobalVar, emitter);
            }
            ProgramElement::Label(l) => {
                let span = l.span;
                match labels.add_label(l) {
                    Ok(_) => {}
                    Err(prism_model::AddLabelError::LabelExists { index }) => {
                        let previous = labels.get(index).unwrap();
                        emitter.emit(
                            PrismParserValidationError::DuplicateElement {
                                previous_occurrence: previous.span,
                                new_definition: span,
                                kind: ElementKind::Label,
                            }
                            .into(),
                        )
                    }
                }
            }
            ProgramElement::Formula(f) => {
                let span = f.span;
                match formulas.add_formula(f) {
                    Ok(_) => {}
                    Err(prism_model::AddFormulaError::FormulaExists { index }) => {
                        let previous = formulas.get(index).unwrap();
                        emitter.emit(
                            PrismParserValidationError::DuplicateElement {
                                previous_occurrence: previous.span,
                                new_definition: span,
                                kind: ElementKind::Formula,
                            }
                            .into(),
                        )
                    }
                }
            }
            ProgramElement::ModelType(t) => match model_type {
                None => model_type = Some(t),
                Some(first) => {
                    emitter.emit(
                        PrismParserValidationError::DuplicateModelType {
                            first_occurrence: *first.get_span(),
                            duplicate_occurrence: *t.get_span(),
                        }
                        .into(),
                    );
                }
            },
            ProgramElement::InitConstraint(i, span) => match &init_constraint {
                None => init_constraint = Some((i, span)),
                Some((first, first_span)) => {
                    emitter.emit(
                        PrismParserValidationError::DuplicateInitConstraint {
                            first_occurrence: *first_span,
                            first_occurrence_inner: *first.span(),
                            duplicate_occurrence: span,
                            duplicate_occurrence_inner: *i.span(),
                        }
                        .into(),
                    );
                }
            },
            ProgramElement::Rewards(r) => {
                let span = r.span;
                match rewards.add(r) {
                    Ok(_) => {}
                    Err(prism_model::AddRewardsError::RewardsExist { index }) => {
                        let previous = rewards.get(index).unwrap();
                        emitter.emit(
                            PrismParserValidationError::DuplicateElement {
                                previous_occurrence: previous.span,
                                new_definition: span,
                                kind: ElementKind::Reward,
                            }
                            .into(),
                        )
                    }
                }
            }
        }
    }

    let model_type = match model_type {
        None => {
            emitter.emit(PrismParserValidationError::MissingModelType.into());
            ModelType::Mdp(Span::new(0, 0))
        }
        Some(model_type) => model_type,
    };

    prism_model::Model::from_components(
        model_type,
        variables,
        formulas,
        (),
        modules,
        renamed_modules,
        init_constraint.map(|(i, _)| i),
        labels,
        rewards,
        span,
    )
}

enum ProgramElement {
    ModelType(ModelType<Span>),
    Const(prism_model::VariableInfo<Identifier<Span>, Span>),
    Label(prism_model::Label<Identifier<Span>, Span>),
    Module(
        prism_model::Module<Identifier<Span>, Identifier<Span>, Span>,
        Vec<VariableInfo<Identifier<Span>, Span>>,
    ),
    RenamedModule(prism_model::RenamedModule<Span>),
    GlobalVariable(prism_model::VariableInfo<Identifier<Span>, Span>),
    Formula(prism_model::Formula<Identifier<Span>, Span>),
    InitConstraint(prism_model::Expression<Identifier<Span>, Span>, Span),
    Rewards(prism_model::Rewards<Identifier<Span>, Identifier<Span>, Span>),
}

fn program_element_parser<'a, 'b, I>() -> impl Parser<'a, I, ProgramElement, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    model_type_parser()
        .map(ProgramElement::ModelType)
        .or(module_parser().map(|(m, v)| ProgramElement::Module(m, v)))
        .or(renamed_module_parser().map(ProgramElement::RenamedModule))
        .or(const_parser().map(ProgramElement::Const))
        .or(label_parser().map(ProgramElement::Label))
        .or(formula_parser().map(ProgramElement::Formula))
        .or(global_variable_declaration_parser().map(ProgramElement::GlobalVariable))
        .or(rewards_parser().map(ProgramElement::Rewards))
        .or(init_constraint_parser().map(|(i, e)| ProgramElement::InitConstraint(i, e)))
}

fn model_type_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::ModelType<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Dtmc)
        .map_with(|_, e| prism_model::ModelType::Dtmc(e.span()))
        .or(just(Token::Ctmc).map_with(|_, e| prism_model::ModelType::Ctmc(e.span())))
        .or(just(Token::Mdp).map_with(|_, e| prism_model::ModelType::Mdp(e.span())))
        .or(just(Token::Pta).try_map(|_, span: Span| {
            Err(PrismParserValidationError::UnsupportedModelType {
                model_type: "pta",
                span,
            }
            .into())
        }))
        .or(just(Token::Pomdp).try_map(|_, span: Span| {
            Err(PrismParserValidationError::UnsupportedModelType {
                model_type: "pomdp",
                span,
            }
            .into())
        }))
        .or(just(Token::Popta).try_map(|_, span: Span| {
            Err(PrismParserValidationError::UnsupportedModelType {
                model_type: "popta",
                span,
            }
            .into())
        }))
        .labelled("model type")
}

fn const_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableInfo<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Const)
        .ignore_then(variable_domain_parser().or_not().map_with(|t, e| {
            t.unwrap_or(prism_model::VariableRange::UnboundedInt { span: e.span() })
        }))
        .then(identifier_parser())
        .then(just(Token::Equal).ignore_then(expression_parser()).or_not())
        .then_ignore(just(Token::Semicolon))
        .try_map_with(|((const_type, name), value), e| {
            if !const_type.is_legal_for_constant() {
                Err(PrismParserValidationError::IllegalConstType {
                    span: e.span(),
                    illegal_type: const_type,
                }
                .into())
            } else {
                Ok(prism_model::VariableInfo::with_optional_initial_value(
                    name,
                    const_type,
                    true,
                    None,
                    value,
                    e.span(),
                ))
            }
        })
        .labelled("constant")
        .as_context()
}
fn variable_domain_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableRange<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let range_parser = just(Token::LeftSqBracket)
        .ignore_then(
            expression_parser()
                .then_ignore(just(Token::DotDot))
                .then(expression_parser()),
        )
        .then_ignore(just(Token::RightSqBracket))
        .map_with(|(min, max), e| prism_model::VariableRange::BoundedInt {
            min,
            max,
            span: e.span(),
        });

    range_parser
        .or(just(Token::Int)
            .map_with(|_, e| prism_model::VariableRange::UnboundedInt { span: e.span() }))
        .or(just(Token::Bool)
            .map_with(|_, e| prism_model::VariableRange::Boolean { span: e.span() }))
        .or(just(Token::Double)
            .map_with(|_, e| prism_model::VariableRange::Float { span: e.span() }))
        .labelled("variable domain ([n..m], int, bool or double)")
        .as_context()
}

fn label_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Label<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Label)
        .ignore_then(identifier_parser().delimited_by(just(Token::Quote), just(Token::Quote)))
        .then_ignore(just(Token::Equal))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|(name, expression), e| prism_model::Label::new(name, expression, e.span()))
        .labelled("label")
        .as_context()
}

fn formula_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Formula<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Formula)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Equal))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|(name, expression), e| prism_model::Formula::new(name, expression, e.span()))
        .labelled("formula")
        .as_context()
}

fn module_parser<'a, 'b, I>() -> impl Parser<
    'a,
    I,
    (
        prism_model::Module<Identifier<Span>, Identifier<Span>, Span>,
        Vec<VariableInfo<Identifier<Span>, Span>>,
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
    prism_model::Module<Identifier<Span>, Identifier<Span>, Span>,
    Vec<VariableInfo<Identifier<Span>, Span>>,
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

fn renamed_module_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::RenamedModule<Span>, E<'a>>
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
                .then(
                    just(Token::Comma)
                        .ignore_then(rename_rule_parser())
                        .repeated()
                        .collect::<Vec<_>>(),
                )
                .or_not()
                .map(|rr| match rr {
                    Some((head, tail)) => vec_from_head_and_tail(head, tail),
                    None => Vec::new(),
                }),
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

fn identifier_parser<'a, 'b, I>() -> impl Parser<'a, I, prism_model::Identifier<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    (select! {
        Token::Identifier(name) = e => prism_model::Identifier::new::<String>(name.clone(), e.span())
    })
        .try_map_with(|i, e|
            i.map_err(|reason| PrismParserValidationError::InvalidIdentifierName { span: e.span(), reason }.into()))
        .labelled("identifier")
}

fn identifier_parser_potentially_reserved<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Identifier<Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    (select! {
        Token::Identifier(name) = e => prism_model::Identifier::new_potentially_reserved::<String>(name.clone(), e.span())
    })
        .try_map_with(|i, e|
            i.map_err(|reason| PrismParserValidationError::InvalidIdentifierName { span: e.span(), reason }.into()))
        .labelled("identifier")
}

enum ModuleElement {
    Command(prism_model::Command<Identifier<Span>, Identifier<Span>, Span>),
    Variable(prism_model::VariableInfo<Identifier<Span>, Span>),
}

fn module_element_parser<'a, 'b, I>() -> impl Parser<'a, I, ModuleElement, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    command_parser()
        .map(|c| ModuleElement::Command(c))
        .or(variable_declaration_parser().map(|v| ModuleElement::Variable(v)))
}

fn global_variable_declaration_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableInfo<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let init_parser = just(Token::Init).ignore_then(expression_parser());
    just(Token::Global)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Colon))
        .then(variable_domain_parser())
        .then(init_parser.or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((name, domain), init), e| {
            prism_model::VariableInfo::with_optional_initial_value(
                name,
                domain,
                false,
                None,
                init,
                e.span(),
            )
        })
        .labelled("global variable declaration")
        .as_context()
}

fn variable_declaration_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::VariableInfo<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let init_parser = just(Token::Init).ignore_then(expression_parser());
    identifier_parser()
        .then_ignore(just(Token::Colon))
        .then(variable_domain_parser())
        .then(init_parser.or_not())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((name, domain), init), e| {
            VariableInfo::with_optional_initial_value(name, domain, false, None, init, e.span())
            // Module must be changed from None to Some(...) later on
        })
        .labelled("variable declaration")
        .as_context()
}

fn rewards_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Rewards<Identifier<Span>, Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let rewards_name_parser = just(Token::Quote)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::Quote))
        .labelled("reward name")
        .as_context();

    just(Token::Rewards)
        .ignore_then(rewards_name_parser.or_not())
        .then(rewards_element_parser().repeated().collect::<Vec<_>>())
        .then_ignore(just(Token::EndRewards))
        .map_with(|(name, entries), e| prism_model::Rewards::with_entries(name, entries, e.span()))
        .labelled("rewards structure")
        .as_context()
}
fn rewards_element_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::RewardsElement<Identifier<Span>, Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::LeftSqBracket)
        .ignore_then(identifier_parser().or_not())
        .then_ignore(just(Token::RightSqBracket))
        .or_not()
        .map(|a| match a {
            Some(a) => RewardsTarget::Action(a),
            None => RewardsTarget::State,
        })
        .then(expression_parser())
        .then_ignore(just(Token::Colon))
        .then(expression_parser())
        .then_ignore(just(Token::Semicolon))
        .map_with(|((action, guard), value), e| {
            prism_model::RewardsElement::with_target(guard, value, action, e.span())
        })
        .labelled("rewards structure entry")
        .as_context()
}

fn init_constraint_parser<'a, 'b, I>()
-> impl Parser<'a, I, (prism_model::Expression<Identifier<Span>, Span>, Span), E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::Init)
        .ignore_then(expression_parser())
        .then_ignore(just(Token::EndInit))
        .map_with(|i, e| (i, e.span()))
        .labelled("init constraint")
        .as_context()
}

fn command_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Command<Identifier<Span>, Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    let action_parser = just(Token::LeftSqBracket)
        .ignore_then(identifier_parser().or_not())
        .then_ignore(just(Token::RightSqBracket))
        .labelled("action")
        .as_context();

    let no_updates_parser = just(Token::True).map(|_| Vec::new());

    let some_updates_parser = update_parser()
        .then(
            just(Token::Plus)
                .ignore_then(update_parser())
                .repeated()
                .collect(),
        )
        .map(|(head, tail)| vec_from_head_and_tail(head, tail));

    let updates_parser = no_updates_parser
        .or(some_updates_parser)
        .labelled("updates")
        .as_context();

    action_parser
        .then(expression_parser())
        .then_ignore(just(Token::Arrow))
        .then(updates_parser)
        .then_ignore(just(Token::Semicolon))
        .map_with(|((action, guard), updates), e| {
            prism_model::Command::with_updates(action, guard, updates, e.span())
        })
        .labelled("command")
        .as_context()
}

fn update_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Update<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    expression_parser()
        .then_ignore(just(Token::Colon))
        .or_not()
        .map_with(|exp, e| exp.unwrap_or(prism_model::Expression::Int(1, e.span())))
        .then(
            just(Token::True).map(|_| Vec::new()).or(assignment_parser()
                .separated_by(just(Token::And))
                .at_least(1)
                .collect::<Vec<_>>()),
        )
        .map_with(|(probability, assignments), e| {
            prism_model::Update::with_assignments(probability, assignments, e.span())
        })
        .labelled("update")
        .as_context()
}

fn vec_from_head_and_tail<T>(head: T, mut tail: Vec<T>) -> Vec<T> {
    let mut new_vec = vec![head];
    new_vec.append(&mut tail);
    new_vec
}

fn assignment_parser<'a, 'b, I>()
-> impl Parser<'a, I, prism_model::Assignment<Identifier<Span>, Span>, E<'a>>
where
    I: ValueInput<'a, Token = Token, Span = Span>,
{
    just(Token::LeftBracket)
        .ignore_then(identifier_parser())
        .then_ignore(just(Token::AssignedTo))
        .then(expression_parser())
        .then_ignore(just(Token::RightBracket))
        .map_with(|(lhs, rhs), e| {
            let target_span = lhs.span;
            prism_model::Assignment::new(lhs, rhs, target_span, e.span())
        })
        .labelled("assignment")
        .as_context()
}

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
