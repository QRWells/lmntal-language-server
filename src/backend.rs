use crate::analysis::semantic_token::to_semantic_tokens;
use crate::analysis::Analyzer;
use crate::capabilities;
use crate::config::Config;
use crate::diagnostics::Diagnostics;
use crate::reference::RefereceMap;
use crate::utils::check_update;

use dashmap::DashMap;
use lmntalc::util::Source;
use lmntalc::ASTNode;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

pub struct Backend {
    client: Client,
    config: RwLock<Config>,
    ast_map: DashMap<String, ASTNode>,
    document_map: DashMap<String, Source>,
    document_symbol_map: DashMap<String, Vec<DocumentSymbol>>,
    semantic_token_map: DashMap<String, Vec<SemanticToken>>,
    reference_map: DashMap<String, RefereceMap>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(capabilities::capabilities())
    }

    async fn initialized(&self, _: InitializedParams) {
        let config_items = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("lmntal".to_string()),
            }])
            .await;

        let mut updated_config = false;
        let mut config = self.config.write().await;

        if let Ok(config_items) = config_items {
            if let Some(des_config) = config_items.into_iter().next() {
                if let Ok(new) = serde_json::from_value(des_config) {
                    *config = new;
                    updated_config = true;
                }
            }
        }

        if !updated_config {
            self.client
                .log_message(
                    MessageType::ERROR,
                    "Failed to retrieve configuration from client.",
                )
                .await;
        }

        self.client
            .log_message(
                MessageType::INFO,
                format!("LMNtal Language Server v{}", env!("CARGO_PKG_VERSION")),
            )
            .await;

        self.client
            .log_message(MessageType::INFO, "Checking for updates...".to_string())
            .await;

        if config.check_for_updates {
            if let Some(new_version) = check_update().await {
                self.client
                    .show_message(
                        MessageType::INFO,
                        format!(
                            "A new version of the LMNtal Language Server is available: v{}",
                            new_version
                        ),
                    )
                    .await;
            }
        }
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Ok(new) = serde_json::from_value(params.settings) {
            let mut config = self.config.write().await;
            *config = new;
            self.client
                .log_message(
                    MessageType::INFO,
                    "Updated configuration from client.".to_string(),
                )
                .await;
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
            language_id: "".to_string(),
        })
        .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        _ = params;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        _ = params;
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();
        if let Some(symbols) = self.document_symbol_map.get(&uri) {
            Ok(Some(DocumentSymbolResponse::Nested(symbols.clone())))
        } else {
            Ok(None)
        }
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let param = params.text_document_position_params;
        let uri = param.text_document.uri.to_string();
        if let Some(ref_map) = self.reference_map.get(&uri) {
            let line = param.position.line;
            let col = param.position.character;
            if let Some(refs) = ref_map.query_references_with_self(line, col) {
                Ok(Some(
                    refs.iter()
                        .map(|r| DocumentHighlight {
                            range: (*r).into(),
                            kind: None,
                        })
                        .collect(),
                ))
            } else if let Some(symbol) = ref_map.query(line, col) {
                Ok(Some(vec![DocumentHighlight {
                    range: symbol.into(),
                    kind: None,
                }]))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();
        if let Some(tokens) = self.semantic_token_map.get(&uri) {
            Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
                result_id: None,
                data: tokens.clone(),
            })))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        _ = params;
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        _ = params;
        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        _ = params;
        Ok(None)
    }
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            config: RwLock::new(Config::default()),
            ast_map: DashMap::new(),
            document_map: DashMap::new(),
            document_symbol_map: DashMap::new(),
            semantic_token_map: DashMap::new(),
            reference_map: DashMap::new(),
        }
    }

    async fn on_change(&self, doc: TextDocumentItem) {
        let uri = doc.uri;
        let text = doc.text;

        if text.is_empty() {
            return;
        }
        let src = Source::from_string(text);
        let mut lexer = lmntalc::LMNtalLexer::new(&src);
        let mut parser = lmntalc::LMNtalParser::new();

        let mut diagnostics = Diagnostics::default();
        let lexing_result = lexer.lex();
        diagnostics.extend(lexing_result.errors);

        let parsing_result = parser.parse(lexing_result.tokens);
        diagnostics.extend(parsing_result.parsing_errors);
        diagnostics.extend(parsing_result.parsing_warnings);

        let ast = parsing_result.ast;
        let analyzer = Analyzer::new(uri.clone(), &ast);
        let mut analysis_result = analyzer.analyze();
        let tokens = to_semantic_tokens(&mut analysis_result.semantic_tokens);

        self.ast_map.insert(uri.to_string(), ast);
        self.document_map.insert(uri.to_string(), src);
        self.semantic_token_map.insert(uri.to_string(), tokens);
        self.document_symbol_map
            .insert(uri.to_string(), analysis_result.doc_symbol);
        self.reference_map.insert(
            uri.to_string(),
            RefereceMap::new(analysis_result.refs, analysis_result.symbols),
        );
        diagnostics.extend(analysis_result.diagnostics);

        self.client
            .publish_diagnostics(uri, diagnostics.diagnostics, None)
            .await;
    }
}
