use std::{io::Write, collections::HashMap, ops::Deref, cell::RefCell, path::PathBuf, fs::File, rc::Rc};

use byteorder::WriteBytesExt;
use itertools::Itertools;

use crate::{parser::{Node, Parser, NodeKind}, lexer::{Lexer, Token, TokenKind}, message::{Message, Result, human_count}, signature::{self, Argument, INSTRUCTIONS}, source::{WithSpan, IntoWithSpan, Source}, preprocessor::{Preprocessor, Scope}};

struct UsedMarker<T> {
    value: T,
    used: RefCell<bool>
}

impl<T> UsedMarker<T> {
    fn new(value: T) -> Self {
        Self { value, used: false.into() }
    }
}

impl<T> From<T> for UsedMarker<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for UsedMarker<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        *self.used.borrow_mut() = true;
        &self.value
    }
}

struct Compiler<'a> {
    program: &'a Vec<Node>,
    output_path: PathBuf,
    output: &'a mut dyn Write,
    cursor: usize,
    symbols: HashMap<String, UsedMarker<WithSpan<u8>>>,
}

impl<'a> Compiler<'a> {
    fn new(program: &'a Vec<Node>, output: &'a mut dyn Write,output_path: PathBuf) -> Self {
        Self {
            program,
            output,
            output_path,
            cursor: 0,
            symbols: HashMap::new(),
        }
    }

    fn write(&mut self, byte: u8) {
        self.output.write_u8(byte).unwrap();
        self.cursor += 1;
    }

    fn immediate(&self, token: &Token) -> core::result::Result<u8, Vec<Message>> {
        match &token.value {
            TokenKind::Number(byte) => Ok(*byte),
            TokenKind::Character(byte) => Ok(*byte),
            TokenKind::Word(word) => {
                self.symbols.get(word).map(|i| (**i).value).ok_or(vec![
                    Message::error(format!("use of undeclared label: '{}'", word))
                        .with_code(String::from("unknown label"), token.span.clone())
                ])
            },
            TokenKind::String(_) => {
                Err(vec![
                    Message::error(format!("usage of strings as immediate values is currently not supported"))
                        .with_code(String::from("unsupported value"), token.span.clone())
                ])
            }
            _ => unreachable!("should be handled by the parser")
        }
    }

    fn compile_instruction(&mut self, node: &Node) -> Result<()> {
        match &node.value {
            NodeKind::Instruction { name, arguments } => {
                let mut valid_signatures = match INSTRUCTIONS.get(&name.value) {
                    Some(instruction) => instruction,
                    None => return Err(vec![
                        Message::error(format!("use of invalid instruction: '{}' does not exist", &name.value))
                            .with_code(String::from("unknown instruction"), name.span.clone())
                    ])
                }.signatures.clone();
                let valid_num_arguments: Vec<usize> = valid_signatures.iter()
                    .map(|signature| signature.arguments.len())
                    .unique()
                    .collect();

                let arguments_signature = signature::parse_arguments(arguments);

                valid_signatures = valid_signatures.into_iter()
                    .filter(|signature| signature.arguments.len() == arguments_signature.len())
                    .collect();

                if valid_signatures.is_empty() {
                    return Err(vec![
                        Message::error(format!(
                            "instruction '{}' takes {}, but {} were supplied",
                            &name.value,
                            {
                                if valid_num_arguments.len() == 1 {
                                    human_count("argument", valid_num_arguments[0])
                                } else {
                                    format!("{} arguments", valid_num_arguments.iter().join(" or "))
                                }
                            },
                            arguments_signature.len()
                        ))
                            .with_code(String::from("wrong number of arguments"), name.span.clone())
                    ]);
                }

                let best_match = valid_signatures.iter()
                    .map(|signature| {
                        let diff = signature.arguments.iter()
                            .zip(&arguments_signature)
                            .map(|(valid_arg, arg)| valid_arg == arg)
                            .collect::<Vec<bool>>();
                        let diffs = diff.iter()
                            .filter(|i| !**i)
                            .count();

                        (signature, diff, diffs)
                    })
                    .reduce(|(acc_sig, acc_diff, acc_diffs), (sig, diff, diffs)| {
                        if diffs < acc_diffs { (sig, diff, diffs) }
                        else { (acc_sig, acc_diff, acc_diffs) }
                    }).expect("valid_signatures.len() should >= 1");

                if best_match.2 != 0 {
                    let mut errors = vec![];
                    for (i, matches) in best_match.1.iter().enumerate() {
                        if !matches {
                            errors.push(
                                Message::error(format!(
                                    "expected argument {}, found {}",
                                    best_match.0.arguments[i],
                                    arguments_signature[i]
                                )).with_code(String::from("wrong argument"), arguments[i].span.clone())
                            )
                        }
                    }
                    return Err(errors)
                }

                self.write(best_match.0.code);

                for (sig, arg) in arguments_signature.iter().zip(arguments.iter()) {
                    if let Argument::Im = sig {
                        self.write(self.immediate(arg)?);
                    }
                }

                Ok(((), vec![]))
            }
            NodeKind::Value { value } => {
                self.write(self.immediate(&value)?);
                Ok(((), vec![]))
            }
            NodeKind::Label { .. } => Ok(((), vec![]))
        }
    }

