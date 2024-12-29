use std::{collections::HashMap, error::Error};

use lsp_server::{ExtractError, IoThreads, Message, ProtocolError, Request, RequestId, Response};
use request::{GotoDefinition, HoverRequest, InlayHintRequest};
use ropey::Rope;
//use typort_interpreter::mir::hir_to_mir;
use typort_parser::{combinator::PathId, parse, Fn};
use typort_tyck::{hir::{self, Expr, ExprIter}, tyck};
use lsp_types::*;

use crate::{client::Client, ls::{LanguageServer, Result}};

pub struct Backend {
    pub client: Client,
    path_id: HashMap<String, PathId>,
    document_map: HashMap<String, Rope>,
    ast_map: HashMap<String, Vec<Fn>>,
    hir_map: HashMap<String, Vec<hir::Fn>>,
    //mir_map: HashMap<String, Vec<typort_interpreter::mir::Class>>
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Backend {
            client,
            path_id: Default::default(),
            document_map: Default::default(),
            ast_map: Default::default(),
            hir_map: Default::default(),
        }
    }
    pub fn init(&self) -> std::result::Result<serde_json::Value, ProtocolError> {
        let server_capabilities = serde_json::to_value(
            self.initialize(Default::default()).unwrap().capabilities
        ).unwrap();
        self.client.connection.initialize(server_capabilities)
    }
    pub fn main_loop(&self) -> std::result::Result<(), Box<dyn Error + Sync + Send>> {
        eprintln!("starting example main loop");
        for msg in &self.client.connection.receiver {
            eprintln!("got msg: {msg:?}");
            match msg {
                Message::Request(req) => {
                    if self.client.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    eprintln!("got request: {req:?}");
                    match cast::<GotoDefinition>(req.clone()) {
                        Ok((id, params)) => {
                            eprintln!("got gotoDefinition request #{id}: {params:?}");
                            let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                            let result = serde_json::to_value(&result).unwrap();
                            let resp = Response { id, result: Some(result), error: None };
                            self.client.connection.sender.send(Message::Response(resp))?;
                            continue;
                        }
                        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                        Err(ExtractError::MethodMismatch(req)) => req,
                    };
                    match cast::<HoverRequest>(req.clone()) {
                        Ok((id, params)) => {
                            let result = self.hover(params)?;
                            let result = serde_json::to_value(&result).unwrap();
                            let resp = Response { id, result: Some(result), error: None };
                            self.client.connection.sender.send(Message::Response(resp))?;
                            continue;
                        }
                        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                        Err(ExtractError::MethodMismatch(req)) => req,
                    };
                    match cast::<InlayHintRequest>(req) {
                        Ok((id, params)) => {
                            eprintln!("got gotoDefinition request #{id}: {params:?}");
                            let result = Some(GotoDefinitionResponse::Array(Vec::new()));
                            let result = serde_json::to_value(&result).unwrap();
                            let resp = Response { id, result: Some(result), error: None };
                            self.client.connection.sender.send(Message::Response(resp))?;
                            continue;
                        }
                        Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                        Err(ExtractError::MethodMismatch(req)) => req,
                    };
                    // ...
                }
                Message::Response(resp) => {
                    eprintln!("got response: {resp:?}");
                }
                Message::Notification(not) => {
                    eprintln!("got notification: {not:?}");
                }
            }
        }
        Ok(())
    }
    pub fn on_change(&mut self, params: TextDocumentItem) {
        let path_id_len = self.path_id.len() as u32;
        let path_id = self
            .path_id
            .entry(params.uri.path().to_owned())
            .or_insert(path_id_len);
        let ast = parse(&params.text, *path_id);
        //TODO: diagnostic
        //if let Some(ast) = ast {
            self.client.log_message(MessageType::INFO, "parse success");
            self.ast_map.insert(params.uri.to_string(), ast.clone());
            let hir = tyck(ast, Default::default());
            match hir {
                Ok(hir) => {
                    self.hir_map.insert(params.uri.to_string(), hir.clone());
                }
                Err(e) => {
                    todo!()
                }
            }
            //let mir = hir_to_mir(hir);
            //self.mir_map.insert(params.uri.to_string(), mir);
        /*} else {
            self.client.log_message(MessageType::INFO, "parse fail");
            self.client
                .publish_diagnostics(
                    params.uri,
                    vec![],//TODO:
                    None
                )
        }*/
    }
}

impl LanguageServer for Backend {
    fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                //inlay_hint_provider: Some(OneOf::Left(true)),
                //completion_provider: Some(CompletionOptions {
                //    resolve_provider: Some(false),
                //    trigger_characters: Some(vec![".".to_string()]),
                //    work_done_progress_options: Default::default(),
                //    all_commit_characters: None,
                //    completion_item: None,
                //}),
                //code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(true) }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),

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
    fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!");
    }

    fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "shutdown!");
        Ok(())
    }

    fn did_open(&mut self, params: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "on open");
        self.on_change(TextDocumentItem {
            uri:params.text_document.uri,
            text:params.text_document.text,
            version:params.text_document.version,
            language_id: "typort".to_owned(),
        })
    }
    fn did_change(&mut self, mut params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "on change");
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
            language_id: "typort".to_owned()
        })
    }

    fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let definition = || {
            let url = params.text_document_position_params.text_document.uri.as_str();
            let rope = self.document_map.get(url)?;
            let position = params.text_document_position_params.position;
            let char = rope.try_line_to_char(position.line as usize).ok()?;
            let offset = char + position.character as usize;
            let hir= self.hir_map.get(url)?;
            hir.iter()
                .flat_map(|x| x.expr_iter())
                .flat_map(|x| match x {
                    Expr::Name(x) => Some(x),
                    _ => None,
                })
                .find(|x| x.contains(offset))
                .map(|x| &x.data.1)
                .map(|x| Hover {
                    contents: HoverContents::Scalar(MarkedString::String(
                        format!("{:?}", x)
                    )),
                    range: None,
                })
        };
        Ok(definition())
    }
    /*fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
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
    }*/
}

fn cast<R>(req: Request) -> std::result::Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}
