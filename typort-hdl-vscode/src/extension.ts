import { commands, workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions
} from 'vscode-languageclient';

let client: LanguageClient;

function startServer() {
	let typortls_binary_path: string | undefined = workspace.getConfiguration("typort-hdl").get("typort-lsp-Binary.path");
	if (typeof typortls_binary_path === "undefined")	typortls_binary_path = "typort-lsp";
	let serverOptions: ServerOptions = {
		run: {command: typortls_binary_path},
		debug: {command: typortls_binary_path},
	};

	// Options to control the language client
	let clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ scheme: 'file', language: 'typort' }],
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'typort-lsp',
		'typort language server',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

function stopServer(): Thenable<void> {
	if (!client) {
		return Promise.resolve();
	}
	return client.stop();
}

export function activate(context: ExtensionContext) {
	context.subscriptions.push(
		commands.registerCommand("typort-hdl.restartLanguageServer", () => {
			stopServer().then(startServer, startServer);
		})
	)

	startServer();
}

export function deactivate(): Thenable<void> {
	return stopServer();
}