    fn compile(&mut self, image_size: Option<usize>) -> Result<()> {
        let mut errors = vec![];
        let mut warnings = vec![];

        let mut program = self.program.iter();
        self.cursor = 0;

        while let Some(current) = program.next() {
            match self.compile_instruction(current) {
                Ok(((), mut w)) => warnings.append(&mut w),
                Err(mut e) => errors.append(&mut e)
            }
        }

        for (symbol, value) in &self.symbols {
            if !*value.used.borrow() {
                warnings.push(
                    Message::warning(format!("label '{}' is never used", symbol))
                        .with_code(String::from("unused label"),(*value).span.clone())
                )
            }
        }

        let mut padding_string = String::new();
        if let Some(image_size) = image_size {
            if self.cursor > image_size {
                errors.push(Message::error(format!("program ({}) does not fit inside image ({})!", human_count("byte", self.cursor), human_count("byte", image_size))));
                return Err(errors);
            }
            if self.cursor < image_size {
                let padding = image_size - self.cursor;
                self.output.write_all(&vec![0x00; padding][..]).unwrap();
                padding_string = format!(" (+ {} padding)", padding);
            }
        }
        println!("wrote {}{} into '{}'", human_count("byte", self.cursor), padding_string, self.output_path.display());
        
        if errors.is_empty() {
            Ok(((), warnings))
        } else {
            Err(errors)
        }
    }

    fn do_declaration_pass(&mut self) -> Result<()> {
        let mut errors = vec![];

        let mut program = self.program.iter();
        let mut cursor: usize = 0;

        while let Some(current) = program.next() {
            cursor += match &current.value {
                NodeKind::Instruction { arguments, .. } => {
                    let arguments_signature = signature::parse_arguments(arguments);
                    
                    1 + arguments_signature.iter().filter(|argument| argument == &&Argument::Im).count() as u8
                },
                NodeKind::Value { .. } => 1,
                NodeKind::Label { name } => {
                    let name_str = &name.value;

                    if let Some(previous) = self.symbols.get(name_str) {
                        errors.push(
                            Message::error(format!("redefinition of label '{}'", name_str))
                                .with_code(String::from("already defined"), name.span.clone())
                                .with_code_context(String::from("previously defined here"), previous.span.clone())
                        )
                    }

                    self.symbols.insert(name_str.clone(), (cursor as u8).with_span(name.span.clone()).into());

                    0
                }
            } as usize
        }

        if errors.is_empty() {
            Ok(((), vec![]))
        } else {
            Err(errors)
        }
    }
}

pub fn compile(source: Source, output_path: PathBuf, image_size: Option<usize>, verbose: bool) -> Result<()> {
    let mut warnings = vec![];

    if verbose {
        println!("lexing tokens...")
    }

    let (tokens, mut w) = Lexer::new(&Rc::new(source)).lex()?;
    warnings.append(&mut w);

    let mut scope = Scope::new(None);
    let ((), mut w) = Preprocessor::from(tokens).preprocess(&mut scope)?;
    let tokens = scope.tokens;
    warnings.append(&mut w);

    if verbose {
        println!("parsing nodes...")
    }

    let (nodes, mut w) = Parser::new(tokens).parse()?;
    warnings.append(&mut w);

    if verbose {
        println!("compiling...")
    }

    let mut output = File::create(&output_path).unwrap();

    let mut compiler = Compiler::new(&nodes, &mut output, output_path);
    let ((), mut w) = compiler.do_declaration_pass()?;
    warnings.append(&mut w);

    let ((), mut w) = compiler.compile(image_size)?;
    warnings.append(&mut w);

    Ok(((), warnings))
}