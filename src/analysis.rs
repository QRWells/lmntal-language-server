pub mod rule;
pub mod semantic_token;

use self::{
    rule::RuleAnalysisResult,
    semantic_token::{
        Token, ATOM_LEGEND_TYPE, CONTEXT_LEGEND_TYPE, LINK_LEGEND_TYPE, MEMBRANE_LEGEND_TYPE,
    },
};
use crate::utils::span_to_range;
use lmntalc::{util::Span, ASTNode};
use std::collections::HashMap;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DocumentSymbol, Location, SymbolKind, Url,
};

pub use self::semantic_token::LEGEND_TYPE;

#[derive(Debug, Default)]
pub struct ProgramInfo {
    pub semantic_tokens: Vec<Token>,
    pub doc_symbol: Vec<DocumentSymbol>,
    pub diagnostics: Vec<Diagnostic>,
    pub refs: Vec<Vec<Span>>,
}

#[derive(Debug)]
pub struct Analyzer<'ast> {
    uri: Url,
    ast: &'ast ASTNode,
    semantic_tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    refs: Vec<Vec<Span>>,
}

#[derive(Debug, Default)]
pub struct AnalysisResult {
    symbols: Vec<DocumentSymbol>,
    link_occurrences: HashMap<String, Vec<Span>>,
    hyperlink_occurrences: HashMap<String, Vec<Span>>,
}

impl<'ast> Analyzer<'ast> {
    pub fn new(uri: Url, ast: &'ast ASTNode) -> Self {
        Self {
            uri,
            ast,
            semantic_tokens: Vec::new(),
            diagnostics: Vec::new(),
            refs: Vec::new(),
        }
    }

    pub fn analyze(mut self) -> ProgramInfo {
        let mut result = AnalysisResult::default();

        if let ASTNode::Membrane {
            process_lists,
            rules,
            ..
        } = self.ast
        {
            for process_list in process_lists {
                let res = self.analyze_process_list(process_list);
                result.extend(res);
            }

            for rule in rules {
                let rule_info = self.analyze_rule(rule);
                result.extend_rules(rule_info);
            }
        }

        self.filter_links_top(result.link_occurrences);

        self.refs
            .extend(result.hyperlink_occurrences.values().cloned());

        ProgramInfo {
            semantic_tokens: self.semantic_tokens,
            doc_symbol: result.symbols,
            refs: self.refs,
            diagnostics: self.diagnostics,
        }
    }

    fn analyze_process_list(&mut self, ast: &ASTNode) -> AnalysisResult {
        if let ASTNode::ProcessList { processes, .. } = ast {
            let mut result = AnalysisResult::default();
            for process in processes {
                result.extend(self.analyze_process(process));
            }
            result
        } else {
            unreachable!()
        }
    }

    fn analyze_process(&mut self, process: &ASTNode) -> AnalysisResult {
        let mut result = AnalysisResult::default();
        match process {
            ASTNode::Membrane { .. } => {
                result.extend(self.analyze_membrane(process));
            }
            ASTNode::Atom { name, args, .. } => {
                for arg in args {
                    result.extend(self.analyze_process(arg));
                }

                self.semantic_tokens.push(Token {
                    line: name.1.low().line,
                    col: name.1.low().column,
                    length: name.1.len(),
                    token_type: ATOM_LEGEND_TYPE,
                });
            }
            ASTNode::Link {
                name,
                hyperlink,
                span,
            } => {
                self.semantic_tokens.push(Token {
                    line: span.low().line,
                    col: span.low().column,
                    length: span.len(),
                    token_type: LINK_LEGEND_TYPE,
                });
                if *hyperlink {
                    result
                        .hyperlink_occurrences
                        .entry(name.clone())
                        .or_default()
                        .push(*span);
                } else {
                    result
                        .link_occurrences
                        .entry(name.clone())
                        .or_default()
                        .push(*span);
                }
            }
            ASTNode::Context { span, .. } => self.semantic_tokens.push(Token {
                line: span.low().line,
                col: span.low().column,
                length: span.len(),
                token_type: CONTEXT_LEGEND_TYPE,
            }),
            _ => unreachable!(),
        }
        result
    }

    fn analyze_membrane(&mut self, ast: &ASTNode) -> AnalysisResult {
        if let ASTNode::Membrane {
            name,
            process_lists,
            rules,
            span,
        } = ast
        {
            let mut result = AnalysisResult::default();

            for process_list in process_lists {
                result.extend(self.analyze_process_list(process_list));
            }

            self.filter_links_inner(&mut result.link_occurrences);

            for rule in rules {
                result.extend_rules(self.analyze_rule(rule));
            }

            self.semantic_tokens.push(Token {
                line: name.1.low().line,
                col: name.1.low().column,
                length: name.1.len(),
                token_type: MEMBRANE_LEGEND_TYPE,
            });

            let children = std::mem::take(&mut result.symbols);

            result.symbols.push(DocumentSymbol {
                name: if name.0.is_empty() {
                    "Anonymous membrane".to_string()
                } else {
                    name.0.clone()
                },
                detail: None,
                kind: SymbolKind::STRUCT,
                tags: None,
                deprecated: None,
                range: span_to_range(*span),
                selection_range: span_to_range(name.1),
                children: Some(children),
            });

            result
        } else {
            unreachable!()
        }
    }

    fn filter_links_top(&mut self, links: HashMap<String, Vec<Span>>) {
        for (_, occur) in links {
            match occur.len() {
                0 => {}
                1 => {
                    self.diagnostics.push(Diagnostic {
                        range: span_to_range(occur[0]),
                        severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
                        code: None,
                        source: None,
                        message: "Free link".to_string(),
                        related_information: None,
                        tags: None,
                        data: None,
                        code_description: None,
                    });
                }
                2 => self.refs.push(occur),
                _ => self.report_multi_occur(&occur),
            }
        }
    }

    fn filter_links_inner(&mut self, links: &mut HashMap<String, Vec<Span>>) {
        links.retain(|_, links| match links.len() {
            0 | 1 => true,
            2 => {
                self.refs.push(links.clone());
                false
            }
            _ => {
                self.report_multi_occur(links);
                false
            }
        });
    }

    fn report_multi_occur(&mut self, occurs: &[Span]) {
        let mut occurs = occurs.iter();
        let relate = vec![
            DiagnosticRelatedInformation {
                location: Location {
                    range: occurs.next().map(|x| span_to_range(*x)).unwrap(),
                    uri: self.uri.clone(),
                },
                message: "First occurrence".to_string(),
            },
            DiagnosticRelatedInformation {
                location: Location {
                    range: occurs.next().map(|x| span_to_range(*x)).unwrap(),
                    uri: self.uri.clone(),
                },
                message: "Second occurrence".to_string(),
            },
        ];

        for occur in occurs {
            self.diagnostics.push(Diagnostic {
                range: span_to_range(*occur),
                severity: Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
                code: None,
                source: None,
                message: "Link occurs more than twice".to_string(),
                related_information: Some(relate.clone()),
                tags: None,
                data: None,
                code_description: None,
            });
        }
    }
}

impl AnalysisResult {
    fn extend(&mut self, other: AnalysisResult) {
        for (link, occur) in other.link_occurrences {
            self.link_occurrences.entry(link).or_default().extend(occur);
        }
        for (link, occur) in other.hyperlink_occurrences {
            self.hyperlink_occurrences
                .entry(link)
                .or_default()
                .extend(occur);
        }
        self.symbols.extend(other.symbols);
    }

    fn extend_rules(&mut self, rule_result: RuleAnalysisResult) {
        self.symbols.extend(rule_result.symbols);
    }
}
