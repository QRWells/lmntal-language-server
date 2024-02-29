use std::fmt::{Display, Formatter};

use lmntalc::util::Span;
use tower_lsp::lsp_types::{Position, Range};

/// A symbol in the source code.
///
/// The position is zero-based.
#[derive(Debug, Copy, Clone)]
pub struct Symbol {
    pub line: u32,
    pub col: u32,
    pub length: usize,
}

impl Symbol {
    pub fn new(span: Span) -> Self {
        Self {
            line: span.low().line,
            col: span.low().column,
            length: span.len(),
        }
    }

    pub fn is_inside(&self, line: u32, col: u32) -> bool {
        self.line == line && self.col <= col && col <= self.col + self.length as u32
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.line == other.line && self.col == other.col && self.length == other.length
    }
}

impl Eq for Symbol {}

impl std::hash::Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.col.hash(state);
        self.length.hash(state);
    }
}

impl std::cmp::Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line
            .cmp(&other.line)
            .then(self.col.cmp(&other.col))
            .then(self.length.cmp(&other.length))
    }
}

impl std::cmp::PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, col: {}, length: {}",
            self.line, self.col, self.length
        )
    }
}

impl From<Symbol> for Range {
    fn from(val: Symbol) -> Self {
        Range {
            start: Position {
                line: val.line,
                character: val.col,
            },
            end: Position {
                line: val.line,
                character: val.col + val.length as u32,
            },
        }
    }
}
