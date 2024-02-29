use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

pub const LEGEND_TYPE: &[SemanticTokenType] = &[
    SemanticTokenType::FUNCTION,  // Rule
    SemanticTokenType::NAMESPACE, // Membrane
    SemanticTokenType::CLASS,     // Atom
    SemanticTokenType::VARIABLE,  // Link
    SemanticTokenType::STRUCT,    // Hyperlink
    SemanticTokenType::PROPERTY,  // Context
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::COMMENT,
];

pub const RULE_LEGEND_TYPE: u32 = 0;
pub const MEMBRANE_LEGEND_TYPE: u32 = 1;
pub const ATOM_LEGEND_TYPE: u32 = 2;
pub const LINK_LEGEND_TYPE: u32 = 3;
pub const HYPERLINK_LEGEND_TYPE: u32 = 4;
pub const CONTEXT_LEGEND_TYPE: u32 = 5;
pub const KEYWORD_ATOM_LEGEND_TYPE: u32 = 6;
pub const OPERATOR_ATOM_LEGEND_TYPE: u32 = 7;
pub const STRING_ATOM_LEGEND_TYPE: u32 = 8;
pub const NUMBER_ATOM_LEGEND_TYPE: u32 = 9;

#[derive(Debug, Default)]
pub struct Token {
    pub token_type: u32,
    pub line: u32,
    pub col: u32,
    pub length: usize,
}

pub fn to_semantic_tokens(tokens: &mut [Token]) -> Vec<SemanticToken> {
    let mut last_line: u32 = 0;
    let mut last_start: u32 = 0;
    tokens.sort_by(|a, b| a.line.cmp(&b.line).then(a.col.cmp(&b.col)));
    tokens
        .iter()
        .map(|token| {
            let (line, col) = (token.line, token.col);
            let length = token.length;
            let delta_line = line - last_line;
            let delta_start = if delta_line == 0 {
                col - last_start
            } else {
                col
            };
            last_line = line;
            last_start = col;
            SemanticToken {
                delta_line,
                delta_start,
                length: length as u32,
                token_type: token.token_type,
                token_modifiers_bitset: 0,
            }
        })
        .collect()
}
