use dashmap::DashMap;
use tower_lsp::{lsp_types::*, Client, LanguageServer};
use tower_lsp::jsonrpc::Result;
use typort_interpreter::hir::parse_to_hir;
use typort_interpreter::mir::hir_to_mir;
use typort_parser::simple_example::TopItem;

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
    ast_map: DashMap<String, Vec<TopItem>>,
    hir_map: DashMap<String, Vec<typort_interpreter::hir::Class>>,
    mir_map: DashMap<String, Vec<typort_interpreter::mir::Class>>
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,
            ast_map: Default::default(),
            hir_map: Default::default(),
            mir_map: Default::default(),
        }
    }
    pub async fn on_change(&self, params: TextDocumentItem) {
        let (ast, parse_fail, _) = typort_parser::simple_example::file()
            .run_with_out(&params.text, Default::default());
        //TODO: diagnostic
        if let Some(ast) = ast {
            self.client.log_message(MessageType::INFO, "parse success").await;
            self.ast_map.insert(params.uri.to_string(), ast.clone());
            let hir = parse_to_hir(ast);
            self.hir_map.insert(params.uri.to_string(), hir.clone());
            let mir = hir_to_mir(hir);
            self.mir_map.insert(params.uri.to_string(), mir);
        } else {
            self.client.log_message(MessageType::INFO, "parse fail").await;
            self.client
                .publish_diagnostics(
                    params.uri,
                    vec![],//TODO:
                    None
                ).await
        }
    }
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
                code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(true) }),

                //definition_provider: Some(OneOf::Left(true)),
                //references_provider: Some(OneOf::Left(true)),
                //rename_provider: Some(OneOf::Left(true)),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
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
        self.client
            .log_message(MessageType::INFO, "shutdown!")
            .await;
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "on open").await;
        self.on_change(TextDocumentItem {
            uri:params.text_document.uri,
            text:params.text_document.text,
            version:params.text_document.version,
            language_id: "typort".to_owned(),
        })
        .await
    }
    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "on change").await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
            language_id: "typort".to_owned()
        })
        .await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let uri = params.text_document.uri;
        let codelens = || -> Option<Vec<CodeLens>> {
            let ast = self.ast_map.get(&uri.to_string())?;
            Some(ast.iter()
                .flat_map(|x| {
                    let extends = match x {
                        TopItem::Class(c) => (&c.name, &c.extends),
                        TopItem::Object(o) => (&o.name, &o.extends),
                    };
                    extends.1.as_ref().and_then(|x| if x.data == "App" {
                        let name = extends.0;
                        Some(CodeLens {
                            range: Range { start: Position {
                                line: name.range.0.0 as u32,
                                character: name.range.0.1 as u32,
                            }, end: Position {
                                line: name.range.1.0 as u32,
                                character: name.range.1.1 as u32,
                            } },
                            command: Some(Command {
                                title: "run code".to_owned(),
                                command: "typort".to_owned(),
                                arguments: Some(vec![
                                    serde_json::Value::String("cli".to_owned()),
                                    serde_json::Value::String(uri.path().to_owned()),
                                    serde_json::Value::String(name.data.clone()),
                                ]),
                            }),
                            data: Some(serde_json::Value::String("Run".to_owned())),
                        })
                    } else {
                        None
                    })
                }).collect())
        }();
        Ok(codelens)
    }
}
