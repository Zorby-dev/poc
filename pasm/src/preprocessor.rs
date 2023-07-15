use std::{fs, vec::IntoIter, rc::Rc, collections::HashMap};

use crate::{lexer::{Token, TokenKind, Lexer}, message::{Result, Message}, source::{WithSpan, Span, Source}};

type Tokens = Vec<Token>;
type Symbols = HashMap<String, Vec<TokenKind>>;

#[derive(Debug)]
pub struct Scope<'a> {
    pub tokens: Tokens,
    symbols: Symbols,
    parent: Option<&'a Scope<'a>>
}

impl<'a> Scope<'a> {
    pub fn new(parent: Option<&'a Self>) -> Self {
        Self {
            tokens: vec![],
            symbols: HashMap::new(),
            parent
        }
    }

    fn get_symbol(&self, symbol: &String) -> Option<&Vec<TokenKind>> {
        if let token_stream @ Some(..) = self.symbols.get(symbol) {
            return token_stream;
        }
        
        self.parent?.get_symbol(symbol)
    }

    fn extract(self) -> (Tokens, Symbols) {
        (self.tokens, self.symbols)
    }

    fn extend(&mut self, extract: (Tokens, Symbols)) {
        self.tokens.extend(extract.0);
        self.symbols.extend(extract.1);
    }
}

pub struct Preprocessor {
    tokens: IntoIter<Token>,
    current: Option<Token>
}

