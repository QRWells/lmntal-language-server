use chumsky::prelude::*;
use chumsky::Parser;
use core::fmt;
use std::fmt::Display;

pub type Span = std::ops::Range<usize>;
pub type Spanned<T> = (T, Span);

#[derive(Debug)]
pub struct ImCompleteSemanticToken {
    pub start: usize,
    pub length: usize,
    pub token_type: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    // Literals
    Identifier(String),
    Number(String),
    String(String),
    Operator(String),

    // Reserved operators
    /// `@@`
    AtAt,
    /// `:-`
    ColonDash,
    /// `,`
    Comma,
    /// `.`
    Dot,
    /// `|`
    Vert,
    /// `!`
    Bang,
    /// `$`
    Dollar,
    /// `=`
    Equal,

    // Brackets
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(s) | Token::Number(s) | Token::String(s) | Token::Operator(s) => {
                write!(f, "{}", s)
            }
            Token::AtAt => write!(f, "@@"),
            Token::ColonDash => write!(f, ":-"),
            Token::Dot => write!(f, "."),
            Token::Comma => write!(f, ","),
            Token::Vert => write!(f, "|"),
            Token::Bang => write!(f, "!"),
            Token::Dollar => write!(f, "$"),
            Token::Equal => write!(f, "="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
        }
    }
}

pub fn lexer() -> impl Parser<char, Vec<Spanned<Token>>, Error = Simple<char>> {
    // A parser for numbers
    let num = text::int(10)
        .chain::<char, _, _>(just('.').chain(text::digits(10)).or_not().flatten())
        .collect::<String>()
        .map(Token::Number);

    // A parser for strings
    let str_ = just('"')
        .ignore_then(filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Token::String);

    // A parser for operators
    let op = one_of("+-*/")
        .repeated()
        .at_least(1)
        .collect::<String>()
        .map(Token::Operator);

    // A parser for control characters (delimiters, semicolons, etc.)
    let single_delim = choice([
        just('(').to(Token::LeftParen),
        just(')').to(Token::RightParen),
        just('[').to(Token::LeftBracket),
        just(']').to(Token::RightBracket),
        just('{').to(Token::LeftBrace),
        just('}').to(Token::RightBrace),
        just('.').to(Token::Dot),
        just(',').to(Token::Comma),
        just('|').to(Token::Vert),
        just('!').to(Token::Bang),
        just('$').to(Token::Dollar),
        just('=').to(Token::Equal),
    ]);

    let double_delim = choice([just("@@").to(Token::AtAt), just(":-").to(Token::ColonDash)]);

    let ident = text::ident().map(|s| Token::Identifier(s));

    // A single token can be one of the above
    let token = num
        .or(str_)
        .or(op)
        .or(single_delim)
        .or(double_delim)
        .or(ident)
        .recover_with(skip_then_retry_until([]));

    let line_comment = just("//").then(take_until(just('\n'))).padded();
    let block_comment = just("/*").then(take_until(just("*/"))).padded();

    token
        .padded_by(block_comment.repeated())
        .padded_by(line_comment.repeated())
        .map_with_span(|tok, span| (tok, span))
        .padded()
        .repeated()
}
