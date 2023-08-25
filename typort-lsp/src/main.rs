use backend::Backend;
use tower_lsp::{LspService, Server};

mod backend;

#[tokio::main]
async fn main() {

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
