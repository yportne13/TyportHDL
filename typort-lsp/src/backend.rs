use tower_lsp::{lsp_types::*, Client, LanguageServer};
use tower_lsp::jsonrpc::Result;

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                //inlay_hint_provider: Some(OneOf::Left(true)),
                //completion_provider: Some(CompletionOptions {
                //    resolve_provider: Some(false),
                //    trigger_characters: Some(vec![".".to_string()]),
                //    work_done_progress_options: Default::default(),
                //    all_commit_characters: None,
                //    completion_item: None,
                //}),

                //definition_provider: Some(OneOf::Left(true)),
                //references_provider: Some(OneOf::Left(true)),
                //rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    
}
