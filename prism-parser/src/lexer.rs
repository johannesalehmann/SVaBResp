use crate::PrismParserError;
use chumsky::prelude::*;
use std::fmt::{Display, Formatter};

pub type Span = SimpleSpan<usize>;
pub type Spanned<T> = (T, Span);

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Dtmc,
    Ctmc,
    Mdp,
    Pta,
    Pomdp,
    Popta,

    Module,
    EndModule,
    Const,
    Global,
    Label,
    Formula,
    Init,
    EndInit,
    Rewards,
    EndRewards,

    Int,
    Double,
    Bool,

    P,
    PMax,
    PMin,

    Finally,
    Generally,

    Identifier(String),

    // Syntax elements:
    LeftSqBracket,
    RightSqBracket,
    LeftBracket,
    RightBracket,
    Arrow,
    AssignedTo,
    Colon,
    DotDot,
    Semicolon,
    Quote,
    Comma,

    // Expressions:
    Integer(String),
    Float(String),

    True,
    False,

    Minus,
    Multiply,
    Divide,
    Modulo,
    Plus,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Negation,
    And,
    Or,
    IfAndOnlyIf,
    Implies,
    Questionmark,
}

impl Token {
    pub fn is_identifier(&self) -> bool {
        match self {
            Token::Identifier(_) => true,
            _ => false,
        }
    }
    pub fn get_identifier(&self) -> Option<&str> {
        match self {
            Token::Identifier(name) => Some(name),
            _ => None,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Dtmc => write!(f, "dtmc"),
            Token::Ctmc => write!(f, "ctmc"),
            Token::Mdp => write!(f, "mdp"),
            Token::Pta => write!(f, "pta"),
            Token::Pomdp => write!(f, "pomdp"),
            Token::Popta => write!(f, "popta"),
            Token::Module => write!(f, "module"),
            Token::EndModule => write!(f, "endmodule"),
            Token::Const => write!(f, "const"),
            Token::Global => write!(f, "global"),
            Token::Label => write!(f, "label"),
            Token::Formula => write!(f, "formula"),
            Token::Init => write!(f, "init"),
            Token::EndInit => write!(f, "endinit"),
            Token::Rewards => write!(f, "rewards"),
            Token::EndRewards => write!(f, "endrewards"),
            Token::Int => write!(f, "int"),
            Token::Double => write!(f, "double"),
            Token::Bool => write!(f, "bool"),
            Token::PMax => write!(f, "PMax"),
            Token::PMin => write!(f, "PMin"),
            Token::P => write!(f, "P"),
            Token::Finally => write!(f, "F"),
            Token::Generally => write!(f, "G"),
            Token::Identifier(_) => write!(f, "[Identifier]"),
            Token::LeftSqBracket => write!(f, "["),
            Token::RightSqBracket => write!(f, "]"),
            Token::LeftBracket => write!(f, "("),
            Token::RightBracket => write!(f, ")"),
            Token::Arrow => write!(f, "->"),
            Token::AssignedTo => write!(f, "'="),
            Token::Colon => write!(f, ":"),
            Token::DotDot => write!(f, ".."),
            Token::Semicolon => write!(f, ";"),
            Token::Quote => write!(f, "\""),
            Token::Comma => write!(f, ","),
            Token::Integer(_) => write!(f, "[Integer]"),
            Token::Float(_) => write!(f, "[Float]"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Minus => write!(f, "-"),
            Token::Multiply => write!(f, "*"),
            Token::Divide => write!(f, "/"),
            Token::Modulo => write!(f, "%"),
            Token::Plus => write!(f, "+"),
            Token::LessThan => write!(f, "<"),
            Token::LessOrEqual => write!(f, "<="),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterOrEqual => write!(f, ">="),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "!="),
            Token::Negation => write!(f, "!"),
            Token::And => write!(f, "&"),
            Token::Or => write!(f, "|"),
            Token::IfAndOnlyIf => write!(f, "<=>"),
            Token::Implies => write!(f, "=>"),
            Token::Questionmark => write!(f, "?"),
        }
    }
}

fn lexer<'a>()
-> impl Parser<'a, &'a str, Vec<Spanned<Token>>, extra::Err<PrismParserError<'a, Span, char>>> {
    let float = text::int(10)
        .then(just('.').then(text::digits(10)))
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Float);

    let int = text::int(10)
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Integer);

    let operator = just('-')
        .map(|_| Token::Minus)
        .or(just('*').map(|_| Token::Multiply))
        .or(just('/').map(|_| Token::Divide))
        .or(just('%').map(|_| Token::Modulo))
        .or(just('+').map(|_| Token::Plus))
        .or(just("=>").map(|_| Token::Implies))
        .or(just("<=>").map(|_| Token::IfAndOnlyIf))
        .or(just("<=").map(|_| Token::LessOrEqual))
        .or(just('<').map(|_| Token::LessThan))
        .or(just(">=").map(|_| Token::GreaterOrEqual))
        .or(just('>').map(|_| Token::GreaterThan))
        .or(just('=').map(|_| Token::Equal))
        .or(just("!=").map(|_| Token::NotEqual))
        .or(just('!').map(|_| Token::Negation))
        .or(just('&').map(|_| Token::And))
        .or(just('|').map(|_| Token::Or))
        .or(just('?').map(|_| Token::Questionmark))
        .or(just(',').map(|_| Token::Comma));

    let syntax_element = just('[')
        .map(|_| Token::LeftSqBracket)
        .or(just(']').map(|_| Token::RightSqBracket))
        .or(just('(').map(|_| Token::LeftBracket))
        .or(just(')').map(|_| Token::RightBracket))
        .or(just("->").map(|_| Token::Arrow))
        .or(just('\'')
            .then(chumsky::text::whitespace())
            .then(just('='))
            .map(|_| Token::AssignedTo))
        .or(just(':').map(|_| Token::Colon))
        .or(just("..").map(|_| Token::DotDot))
        .or(just(';').map(|_| Token::Semicolon))
        .or(just('\"').map(|_| Token::Quote));

    let identifier = text::ident().map(|ident: &str| match ident {
        "dtmc" => Token::Dtmc,
        "ctmc" => Token::Ctmc,
        "mdp" => Token::Mdp,
        "pta" => Token::Pta,
        "pomdp" => Token::Pomdp,
        "popta" => Token::Popta,

        "module" => Token::Module,
        "endmodule" => Token::EndModule,
        "const" => Token::Const,
        "global" => Token::Global,
        "label" => Token::Label,
        "formula" => Token::Formula,
        "init" => Token::Init,
        "endinit" => Token::EndInit,
        "rewards" => Token::Rewards,
        "endrewards" => Token::EndRewards,

        "int" => Token::Int,
        "double" => Token::Double,
        "bool" => Token::Bool,

        "PMax" => Token::PMax,
        "PMin" => Token::PMin,
        "P" => Token::P,

        "F" => Token::Finally,
        "G" => Token::Generally,

        "true" => Token::True,
        "false" => Token::False,

        _ => Token::Identifier(ident.to_string()),
    });

    let token = float.or(int).or(syntax_element).or(operator).or(identifier);

    let comment = just("//")
        .then(any().and_is(just('\n').not()).repeated())
        .padded();

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

pub fn raw_lex(
    program: &str,
) -> ParseResult<Vec<Spanned<Token>>, PrismParserError<'_, Span, char>> {
    lexer().parse(program)
}
