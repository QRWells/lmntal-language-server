use tower_lsp::lsp_types::Url;

struct TextDocument {
    uri: Url,
    text: String,
    version: i32,
}
