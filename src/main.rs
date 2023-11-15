use chumsky::Parser;
use lmntal_lsp::parsing;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

pub struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    #[doc = " The [`initialize`] request is the first request sent from the client to the server."]
    #[doc = ""]
    #[doc = " [`initialize`]: https://microsoft.github.io/language-server-protocol/specification#initialize"]
    #[doc = ""]
    #[doc = " This method is guaranteed to only execute once. If the client sends this request to the"]
    #[doc = " server again, the server will respond with JSON-RPC error code `-32600` (invalid request)."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        todo!()
    }

    #[doc = " The [`shutdown`] request asks the server to gracefully shut down, but to not exit."]
    #[doc = ""]
    #[doc = " [`shutdown`]: https://microsoft.github.io/language-server-protocol/specification#shutdown"]
    #[doc = ""]
    #[doc = " This request is often later followed by an [`exit`] notification, which will cause the"]
    #[doc = " server to exit immediately."]
    #[doc = ""]
    #[doc = " [`exit`]: https://microsoft.github.io/language-server-protocol/specification#exit"]
    #[doc = ""]
    #[doc = " This method is guaranteed to only execute once. If the client sends this request to the"]
    #[doc = " server again, the server will respond with JSON-RPC error code `-32600` (invalid request)."]
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    async fn shutdown(&self) -> Result<()> {
        todo!()
    }
}

#[tokio::main]
async fn main() {
    let src = std::fs::read_to_string(std::env::args().nth(1).unwrap()).unwrap();

    let parser = parsing::lexer().parse(src).unwrap();
    println!("{:#?}", parser);
}
