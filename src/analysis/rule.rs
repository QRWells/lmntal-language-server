use lmntalc::ASTNode;
use tower_lsp::lsp_types::{DocumentSymbol, Position, Range, SymbolKind};

use crate::utils::span_to_range;

use super::{
    semantic_token::{Token, RULE_LEGEND_TYPE},
    Analyzer,
};

#[derive(Debug, Default)]
pub(super) struct RuleAnalysisResult {
    pub(super) symbols: Vec<DocumentSymbol>,
}

impl<'ast> Analyzer<'ast> {
    pub(super) fn analyze_rule(&mut self, ast: &ASTNode) -> RuleAnalysisResult {
        if let ASTNode::Rule {
            name,
            head,
            propagation,
            guard,
            body,
            span,
        } = ast
        {
            let mut selection_range = span_to_range(*span);
            let mut range = span_to_range(*span);
            if !name.1.is_empty() {
                self.semantic_tokens.push(Token {
                    line: name.1.low().line,
                    col: name.1.low().column,
                    length: name.1.len(),
                    token_type: RULE_LEGEND_TYPE,
                });
                selection_range = span_to_range(name.1);
                range = Range {
                    start: Position {
                        line: name.1.low().line,
                        character: name.1.low().column,
                    },
                    end: Position {
                        line: span.high().line,
                        character: span.high().column,
                    },
                };
            }

            let mut result = self.analyze_process_list(head);

            if let Some(propagation) = propagation {
                result.extend(self.analyze_process_list(propagation));
            }

            self.filter_links_inner(&mut result.link_occurrences);

            if let Some(guard) = guard {
                self.analyze_guard(guard);
            }

            if let Some(body) = body {
                result.extend(self.analyze_process_list(body));
            }

            self.filter_links_top(result.link_occurrences);

            RuleAnalysisResult {
                symbols: vec![DocumentSymbol {
                    name: name.0.clone(),
                    detail: None,
                    kind: SymbolKind::FUNCTION,
                    tags: None,
                    deprecated: None,
                    range,
                    selection_range,
                    children: None,
                }],
            }
        } else {
            unreachable!()
        }
    }

    fn analyze_guard(&mut self, _guard: &ASTNode) {}
}
