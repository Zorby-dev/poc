use std::{collections::HashMap, fmt::Display};

use lazy_static::lazy_static;
use serde::Deserialize;

use crate::lexer::{Token, TokenKind};

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum Argument {
    #[serde(rename = "rx")] Rx,
    #[serde(rename = "ry")] Ry,
    #[serde(rename = "rz")] Rz,
    #[serde(rename = "im")] Im
}

impl Argument {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Rx => "'rx'",
            Self::Ry => "'ry'",
            Self::Rz => "'rz'",
            Self::Im => "<immediate>"
        }
    }

    pub fn assembly_name(&self) -> &'static str {
        match self {
            Self::Rx => "rx",
            Self::Ry => "ry",
            Self::Rz => "rz",
            Self::Im => "<im>"
        }
    }
}

impl Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Signature {
    pub arguments: Vec<Argument>,
    pub code: u8
}

#[derive(Deserialize, Debug)]
pub struct Instruction {
    pub signatures: Vec<Signature>
}

pub type Instructions = HashMap<String, Instruction>;

lazy_static! {
    pub static ref INSTRUCTIONS: Instructions = serde_json::from_str(include_str!("instructions.json")).unwrap();
}

pub fn parse_arguments(tokens: &Vec<Token>) -> Vec<Argument> {
    tokens.iter()
        .map(|token| match &token.value {
            TokenKind::Word(word) => match word.as_str() {
                "rx" => Argument::Rx,
                "ry" => Argument::Ry,
                "rz" => Argument::Rz,
                _    => Argument::Im
            }
            | TokenKind::Character(..)
            | TokenKind::Number(..) => Argument::Im,
            _                      => unreachable!("should be handled by the parser")
        })
        .collect()
}

pub fn format_instruction_code(code: u8) -> Option<String> {
    for (name, instruction) in &*INSTRUCTIONS {
        for signature in &instruction.signatures {
            if signature.code == code {
                return Some(format!(
                    "{} {}",
                    name,
                    signature.arguments.iter()
                        .map(|arg| arg.assembly_name())
                        .collect::<Vec<&'static str>>()
                        .join(",")
                ));
            }
        }
    }

    None
}