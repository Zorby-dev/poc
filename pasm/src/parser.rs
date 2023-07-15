use std::{vec::IntoIter, rc::Rc};

use crate::{lexer::{Token, TokenKind}, source::{Span, WithSpan, IntoWithSpan}, message::{Result, Message}};

#[derive(Debug)]
pub enum NodeKind {
    Instruction { name: WithSpan<String>, arguments: Vec<Token> },
    Value { value: Token },
    Label { name: WithSpan<String> }
}

pub type Node = WithSpan<NodeKind>;

pub struct Parser {
    tokens: IntoIter<Token>,
    current: Option<Token>
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut out = Self {
            tokens: tokens.into_iter(),
            current: None
        };

        out.advance();
        out
    }

    fn advance(&mut self) -> &Option<Token> {
        self.current = self.tokens.next();
        &self.current
    }

    fn make_instruction(&mut self, name: WithSpan<String>) -> Result<Node> {
        let source = Rc::clone(&name.span.source);
        let begin = name.span.begin;
        let mut end = name.span.end;

        let mut arguments = Vec::new();

        while let Some(current) = self.current.clone() {
            match current.value {
                | TokenKind::Word(..)
                | TokenKind::Character(..)
                | TokenKind::String(..)
                | TokenKind::Number(..) => {
                    arguments.push(current.clone());
                    end = current.span.end;
                }
                TokenKind::NewLine => break,
                | TokenKind::Comma
                | TokenKind::Colon
                | TokenKind::Backslash => {
                    let span = current.span.clone();
                    self.advance();

                    return Err(
                        vec![Message::error(format!("expected argument, found '{}'", current.span.get_text()))
                            .with_code(String::from("invalid syntax"), span)
                    ])
                }
                | TokenKind::Percent { .. } => unreachable!()
            }

            self.advance();

            match self.current {
                Some(WithSpan { value: TokenKind::Comma, .. }) => { self.advance(); },
                _ => break
            }
        }

        Ok((
            NodeKind::Instruction { name, arguments }.with_span(Span::new(begin, end, source)),
            vec![]
        ))
    }

    fn make_instruction_or_label(&mut self, name: WithSpan<String>) -> Result<Node> {
        let source = Rc::clone(&name.span.source);
        let begin = name.span.begin;

        match self.advance() {
            Some(WithSpan { value: TokenKind::Colon, span }) => {
                let end = span.end;

                self.advance();

                Ok((
                    NodeKind::Label { name }.with_span(Span::new(begin, end, source)),
                    vec![]
                ))
            },
            _ => self.make_instruction(name)
        }
    }

    fn chop_whitespace(&mut self) {
        while let Some(WithSpan { value: TokenKind::NewLine, .. }) = self.current {
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Node>> {
        let mut errors = vec![];
        let mut warnings = vec![];
        let mut items = vec![];

        for item in self {
            match item {
                Ok((item, mut w)) => {
                    items.push(item);
                    warnings.append(&mut w);
                },
                Err(mut e) => errors.append(&mut e)
            }
        }

        if errors.is_empty() {
            Ok((items, warnings))
        } else {
            Err(errors)
        }
    }
}

impl Iterator for Parser {
    type Item = Result<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chop_whitespace();

        let current = self.current.clone()?;
        Some(match current.value {
            TokenKind::Word(name) => self.make_instruction_or_label(WithSpan { value: name, span: current.span }),
            | TokenKind::Number(..)
            | TokenKind::String(..)
            | TokenKind::Character(..) => {
                self.advance();

                Ok((
                    NodeKind::Value { value: current.clone() }.with_span(current.span.clone()),
                    vec![]
                ))
            }
            _ => {
                let span = current.span.clone();
                self.advance();

                Err(vec![
                    Message::error(format!("expected statement, found '{}'", current.span.get_text()))
                        .with_code(String::from("invalid syntax"), span)
                ])
            }
        })
    }
}