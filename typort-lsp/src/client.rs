use lsp_server::Connection;
use lsp_types::*;
use notification::{LogMessage, Notification, PublishDiagnostics};
use std::fmt::Display;

pub struct Client {
    pub connection: Connection,
}

impl Client {
    pub fn log_message<M: Display>(&self, typ: MessageType, message: M) {
        self.connection
            .sender
            .send(lsp_server::Message::Notification(
                lsp_server::Notification::new(
                    LogMessage::METHOD.to_owned(),
                    LogMessageParams {
                        typ,
                        message: message.to_string(),
                    },
                ),
            ))
            .unwrap();
    }
    pub fn publish_diagnostics(&self, uri: Url, diagnostics: Vec<Diagnostic>, version: Option<i32>) {
        self.connection
            .sender
            .send(lsp_server::Message::Notification(
                lsp_server::Notification::new(
                    PublishDiagnostics::METHOD.to_owned(),
                    PublishDiagnosticsParams {
                        uri,
                        diagnostics,
                        version,
                    },
                ),
            ))
            .unwrap();
    }
}
