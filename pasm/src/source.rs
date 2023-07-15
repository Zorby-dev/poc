use std::{path::PathBuf, rc::Rc, fmt::Debug};

pub struct Source {
    pub text: String,
    pub path: PathBuf
}

impl Debug for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Source").field("path", &self.path).finish()
    }
}

#[derive(Debug, Clone)]
pub struct Span {
    pub begin: usize,
    pub end: usize,
    pub source: Rc<Source>
}

impl Span {
    pub fn new(begin: usize, end: usize, source: Rc<Source>) -> Self {
        Self {
            begin, end, source
        }
    }

    pub fn row_num(&self) -> usize {
        1 + self.source.text.chars()
            .take(self.begin)
            .filter(|ch| *ch == '\n')
            .count()
    }

    pub fn row_begin(&self) -> usize {
        self.source.text.chars()
            .take(self.begin)
            .enumerate()
            .filter(|(_, ch)| *ch == '\n')
            .last()
            .map(|(i, _)| i + 1)
            .unwrap_or(0)
    }

    pub fn row<'a>(&'a self) -> impl Iterator<Item = char> + 'a {
        self.source.text.chars()
            .skip(self.row_begin())
            .take_while(|ch| *ch != '\n')
    }

    pub fn col_num(&self) -> usize {
        self.begin + 1 - self.row_begin()
    }

    pub fn get_text(&self) -> String {
        self.source.text.chars()
            .skip(self.begin)
            .take(self.end - self.begin)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct WithSpan<T> {
    pub value: T,
    pub span: Span
}

pub trait IntoWithSpan {
    fn with_span(self, span: Span) -> WithSpan<Self> where Self: Sized;
}

impl<T> IntoWithSpan for T {
    fn with_span(self, span: Span) -> WithSpan<Self> where Self: Sized {
        WithSpan { value: self, span }
    }
}