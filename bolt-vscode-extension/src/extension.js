const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient');
const path = require('path');
const fs = require('fs');

let client;

function activate(context) {
    console.log('Bolt LSP extension is now active!');
    
    // Try to find the bolt-lsp executable
    const possiblePaths = [
        // In the extension's bin directory (installed by our script)
        path.join(__dirname, '..', 'bin', 'bolt-lsp'),
        // Relative to extension (if bolt compiler is in same directory)
        path.join(__dirname, '..', '..', 'target', 'release', 'bolt-lsp'),
        path.join(__dirname, '..', '..', 'target', 'debug', 'bolt-lsp'),
        // System path
        'bolt-lsp',
        // Common installation paths
        path.join(process.env.HOME || '', '.local', 'bin', 'bolt-lsp'),
        '/usr/local/bin/bolt-lsp',
        '/usr/bin/bolt-lsp'
    ];
    
    let serverCommand = null;
    for (const candidatePath of possiblePaths) {
        try {
            if (fs.existsSync(candidatePath) && fs.statSync(candidatePath).isFile()) {
                serverCommand = candidatePath;
                break;
            }
        } catch (e) {
            // Continue to next candidate
        }
    }
    
    if (!serverCommand) {
        // Fallback - try the first path anyway, let it fail with a good error
        serverCommand = possiblePaths[0];
        console.warn(`Could not find bolt-lsp executable. Tried: ${possiblePaths.join(', ')}`);
    }
    
    console.log(`Using bolt-lsp at: ${serverCommand}`);
    
    const serverOptions = {
        command: serverCommand,
        args: [],
        transport: TransportKind.stdio,
        options: {
            cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd()
        }
    };
    
    // Options to control the language client
    const clientOptions = {
        documentSelector: [
            { scheme: 'file', language: 'bolt' },
            { scheme: 'untitled', language: 'bolt' }
        ],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.bolt')
        },
        outputChannelName: 'Bolt Language Server',
        revealOutputChannelOn: 4, // Never automatically reveal
        initializationOptions: {
            supportsGenericTypes: true,
            supportedFeatures: ['hover', 'completion', 'diagnostics'],
            version: "0.3.0"
        },
        middleware: {
            // Custom error handling
            handleDiagnostics: (uri, diagnostics, next) => {
                // You could filter or modify diagnostics here
                next(uri, diagnostics);
            }
        }
    };
    
    // Create the language client
    client = new LanguageClient(
        'boltLSP',
        'Bolt Language Server',
        serverOptions,
        clientOptions
    );
    
    // Handle server start errors
    client.onDidChangeState((event) => {
        if (event.newState === 3) { // Stopped
            vscode.window.showErrorMessage(
                'Bolt Language Server failed to start. Make sure bolt-lsp is installed and accessible.',
                'Show Output'
            ).then((selection) => {
                if (selection === 'Show Output') {
                    client.outputChannel.show();
                }
            });
        }
    });
    
    // Register additional commands
    context.subscriptions.push(
        vscode.commands.registerCommand('bolt.restartLSP', async () => {
            if (client) {
                vscode.window.showInformationMessage('Restarting Bolt Language Server...');
                try {
                    await client.stop();
                    await client.start();
                    vscode.window.showInformationMessage('Bolt Language Server restarted');
                } catch (error) {
                    vscode.window.showErrorMessage(`Failed to restart LSP: ${error.message}`);
                }
            }
        })
    );
    
    // Add command to palette
    context.subscriptions.push(
        vscode.commands.registerCommand('bolt.showLSPOutput', () => {
            client.outputChannel.show();
        })
    );
    
    // Start the client. This will also launch the server
    context.subscriptions.push(client.start());
    
    // Show a message when LSP is ready
    client.onReady().then(() => {
        console.log('Bolt Language Server is ready!');
        vscode.window.showInformationMessage('Bolt Language Server is ready!');
    }).catch((error) => {
        console.error('Failed to start Bolt Language Server:', error);
        vscode.window.showErrorMessage(
            `Failed to start Bolt Language Server: ${error.message}`,
            'Show Output'
        ).then((selection) => {
            if (selection === 'Show Output') {
                client.outputChannel.show();
            }
        });
    });
}

function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

module.exports = {
    activate,
    deactivate
};