impl Preprocessor {
    pub fn preprocess(&mut self, scope: &mut Scope) -> Result<()> {
        let mut errors = vec![];
        let mut warnings = vec![];
        
        while let Some(token) = self.current.clone() {
            match &token.value {
                TokenKind::Percent => {
                    let percent = token;

                    match self.advance().clone() {
                        Some(WithSpan { value: TokenKind::Word(name), span }) => {
                            match name.as_str() {
                                "include" => {
                                    let path_span = match self.advance().clone() {
                                        Some(WithSpan { value: TokenKind::String(string), span }) => WithSpan { value: string, span },
                                        Some(WithSpan { span, .. }) => {
                                            errors.push(
                                                Message::error(format!("expected include path, found {}", span.get_text()))
                                                    .with_code(String::from("expected string"), span)
                                            );
                                            
                                            continue;
                                        },
                                        None => {
                                            errors.push(
                                                Message::error(String::from("'%include' must be supplied with include path"))
                                                .with_code(String::from("expected string"), span)
                                            );
                                            
                                            continue;
                                        }
                                    };

                                    self.advance();
                                    
                                    let mut include_path = percent.span.source.path.clone();
                                    include_path.pop();
                                    include_path.push(&path_span.value);

                                    let text = match fs::read_to_string(&include_path) {
                                        Ok(program) => program,
                                        Err(_) => {
                                            errors.push(
                                                Message::error(format!("no such file: {}", include_path.display()))
                                                    .with_code(String::from("invalid include path"), path_span.span.clone())
                                            );

                                            continue;
                                        }
                                    };

                                    let source = Rc::new(Source {
                                        text,
                                        path: include_path
                                    });

                                    let (tokens, mut w) = Lexer::new(&source).lex()?;
                                    warnings.append(&mut w);

                                    let mut preprocessor = Preprocessor::from(tokens);

                                    let mut child_scope = Scope::new(Some(scope));
                                    let ((), mut w) = preprocessor.preprocess(&mut child_scope)?;
                                    scope.extend(child_scope.extract());
                                    warnings.append(&mut w);

                                }
                                "define" => {
                                    let symbol = match self.advance() {
                                        Some(WithSpan { value: TokenKind::Word(symbol), .. }) => symbol.clone(),
                                        Some(WithSpan { span, .. }) => {
                                            errors.push(
                                                Message::error(format!("expected symbol, found '{}'", span.get_text()))
                                                    .with_code(String::from("expected identifier"), span.clone())
                                            );

                                            continue;
                                        },
                                        None => {
                                            errors.push(
                                                Message::error(String::from("'%define' must be supplied with symbol"))
                                                    .with_code(String::from("expected identifier"), span.clone())
                                            );

                                            continue;
                                        }
                                    };
                                    let mut definition: Vec<TokenKind> = vec![];

                                    self.advance();

                                    while let Some(current) = &self.current {
                                        match &current.value {
                                            TokenKind::Backslash => {
                                                self.advance();
                                                definition.push(self.current.as_ref().unwrap().value.clone());
                                                self.advance();
                                            }
                                            TokenKind::NewLine => break,
                                            _ => {
                                                definition.push(current.value.clone());
                                                self.advance();
                                            }
                                        }
                                    }

                                    scope.symbols.insert(symbol, definition);
                                }
                                "ifndef" => {
                                    let symbol = match self.advance() {
                                        Some(WithSpan { value: TokenKind::Word(symbol), .. }) => symbol.clone(),
                                        Some(WithSpan { span, .. }) => {
                                            errors.push(
                                                Message::error(format!("expected symbol, found '{}'", span.get_text()))
                                                    .with_code(String::from("expected identifier"), span.clone())
                                            );

                                            continue;
                                        },
                                        None => {
                                            errors.push(
                                                Message::error(String::from("'%ifndef' must be supplied with symbol"))
                                                    .with_code(String::from("expected identifier"), span.clone())
                                            );

                                            continue;
                                        }
                                    };

                                    self.advance();

                                    let mut child_scope = Scope::new(Some(scope));
                                    self.preprocess(&mut child_scope)?;

                                    let ndef = scope.get_symbol(&symbol) == None;
                                    if ndef {
                                        scope.extend(child_scope.extract());
                                    }   
                                }
                                "end" => {
                                    self.advance();

                                    return if errors.is_empty() {
                                        Ok(((), warnings))
                                    } else {
                                        Err(errors)
                                    }
                                }
                                _ => {
                                    errors.push(
                                        Message::error(format!("use of undeclared macro: '{}'", name))
                                            .with_code(String::from("unknown macro"), span.clone()),
                                    );

                                    self.advance();

                                    continue;
                                }
                            }
                        }
                        Some(WithSpan { span, .. }) => {
                            self.advance();

                            errors.push(
                                Message::error(format!("expected macro name, found '{}'", &span.get_text()))
                                    .with_code(String::from("expected identifier"), span.clone())
                            )
                        },
                        None => {
                            errors.push(
                                Message::error(format!("macro invocation must include macro name"))
                                    .with_code(
                                        String::from("expected identifier"),
                                        Span::new(percent.span.end, percent.span.end + 1, percent.span.source)
                                    )
                            )
                        }
                    }

                    match &self.current {
                        Some(WithSpan { value: TokenKind::NewLine, .. }) | None => {
                            self.advance();
                        },
                        Some(WithSpan { span, .. }) => {
                            errors.push(
                                Message::error(format!("macro invocations must end with a new line"))
                                    .with_code(
                                        String::from("expected new line"),
                                        span.clone()
                                    )
                            )
                        }
                    }
                }
                TokenKind::Word(word) => {
                    if let Some(token_stream) = scope.get_symbol(word).cloned() {
                        let tokens: Vec<Token> = token_stream.into_iter()
                            .map(|value| Token { value, span: token.span.clone() })
                            .collect();
                        let mut preprocessor = Preprocessor::from(tokens);
                        let mut child_scope = Scope::new(Some(scope));
                        let ((), mut w) = preprocessor.preprocess(&mut child_scope)?;
                        scope.extend(child_scope.extract());
                        warnings.append(&mut w);
                    } else {
                        scope.tokens.push(token.clone())
                    }

                    self.advance();
                }
                _ => {
                    scope.tokens.push(token.clone());

                    self.advance();
                }
            }
        }

        if errors.is_empty() {
            Ok(((), warnings))
        } else {
            Err(errors)
        }
    }

    fn advance(&mut self) -> &Option<Token> {
        self.current = self.tokens.next();
        &self.current
    }
}

impl From<Vec<Token>> for Preprocessor {
    fn from(value: Vec<Token>) -> Self {
        let mut out = Self {
            tokens: value.into_iter(),
            current: None
        };

        out.advance();
        out
    }
}