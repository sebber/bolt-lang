import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { 
    LanguageClient, 
    LanguageClientOptions, 
    ServerOptions,
    TransportKind 
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let outputChannel: vscode.OutputChannel;
let statusBarItem: vscode.StatusBarItem;

export function activate(context: vscode.ExtensionContext) {
    console.log('Bolt Language extension is activating...');
    
    // Create output channel for logging
    outputChannel = vscode.window.createOutputChannel('Bolt Language Server');
    
    // Create status bar item
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
    statusBarItem.text = '$(sync~spin) Bolt LSP Starting...';
    statusBarItem.tooltip = 'Bolt Language Server Status';
    statusBarItem.show();
    context.subscriptions.push(statusBarItem);
    
    // Register commands
    registerCommands(context);
    
    // Start the language server
    startLanguageServer(context);
    
    console.log('Bolt Language extension activated');
}

function registerCommands(context: vscode.ExtensionContext) {
    // Restart server command
    context.subscriptions.push(
        vscode.commands.registerCommand('bolt.restartServer', async () => {
            await stopLanguageServer();
            startLanguageServer(context);
            vscode.window.showInformationMessage('Bolt Language Server restarted');
        })
    );
    
    // Show output channel command
    context.subscriptions.push(
        vscode.commands.registerCommand('bolt.showOutputChannel', () => {
            outputChannel.show();
        })
    );
    
    // Compile command
    context.subscriptions.push(
        vscode.commands.registerCommand('bolt.compile', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || !editor.document.fileName.endsWith('.bolt')) {
                vscode.window.showErrorMessage('Please open a .bolt file');
                return;
            }
            
            await editor.document.save();
            const filePath = editor.document.fileName;
            const outputPath = filePath.replace('.bolt', '');
            
            const terminal = vscode.window.createTerminal('Bolt Compiler');
            terminal.show();
            terminal.sendText(`bolt "${filePath}" -o "${outputPath}"`);
        })
    );
    
    // Run command
    context.subscriptions.push(
        vscode.commands.registerCommand('bolt.run', async () => {
            const editor = vscode.window.activeTextEditor;
            if (!editor || !editor.document.fileName.endsWith('.bolt')) {
                vscode.window.showErrorMessage('Please open a .bolt file');
                return;
            }
            
            await editor.document.save();
            const filePath = editor.document.fileName;
            const outputPath = filePath.replace('.bolt', '');
            
            const terminal = vscode.window.createTerminal('Bolt Runner');
            terminal.show();
            terminal.sendText(`bolt "${filePath}" -o "${outputPath}" && "${outputPath}"`);
        })
    );
}

function findLspServer(context: vscode.ExtensionContext): string | undefined {
    // Check configuration first
    const config = vscode.workspace.getConfiguration('bolt');
    const configuredPath = config.get<string>('lspServerPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
        outputChannel.appendLine(`Using configured LSP server: ${configuredPath}`);
        return configuredPath;
    }
    
    // Search paths for the LSP server
    const searchPaths = [
        // Bundled with extension
        path.join(context.extensionPath, 'bin', 'bolt-lsp'),
        path.join(context.extensionPath, 'bin', 'bolt-lsp.exe'),
        
        // Development paths (relative to extension)
        path.join(context.extensionPath, '..', 'target', 'release', 'bolt-lsp'),
        path.join(context.extensionPath, '..', 'target', 'debug', 'bolt-lsp'),
        path.join(context.extensionPath, '..', 'bin', 'bolt-lsp'),
        
        // User installation paths
        path.join(os.homedir(), '.cargo', 'bin', 'bolt-lsp'),
        path.join(os.homedir(), '.local', 'bin', 'bolt-lsp'),
        
        // System paths
        '/usr/local/bin/bolt-lsp',
        '/usr/bin/bolt-lsp',
        
        // Windows paths
        'C:\\Program Files\\Bolt\\bolt-lsp.exe',
        'C:\\bolt\\bolt-lsp.exe',
    ];
    
    for (const searchPath of searchPaths) {
        if (fs.existsSync(searchPath)) {
            try {
                fs.accessSync(searchPath, fs.constants.X_OK);
                outputChannel.appendLine(`Found LSP server at: ${searchPath}`);
                return searchPath;
            } catch {
                outputChannel.appendLine(`Found but not executable: ${searchPath}`);
            }
        }
    }
    
    outputChannel.appendLine('LSP server not found in any search path');
    return undefined;
}

async function startLanguageServer(context: vscode.ExtensionContext) {
    const serverExecutable = findLspServer(context);
    
    if (!serverExecutable) {
        statusBarItem.text = '$(error) Bolt LSP Not Found';
        statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        vscode.window.showErrorMessage(
            'Bolt Language Server not found. Please install bolt-lsp or configure the path in settings.',
            'Open Settings'
        ).then(selection => {
            if (selection === 'Open Settings') {
                vscode.commands.executeCommand('workbench.action.openSettings', 'bolt.lspServerPath');
            }
        });
        return;
    }
    
    // Server options
    const serverOptions: ServerOptions = {
        run: {
            command: serverExecutable,
            transport: TransportKind.stdio,
            options: {
                env: { ...process.env, RUST_LOG: 'info' }
            }
        },
        debug: {
            command: serverExecutable,
            transport: TransportKind.stdio,
            options: {
                env: { ...process.env, RUST_LOG: 'debug' }
            }
        }
    };
    
    // Client options
    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'bolt' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.bolt')
        },
        outputChannel: outputChannel,
        traceOutputChannel: outputChannel,
        revealOutputChannelOn: 3, // Only on error
        initializationOptions: {
            capabilities: {
                hover: vscode.workspace.getConfiguration('bolt').get('enableHover'),
                completion: vscode.workspace.getConfiguration('bolt').get('enableCompletion'),
                diagnostics: vscode.workspace.getConfiguration('bolt').get('enableDiagnostics')
            }
        }
    };
    
    // Create and start the language client
    client = new LanguageClient(
        'boltLanguageServer',
        'Bolt Language Server',
        serverOptions,
        clientOptions
    );
    
    // Update status bar based on client state
    client.onDidChangeState(event => {
        if (event.newState === 2) { // Running
            statusBarItem.text = '$(check) Bolt LSP Ready';
            statusBarItem.backgroundColor = undefined;
            statusBarItem.tooltip = 'Bolt Language Server is running';
        } else if (event.newState === 1) { // Starting
            statusBarItem.text = '$(sync~spin) Bolt LSP Starting...';
            statusBarItem.backgroundColor = undefined;
        } else {
            statusBarItem.text = '$(error) Bolt LSP Stopped';
            statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        }
    });
    
    try {
        await client.start();
        outputChannel.appendLine('Bolt Language Server started successfully');
    } catch (error) {
        outputChannel.appendLine(`Failed to start language server: ${error}`);
        statusBarItem.text = '$(error) Bolt LSP Failed';
        statusBarItem.backgroundColor = new vscode.ThemeColor('statusBarItem.errorBackground');
        vscode.window.showErrorMessage(`Failed to start Bolt Language Server: ${error}`);
    }
}

async function stopLanguageServer(): Promise<void> {
    if (client) {
        await client.stop();
        client = undefined;
    }
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}