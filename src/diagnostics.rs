use std::vec;

use lmntalc::frontend::{
    lexing::{LexError, LexErrorType},
    parsing::{ParseError, ParseErrorType, ParseWarning, ParseWarningType},
};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Range};

use crate::utils::to_position;

#[derive(Debug, Default)]
pub struct Diagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

impl Diagnostics {
    pub fn push(&mut self, diagnostic: impl DiagnosticProvider) {
        let diag = diagnostic.diagnostics();
        self.diagnostics.extend(diag);
    }

    pub fn extend(&mut self, diagnostics: impl IntoIterator<Item = impl DiagnosticProvider>) {
        for diagnostic in diagnostics {
            self.push(diagnostic);
        }
    }

    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }
}

pub trait DiagnosticProvider {
    fn diagnostics(&self) -> Vec<Diagnostic>;
}

impl DiagnosticProvider for LexError {
    fn diagnostics(&self) -> Vec<Diagnostic> {
        match self.ty {
            LexErrorType::Expected(c) => vec![Diagnostic {
                range: Range {
                    start: to_position(self.pos),
                    end: to_position(self.pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Expected {}", c),
                ..Default::default()
            }],
            LexErrorType::UnexpectedCharacter(c) => vec![Diagnostic {
                range: Range {
                    start: to_position(self.pos),
                    end: to_position(self.pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Unexpected character: {}", c),
                ..Default::default()
            }],
            LexErrorType::UnmatchedBracket(c, pos) => vec![Diagnostic {
                range: Range {
                    start: to_position(pos),
                    end: to_position(pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Unmatched bracket: {}", c),
                ..Default::default()
            }],
            LexErrorType::UncompleteNumber => vec![Diagnostic {
                range: Range {
                    start: to_position(self.pos),
                    end: to_position(self.pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Uncomplete number".to_string(),
                ..Default::default()
            }],
            LexErrorType::UncompleteString => vec![Diagnostic {
                range: Range {
                    start: to_position(self.pos),
                    end: to_position(self.pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Uncomplete string".to_string(),
                ..Default::default()
            }],
            LexErrorType::UnclosedQuote => vec![Diagnostic {
                range: Range {
                    start: to_position(self.pos),
                    end: to_position(self.pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unclosed quote".to_string(),
                ..Default::default()
            }],
            LexErrorType::UnclosedComment => vec![Diagnostic {
                range: Range {
                    start: to_position(self.pos),
                    end: to_position(self.pos),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unclosed comment".to_string(),
                ..Default::default()
            }],
        }
    }
}

impl DiagnosticProvider for ParseWarning {
    fn diagnostics(&self) -> Vec<Diagnostic> {
        match self.ty {
            ParseWarningType::MissingCommaBetweenProcesses => {
                vec![Diagnostic {
                    range: Range {
                        start: to_position(self.span.low()),
                        end: to_position(self.span.high()),
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: "Missing comma between processes".to_string(),
                    ..Default::default()
                }]
            }
        }
    }
}

impl DiagnosticProvider for ParseError {
    fn diagnostics(&self) -> Vec<Diagnostic> {
        match &self.ty {
            ParseErrorType::UnexpectedToken { expected, found } => vec![Diagnostic {
                range: Range {
                    start: to_position(self.span.low()),
                    end: to_position(self.span.high()),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Unexpected token: expected {}, found {}", expected, found),
                ..Default::default()
            }],
            ParseErrorType::UnexpectedEOF => vec![Diagnostic {
                range: Range {
                    start: to_position(self.span.low()),
                    end: to_position(self.span.high()),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: "Unexpected end of file".to_string(),
                ..Default::default()
            }],
            ParseErrorType::WrongCase(kind) => vec![Diagnostic {
                range: Range {
                    start: to_position(self.span.low()),
                    end: to_position(self.span.high()),
                },
                severity: Some(DiagnosticSeverity::ERROR),
                message: format!("Wrong case for {}", kind),
                ..Default::default()
            }],
        }
    }
}

impl DiagnosticProvider for tower_lsp::lsp_types::Diagnostic {
    fn diagnostics(&self) -> Vec<Diagnostic> {
        vec![self.clone()]
    }
}
