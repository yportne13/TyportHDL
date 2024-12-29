
use std::fmt::Display;

use lsp_types::*;

/// A JSON-RPC error object.
#[derive(Clone, Debug)]
/// #[serde(deny_unknown_fields)]
pub struct Error {}

impl Error {
    fn method_not_found() -> Self {
        Error {}
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub trait LanguageServer {
    fn initialize(&self, params: InitializeParams) -> Result<InitializeResult>;

    fn initialized(&self, params: InitializedParams) {
        let _ = params;
    }

    fn shutdown(&self) -> Result<()>;

    fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        let _ = params;
    }

    fn did_change(&mut self, params: DidChangeTextDocumentParams) {
        let _ = params;
    }

    fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let _ = params;
        //error!("Got a textDocument/hover request, but it is not implemented");
        Err(Error::method_not_found())
    }
}
