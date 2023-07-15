use std::{str::Chars, rc::Rc, num::IntErrorKind};

use crate::{source::{Span, WithSpan, IntoWithSpan, Source}, message::{Message, Result}};

trait IsValidWord {
    fn is_valid_word(&self) -> bool;
    fn is_valid_word_begin(&self) -> bool;
}

impl IsValidWord for char {
    fn is_valid_word(&self) -> bool {
        self.is_alphanumeric() || *self == '_'
    }

    fn is_valid_word_begin(&self) -> bool {
        self.is_alphabetic() || *self == '_'
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Word(String),
    Number(u8),
    Character(u8),
    String(String),
    Comma,
    Colon,
    Percent,
    NewLine,
    Backslash
}

pub type Token = WithSpan<TokenKind>;

pub struct Lexer<'a> {
    text: Chars<'a>,
    index: usize,
    current: Option<char>,
    source: Rc<Source>
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a Rc<Source>) -> Self {
        let mut out = Self {
            text: source.text.chars(),
            index: 0,
            current: None,
            source: Rc::clone(&source)
        };
        out.advance();
        out
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(_) = self.current {
            self.index += 1;
        }
        self.current = self.text.next();
        self.current
    }

    fn make_word(&mut self) -> Result<Token> {
        let begin = self.index;
        let mut text = String::new();

        while let Some(current) = self.current {
            if !current.is_valid_word() {
                break
            }

            text.push(current);

            self.advance();
        }

        let end = self.index;

        return Ok((
            TokenKind::Word(text).with_span(Span::new(begin, end, Rc::clone(&self.source))),
            vec![]
        ))
    }

    fn make_number(&mut self) -> Result<Token> {
        let begin = self.index;
        let mut base = 10;
        let mut parsed_radix = false;
        let mut text = String::new();

        while let Some(current) = self.current {
            if !(current.is_alphanumeric() || current == '_') {
                break;
            }

            if current.is_alphabetic() && text == "0" && !parsed_radix {
                base = match current {
                    'b'       => 2,
                    'o' | 'q' => 8,
                    'd'       => 10,
                    'x' | 'h' => 16,
                    _   => return Err(vec![
                        Message::error(format!("'{}' is not a valid radix", current))
                            .with_code(
                                String::from("invalid radix"),
                                Span::new(self.index, self.index + 1, Rc::clone(&self.source))
                            )
                            .with_note(String::from("valid radixes are: 'b' and 'x'"))
                    ])
                };
                
                text = String::new();
                parsed_radix = true;
            } else if current == '_' {
                
            } else {
                text.push(current);
            }

            self.advance();
        }

        let end = self.index;
        let span = Span::new(begin, end, Rc::clone(&self.source));

        let number = u8::from_str_radix(text.as_str(), base).map_err(|error| match error.kind() {
            IntErrorKind::Empty => vec![Message::error(format!("numbers must not be empty"))
                .with_code(String::from("empty number"), span.clone())
            ],
            IntErrorKind::InvalidDigit => vec![Message::error(format!("numbers must contain valid digits"))
                .with_code(String::from("includes invalid digits"), span.clone())
            ],
            IntErrorKind::PosOverflow => vec![Message::error(format!("integer overflow: {} can't fit in a byte", &text))
                .with_code(String::from("too big"), span.clone())
            ],
            IntErrorKind::NegOverflow => vec![Message::error(format!("negative integers are not suppored yet"))
                .with_code(String::from("unsupported sign"), span.clone())
            ],
            _ => unreachable!()
        })?;

        return Ok((
            TokenKind::Number(number).with_span(span),
            vec![]
        ))
    }

    fn make_singleton(&mut self, kind: TokenKind) -> Result<Token> {
        let begin = self.index;
        self.advance();
        let end = self.index;

        Ok((
            kind.with_span(Span::new(begin, end, Rc::clone(&self.source))),
            vec![]
        ))
    }

    fn escaped_char(&mut self) -> Option<char> {
        match self.current {
            Some('\\') => {
                Some(match self.advance()? {
                    'n' => '\n',
                    character => character
                })
            },
            _ => self.current
        }
    }

    fn make_character<'b>(&'b mut self) -> Result<Token> {
        let begin = self.index;

        self.advance();
        let character = self.escaped_char().ok_or(vec![
            Message::error(String::from("expected character, found end of file"))
                .with_code(
                    String::from("expected character"),
                    Span::new(self.index, self.index + 1, Rc::clone(&self.source))
                )
        ])?;

        let character: u8 = character.try_into().map_err(|_| vec![Message::error(format!("characters must fit in a byte"))
            .with_code(
                String::from("does not fit in a byte"),
                Span::new(self.index, self.index + 1, Rc::clone(&self.source))
            )
        ])?;

        match self.advance() {
            Some('\'') => {
                self.advance();
                let end = self.index;

                Ok((
                    TokenKind::Character(character.try_into().unwrap()).with_span(Span::new(begin, end, Rc::clone(&self.source))),
                    vec![]
                ))
            }
            _ => todo!()
        }
    }

    fn make_string(&mut self) -> Result<Token> {
        let begin = self.index;
        let mut text = String::new();

        self.advance();

        while self.current != Some('"') {
            text.push(match self.escaped_char() {
                Some(character) => {
                    self.advance();
                    character
                },
                None => {
                    self.advance();

                    return Err(vec![
                        Message::error(String::from("strings must be closed"))
                            .with_code(
                                String::from("expected '\"'"),
                                Span::new(self.index, self.index + 1, Rc::clone(&self.source))
                            )
                    ])
                }
            });
        }

        self.advance();
        let end = self.index;

        Ok((TokenKind::String(text).with_span(Span::new(begin, end, Rc::clone(&self.source))), vec![]))
    }

    fn chop_whitespace(&mut self) {
        while let Some(' ') = self.current {
            self.advance();
        }
    }

    fn chop_whitespace_and_comments(&mut self) {
        self.chop_whitespace();

        if let Some(';') = self.current {
            loop {
                match self.advance() {
                    Some('\n') | None => break,
                    _ => ()
                }
            }
            self.chop_whitespace();
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>> {
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

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chop_whitespace_and_comments();

        let current = self.current?;
        Some({
            if current.is_valid_word_begin() {
                self.make_word()
            } else if current.is_numeric() {
                self.make_number()
            } else {
                match current {
                    '"'  => self.make_string(),
                    '%'  => self.make_singleton(TokenKind::Percent),
                    ','  => self.make_singleton(TokenKind::Comma),
                    ':'  => self.make_singleton(TokenKind::Colon),
                    '\n' => self.make_singleton(TokenKind::NewLine),
                    '\\' => self.make_singleton(TokenKind::Backslash),
                    '\'' => self.make_character(),
                    _    => {
                        let span = Span::new(self.index, self.index + 1, Rc::clone(&self.source));
                        self.advance();

                        Err(vec![
                            Message::error(format!("'{}' is not a valid character", current))
                                .with_code(String::from("illegal character"), span)
                        ])
                    }
                }
            }
        })
    }
}