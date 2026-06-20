use crate::span::Span;
use logos::{Lexer as LogosLexer, Logos, SpannedIter};
use std::iter::Peekable;

#[derive(Clone)]
pub(crate) struct Lexer<'a> {
    inner: Peekable<SpannedIter<'a, RawToken<'a>>>,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(src: &'a str) -> Self {
        Self {
            inner: LogosLexer::new(src).spanned().peekable(),
        }
    }

    fn next_token(&mut self) -> Option<(Token<'a>, Span)> {
        loop {
            let (res, span) = self.inner.next()?;
            let span = span.into();

            let (tok, span) = match res {
                Ok(RawToken::Whitespace) => continue,
                Ok(RawToken::Comment) => (Token::Comment, span),
                Ok(RawToken::DocComment) => (Token::DocComment, span),
                Ok(RawToken::InlineDocComment) => (Token::InlineDocComment, span),
                Ok(RawToken::Punct(tok)) => (Token::Punct(tok), span),
                Ok(RawToken::Ident(tok)) => self.verify_token(Token::Ident(tok), span),
                Ok(RawToken::LitString) => (Token::LitString, span),
                Ok(RawToken::LitStringError) => (Token::LitStringError, span),
                Ok(RawToken::LitInt) => self.verify_token(Token::LitInt, span),
                Ok(RawToken::LitUuid) => self.verify_token(Token::LitUuid, span),
                Err(()) => self.consume_errors(span),
            };

            break Some((tok, span));
        }
    }

    fn verify_token(&mut self, tok: Token<'a>, span: Span) -> (Token<'a>, Span) {
        match self.inner.peek() {
            Some((Ok(RawToken::Whitespace), _))
            | Some((Ok(RawToken::Comment), _))
            | Some((Ok(RawToken::DocComment), _))
            | Some((Ok(RawToken::InlineDocComment), _))
            | Some((Ok(RawToken::Punct(_)), _))
            | Some((Ok(RawToken::LitString), _))
            | Some((Ok(RawToken::LitStringError), _))
            | None => (tok, span),

            Some((Ok(RawToken::Ident(_)), _))
            | Some((Ok(RawToken::LitInt), _))
            | Some((Ok(RawToken::LitUuid), _))
            | Some((Err(()), _)) => self.consume_errors(span),
        }
    }

    fn consume_errors(&mut self, mut span: Span) -> (Token<'a>, Span) {
        loop {
            match self.inner.peek() {
                Some((Ok(RawToken::Whitespace), _))
                | Some((Ok(RawToken::Comment), _))
                | Some((Ok(RawToken::DocComment), _))
                | Some((Ok(RawToken::InlineDocComment), _))
                | Some((Ok(RawToken::Punct(_)), _))
                | None => break (Token::Error, span),

                Some((Ok(RawToken::Ident(_)), next_span))
                | Some((Ok(RawToken::LitString), next_span))
                | Some((Ok(RawToken::LitStringError), next_span))
                | Some((Ok(RawToken::LitInt), next_span))
                | Some((Ok(RawToken::LitUuid), next_span))
                | Some((Err(()), next_span)) => {
                    debug_assert_eq!(span.end, next_span.start);
                    span.end = next_span.end;
                    self.inner.next();
                }
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = (Token<'a>, Span);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[derive(Logos, Copy, Clone)]
#[logos(subpattern hex = "[0-9a-fA-F]")]
enum RawToken<'a> {
    #[regex(r"[\p{White_Space}]+")]
    Whitespace,

    #[regex("//.*", allow_greedy = true)]
    Comment,

    #[regex("///.*", allow_greedy = true)]
    DocComment,

    #[regex("//!.*", allow_greedy = true)]
    InlineDocComment,

    #[regex(r"[\p{XID_Start}_]\p{XID_Continue}*")]
    Ident(&'a str),

    #[regex(r#""(?:\\.|[^\\\n"])*""#)]
    LitString,

    #[regex(r#""(?:\\.|[^\\\n"])*\\?"#)]
    LitStringError,

    #[regex("-?[0-9]+")]
    LitInt,

    #[regex("(?&hex){8}-(?&hex){4}-(?&hex){4}-(?&hex){4}-(?&hex){12}")]
    LitUuid,

    #[token("!")]
    #[token("#")]
    #[token("(")]
    #[token(")")]
    #[token(",")]
    #[token("->")]
    #[token("::")]
    #[token(";")]
    #[token("<")]
    #[token("=")]
    #[token(">")]
    #[token("@")]
    #[token("[")]
    #[token("]")]
    #[token("{")]
    #[token("}")]
    Punct(&'a str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum Token<'a> {
    Comment,
    DocComment,
    InlineDocComment,
    Ident(&'a str),
    LitString,
    LitStringError,
    LitInt,
    LitUuid,
    Punct(&'a str),
    Error,
}
