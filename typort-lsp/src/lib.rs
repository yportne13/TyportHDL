use backend::Backend;
use tower_lsp::{LspService, Server};

mod backend;

pub async fn main_lsp() {

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(Backend::new)
        .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
