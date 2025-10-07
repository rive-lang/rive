//! Token definitions for the Rive lexer.

use logos::Logos;
use std::fmt;

/// Represents a token in the Rive language.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

/// Represents the different kinds of tokens in Rive.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
pub enum TokenKind {
    // Keywords
    #[token("let")]
    Let,

    #[token("const")]
    Const,

    #[token("mut")]
    Mut,

    #[token("fun")]
    Fun,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("return")]
    Return,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("loop")]
    Loop,

    #[token("when")]
    When,

    #[token("in")]
    In,

    #[token("with")]
    With,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("null")]
    Null,

    #[token("print")]
    Print,

    #[token("type")]
    Type,

    #[token("interface")]
    Interface,

    #[token("impl")]
    Impl,

    #[token("extend")]
    Extend,

    #[token("static")]
    Static,

    // Identifiers and literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"-?[0-9]+")]
    Integer,

    #[regex(r"-?[0-9]+\.[0-9]+")]
    Float,

    #[regex(r#""([^"\\]|\\.)*""#)]
    String,

    // Operators
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("=")]
    Equal,

    #[token("==")]
    EqualEqual,

    #[token("!=")]
    BangEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("&&")]
    AmpersandAmpersand,

    #[token("||")]
    PipePipe,

    #[token("!")]
    Bang,

    // Range operators (order matters: ..= before ..)
    #[token("..=")]
    DotDotEq,

    #[token("..")]
    DotDot,

    // Punctuation
    #[token("_", priority = 10)]
    Underscore,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("{")]
    LeftBrace,

    #[token("}")]
    RightBrace,

    #[token("[")]
    LeftBracket,

    #[token("]")]
    RightBracket,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(";")]
    Semicolon,

    #[token("?")]
    Question,

    #[token(".")]
    Dot,

    #[token("->")]
    Arrow,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let => write!(f, "let"),
            Self::Const => write!(f, "const"),
            Self::Mut => write!(f, "mut"),
            Self::Fun => write!(f, "fun"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::While => write!(f, "while"),
            Self::For => write!(f, "for"),
            Self::Return => write!(f, "return"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Loop => write!(f, "loop"),
            Self::When => write!(f, "when"),
            Self::In => write!(f, "in"),
            Self::With => write!(f, "with"),
            Self::True => write!(f, "true"),
            Self::False => write!(f, "false"),
            Self::Null => write!(f, "null"),
            Self::Print => write!(f, "print"),
            Self::Type => write!(f, "type"),
            Self::Interface => write!(f, "interface"),
            Self::Impl => write!(f, "impl"),
            Self::Extend => write!(f, "extend"),
            Self::Static => write!(f, "static"),

            Self::Identifier => write!(f, "identifier"),
            Self::Integer => write!(f, "integer"),
            Self::Float => write!(f, "float"),
            Self::String => write!(f, "string"),

            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Star => write!(f, "*"),
            Self::Slash => write!(f, "/"),
            Self::Percent => write!(f, "%"),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => write!(f, "=="),
            Self::BangEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::LessEqual => write!(f, "<="),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::AmpersandAmpersand => write!(f, "&&"),
            Self::PipePipe => write!(f, "||"),
            Self::Bang => write!(f, "!"),

            Self::DotDotEq => write!(f, "..="),
            Self::DotDot => write!(f, ".."),

            Self::Underscore => write!(f, "_"),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::Semicolon => write!(f, ";"),
            Self::Question => write!(f, "?"),
            Self::Dot => write!(f, "."),
            Self::Arrow => write!(f, "->"),
        }
    }
}
