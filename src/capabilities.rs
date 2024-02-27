use tower_lsp::lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, DocumentFilter,
    HoverProviderCapability, InitializeResult, OneOf, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensRegistrationOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, ServerInfo, StaticRegistrationOptions,
    TextDocumentRegistrationOptions, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, WorkDoneProgressOptions,
};

use crate::analysis::LEGEND_TYPE;

pub fn capabilities() -> InitializeResult {
    let semantic_tokens_registration_options = SemanticTokensRegistrationOptions {
        text_document_registration_options: {
            TextDocumentRegistrationOptions {
                document_selector: Some(vec![DocumentFilter {
                    language: Some("lmntal".to_string()),
                    scheme: Some("file".to_string()),
                    pattern: None,
                }]),
            }
        },
        semantic_tokens_options: SemanticTokensOptions {
            work_done_progress_options: WorkDoneProgressOptions::default(),
            legend: SemanticTokensLegend {
                token_types: LEGEND_TYPE.into(),
                token_modifiers: vec![],
            },
            range: Some(false),
            full: Some(SemanticTokensFullOptions::Bool(true)),
        },
        static_registration_options: StaticRegistrationOptions::default(),
    };

    InitializeResult {
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    ..Default::default()
                },
            )),
            semantic_tokens_provider: Some(
                SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                    semantic_tokens_registration_options,
                ),
            ),
            code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::QUICKFIX,
                    CodeActionKind::REFACTOR_EXTRACT,
                    CodeActionKind::REFACTOR_INLINE,
                ]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: None,
            })),
            definition_provider: Some(OneOf::Left(true)),
            references_provider: Some(OneOf::Left(true)),
            rename_provider: Some(OneOf::Left(true)),
            document_symbol_provider: Some(OneOf::Left(true)),
            document_formatting_provider: Some(OneOf::Left(true)),
            document_highlight_provider: Some(OneOf::Left(true)),
            hover_provider: Some(HoverProviderCapability::Simple(true)),
            ..ServerCapabilities::default()
        },
        server_info: Some(ServerInfo {
            name: "LMNtal Language Server".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
        }),
        offset_encoding: None,
    }
}
