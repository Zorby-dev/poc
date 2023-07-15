use std::io::Write;

use crate::{source::Span, next_n::NextN};
use colored::*;

#[derive(Debug)]
struct CodeSnippet {
    description: String,
    span: Span,
    color: Color
}

impl CodeSnippet {
    fn line_num_len(&self) -> usize {
        self.span.row_num().to_string().len()
    }

    fn format<T: Write>(&self, mut out: T, pad_line_num: usize) {
        let row = self.span.row_num().to_string();
        let line_num_len = row.len();
        let col = self.span.col_num();
        let row_len = pad_line_num + 2;
        let mut line = self.span.row();
        let before = line.next_n::<String>(col - 1);
        let error = line.next_n::<String>(self.span.end - self.span.begin);
        let after = line.collect::<String>();

        writeln!(out, "{} {}{}{}\n{}{}",
            format!("{}╮┄┄ {}:{}:{}\n {}{} │",
                "─".repeat(row_len),
                self.span.source.path.display(), &row, col,
                " ".repeat(pad_line_num - line_num_len), &row
            ).bright_black(),
            before, (&error).color(self.color), after,
            format!("{}╯ {}", "─".repeat(row_len), " ".repeat(before.len())).bright_black(),
            format!("╰{} {}", "─".repeat(error.len() + 1), &self.description).color(self.color),
        ).unwrap();
    }
}

#[derive(Debug)]
pub enum MessageKind {
    Error, Warning
}

impl MessageKind {
    fn as_str(&self) -> &'static str {
        match self {
            MessageKind::Error => "error",
            MessageKind::Warning => "warning"
        }
    }

    fn as_color(&self) -> Color {
        match self {
            MessageKind::Error => Color::Red,
            MessageKind::Warning => Color::Yellow
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub kind: MessageKind,
    message: String,
    code_snippets: Vec<CodeSnippet>,
    note: Option<String>
}

impl Message {
    pub fn error(message: String) -> Self {
        Self {
            kind: MessageKind::Error,
            message,
            code_snippets: vec![],
            note: None
        }
    }
    
    pub fn warning(message: String) -> Self {
        Self {
            kind: MessageKind::Warning,
            message,
            code_snippets: vec![],
            note: None
        }
    }

    pub fn with_code(mut self, description: String, span: Span) -> Self {
        self.code_snippets.push(CodeSnippet {
            description, span,
            color: self.kind.as_color()
        });
        self
    }

    pub fn with_code_context(mut self, description: String, span: Span) -> Self {
        self.code_snippets.push(CodeSnippet {
            description, span,
            color: Color::Cyan
        });
        self
    }

    pub fn with_note(mut self, note: String) -> Self {
        self.note = Some(note);
        self
    }

    pub fn format<T: Write>(&self, mut out: T) {
        let color = self.kind.as_color();

        writeln!(out, "{}",
            format!("{}: {}", self.kind.as_str().color(color), &self.message).bold()
        ).unwrap();

        if !self.code_snippets.is_empty() {
            let max_line_num_len = self.code_snippets.iter()
                .map(|snip| snip.line_num_len())
                .max()
                .unwrap();
            for snippet in &self.code_snippets {
                snippet.format(&mut out, max_line_num_len)
            }
        }

        if let Some(note) = &self.note {
            writeln!(out, "{}",
                format!("{}: {}", "note".cyan(), note).bold()
            ).unwrap();
        }
    }
}

pub fn human_count(of: &str, count: usize) -> String {
    format!("{} {}{}",
        count, of, {
            if count == 1 {
                ""
            } else {
                "s"
            }
        }
    )
}

pub type Result<T> = core::result::Result<(T, Vec<Message>), Vec<Message>>;