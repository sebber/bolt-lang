// Refactored VS Code Extension - More robust and maintainable
const vscode = require('vscode');
const { LanguageClient, TransportKind } = require('vscode-languageclient');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { spawn } = require('child_process');

let client;
let outputChannel;
let statusBarItem;

// Configuration and state management
class ExtensionConfig {
    constructor() {
        this.lspExecutablePaths = this.getLspExecutablePaths();
        this.compilerPaths = this.getCompilerPaths();
        this.maxTerminals = 3;
        this.activeTerminals = new Set();
    }

    getLspExecutablePaths() {
        const extensionPath = path.dirname(__dirname);
        return [
            // Extension's bin directory (installed by script)
            path.join(extensionPath, 'bin', 'bolt-lsp'),
            path.join(extensionPath, 'bin', 'bolt-lsp.exe'),
            
            // Relative to extension (development)
            path.join(extensionPath, '..', 'target', 'release', 'bolt-lsp'),
            path.join(extensionPath, '..', 'target', 'debug', 'bolt-lsp'),
            
            // User's local bin
            path.join(os.homedir(), '.local', 'bin', 'bolt-lsp'),
            path.join(os.homedir(), '.cargo', 'bin', 'bolt-lsp'),
            
            // System paths
            '/usr/local/bin/bolt-lsp',
            '/usr/bin/bolt-lsp',
            'bolt-lsp' // PATH lookup
        ];
    }

    getCompilerPaths() {
        const extensionPath = path.dirname(__dirname);
        return [
            // Extension's bin directory
            path.join(extensionPath, 'bin', 'bolt'),
            path.join(extensionPath, 'bin', 'bolt.exe'),
            
            // Relative to extension (development)
            path.join(extensionPath, '..', 'target', 'release', 'bolt'),
            path.join(extensionPath, '..', 'target', 'debug', 'bolt'),
            
            // User's local bin
            path.join(os.homedir(), '.local', 'bin', 'bolt'),
            path.join(os.homedir(), '.cargo', 'bin', 'bolt'),
            
            // System paths
            '/usr/local/bin/bolt',
            '/usr/bin/bolt',
            'bolt' // PATH lookup
        ];
    }
}

// Error handling and logging
class ExtensionLogger {
    constructor(outputChannel) {
        this.channel = outputChannel;
    }

    debug(message) {
        this.channel.appendLine(`[DEBUG] ${new Date().toISOString()}: ${message}`);
    }

    info(message) {
        this.channel.appendLine(`[INFO] ${new Date().toISOString()}: ${message}`);
        console.log(`Bolt Extension: ${message}`);
    }

    warn(message) {
        this.channel.appendLine(`[WARN] ${new Date().toISOString()}: ${message}`);
        console.warn(`Bolt Extension: ${message}`);
    }

    error(message, error = null) {
        const errorMsg = error ? `${message}: ${error.message}` : message;
        this.channel.appendLine(`[ERROR] ${new Date().toISOString()}: ${errorMsg}`);
        console.error(`Bolt Extension: ${errorMsg}`, error);
    }
}

// Utility functions
class ExtensionUtils {
    static async findExecutable(paths, logger) {
        for (const candidatePath of paths) {
            try {
                if (await this.isExecutableFile(candidatePath)) {
                    logger.info(`Found executable at: ${candidatePath}`);
                    return candidatePath;
                }
            } catch (error) {
                logger.debug(`Failed to check executable at ${candidatePath}: ${error.message}`);
            }
        }
        return null;
    }

    static async isExecutableFile(filePath) {
        return new Promise((resolve) => {
            fs.access(filePath, fs.constants.F_OK | fs.constants.X_OK, (err) => {
                resolve(!err);
            });
        });
    }

    static async testExecutable(executablePath, args = ['--version'], logger) {
        return new Promise((resolve) => {
            const process = spawn(executablePath, args, { 
                stdio: 'pipe',
                timeout: 5000 
            });
            
            let output = '';
            process.stdout.on('data', (data) => {
                output += data.toString();
            });

            process.stderr.on('data', (data) => {
                output += data.toString();
            });

            process.on('close', (code) => {
                const success = code === 0 || output.length > 0;
                logger.debug(`Executable test for ${executablePath}: ${success ? 'passed' : 'failed'} (code: ${code})`);
                resolve(success);
            });

            process.on('error', (error) => {
                logger.debug(`Executable test error for ${executablePath}: ${error.message}`);
                resolve(false);
            });
        });
    }

    static getWorkspaceRoot() {
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (workspaceFolders && workspaceFolders.length > 0) {
            return workspaceFolders[0].uri.fsPath;
        }
        return process.cwd();
    }
}

// LSP Client management
class LspClientManager {
    constructor(config, logger) {
        this.config = config;
        this.logger = logger;
        this.client = null;
        this.isStarting = false;
        this.restartCount = 0;
        this.maxRestarts = 3;
    }

    async initialize() {
        if (this.isStarting) {
            this.logger.warn('LSP client is already starting');
            return false;
        }

        this.isStarting = true;

        try {
            const serverCommand = await ExtensionUtils.findExecutable(
                this.config.lspExecutablePaths, 
                this.logger
            );

            if (!serverCommand) {
                throw new Error('Could not find bolt-lsp executable');
            }

            // Test the executable before using it
            const isWorking = await ExtensionUtils.testExecutable(serverCommand, [], this.logger);
            if (!isWorking) {
                throw new Error(`bolt-lsp executable at ${serverCommand} is not working`);
            }

            const serverOptions = {
                command: serverCommand,
                args: [],
                transport: TransportKind.stdio,
                options: {
                    cwd: ExtensionUtils.getWorkspaceRoot(),
                    env: { ...process.env }
                }
            };

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
                    supportsTypeInference: true,
                    supportsTypeAwareCompletion: true,
                    supportsRichHover: true,
                    supportedFeatures: ['hover', 'completion', 'diagnostics', 'typeInformation'],
                    version: "0.5.0"
                },
                middleware: {
                    handleDiagnostics: (uri, diagnostics, next) => {
                        // Filter out diagnostics if needed
                        const filteredDiagnostics = diagnostics.filter(diag => {
                            // Could add filtering logic here
                            return true;
                        });
                        next(uri, filteredDiagnostics);
                    }
                }
            };

            this.client = new LanguageClient(
                'boltLSP',
                'Bolt Language Server',
                serverOptions,
                clientOptions
            );

            this.setupClientEventHandlers();
            await this.client.start();
            
            this.logger.info('LSP client started successfully');
            this.restartCount = 0;
            return true;

        } catch (error) {
            this.logger.error('Failed to initialize LSP client', error);
            this.showLspError(error.message);
            return false;
        } finally {
            this.isStarting = false;
        }
    }

    setupClientEventHandlers() {
        if (!this.client) return;

        this.client.onDidChangeState((event) => {
            this.logger.debug(`LSP client state changed: ${event.oldState} -> ${event.newState}`);
            
            if (event.newState === 1) { // Starting
                this.updateStatusBar('$(loading~spin) Starting Bolt LSP...');
            } else if (event.newState === 2) { // Running
                this.updateStatusBar('$(check) Bolt LSP Ready');
            } else if (event.newState === 3) { // Stopped
                this.updateStatusBar('$(error) Bolt LSP Stopped');
                this.handleClientStopped();
            }
        });

        this.client.onReady().then(() => {
            this.logger.info('LSP client is ready');
            vscode.window.showInformationMessage('Bolt Language Server is ready!');
        }).catch((error) => {
            this.logger.error('LSP client failed to become ready', error);
            this.showLspError(`Failed to start Language Server: ${error.message}`);
        });
    }

    async handleClientStopped() {
        if (this.restartCount < this.maxRestarts) {
            this.restartCount++;
            this.logger.warn(`LSP client stopped. Attempting restart ${this.restartCount}/${this.maxRestarts}`);
            
            // Wait a bit before restarting
            setTimeout(() => {
                this.restart();
            }, 2000);
        } else {
            this.logger.error('LSP client stopped and max restarts exceeded');
            this.showLspError('Language Server stopped and could not be restarted');
        }
    }

    async restart() {
        this.logger.info('Restarting LSP client...');
        
        try {
            if (this.client) {
                await this.client.stop();
                this.client = null;
            }
            
            const success = await this.initialize();
            if (success) {
                vscode.window.showInformationMessage('Bolt Language Server restarted successfully');
            }
        } catch (error) {
            this.logger.error('Failed to restart LSP client', error);
            this.showLspError(`Failed to restart Language Server: ${error.message}`);
        }
    }

    async stop() {
        if (this.client) {
            try {
                await this.client.stop();
                this.client = null;
                this.logger.info('LSP client stopped');
            } catch (error) {
                this.logger.error('Error stopping LSP client', error);
            }
        }
    }

    showLspError(message) {
        vscode.window.showErrorMessage(
            message,
            'Show Output',
            'Retry'
        ).then((selection) => {
            if (selection === 'Show Output') {
                outputChannel.show();
            } else if (selection === 'Retry') {
                this.restart();
            }
        });
    }

    updateStatusBar(text) {
        if (statusBarItem) {
            statusBarItem.text = text;
            statusBarItem.show();
        }
    }
}

// Command handlers
class CommandHandlers {
    constructor(config, logger, lspManager) {
        this.config = config;
        this.logger = logger;
        this.lspManager = lspManager;
    }

    async compileFile() {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bolt') {
            vscode.window.showWarningMessage('Please open a .bolt file to compile');
            return;
        }

        const filePath = editor.document.fileName;
        const workspaceRoot = ExtensionUtils.getWorkspaceRoot();
        const outputName = path.basename(filePath, '.bolt');

        // Save the file first
        if (editor.document.isDirty) {
            await editor.document.save();
        }

        const compiler = await ExtensionUtils.findExecutable(
            this.config.compilerPaths,
            this.logger
        );

        if (!compiler) {
            vscode.window.showErrorMessage(
                'Bolt compiler not found. Please install the Bolt compiler.',
                'Show Output'
            ).then((selection) => {
                if (selection === 'Show Output') {
                    outputChannel.show();
                }
            });
            return;
        }

        const terminal = this.getOrCreateTerminal('Bolt Compiler');
        terminal.sendText(`"${compiler}" "${filePath}" -o "${outputName}"`);
        terminal.show();
    }

    async runFile() {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bolt') {
            vscode.window.showWarningMessage('Please open a .bolt file to run');
            return;
        }

        const filePath = editor.document.fileName;
        const workspaceRoot = ExtensionUtils.getWorkspaceRoot();
        const outputName = path.basename(filePath, '.bolt');

        // Save the file first
        if (editor.document.isDirty) {
            await editor.document.save();
        }

        const compiler = await ExtensionUtils.findExecutable(
            this.config.compilerPaths,
            this.logger
        );

        if (!compiler) {
            vscode.window.showErrorMessage('Bolt compiler not found');
            return;
        }

        const terminal = this.getOrCreateTerminal('Bolt Run');
        const executablePath = path.join('out', 'debug', outputName);
        terminal.sendText(`"${compiler}" "${filePath}" -o "${outputName}" && "${executablePath}"`);
        terminal.show();
    }

    async showTypeInfo() {
        const editor = vscode.window.activeTextEditor;
        if (!editor || editor.document.languageId !== 'bolt') {
            vscode.window.showWarningMessage('Please open a .bolt file to show type information');
            return;
        }

        const position = editor.selection.active;
        const document = editor.document;
        const wordRange = document.getWordRangeAtPosition(position);
        
        if (!wordRange) {
            vscode.window.showInformationMessage('No word found at cursor position');
            return;
        }

        const word = document.getText(wordRange);
        
        try {
            const hover = await vscode.commands.executeCommand('vscode.executeHoverProvider', 
                document.uri, position);
            
            if (hover && hover.length > 0) {
                const hoverContent = hover[0].contents;
                let typeInfo = '';
                
                if (Array.isArray(hoverContent)) {
                    typeInfo = hoverContent
                        .map(c => c.value || c.toString())
                        .join('\n');
                } else {
                    typeInfo = hoverContent.value || hoverContent.toString();
                }
                
                if (typeInfo && typeInfo.trim()) {
                    const cleanInfo = typeInfo
                        .replace(/\*\*/g, '')
                        .replace(/`/g, '')
                        .replace(/\n+/g, ' ')
                        .trim();
                    vscode.window.showInformationMessage(`${word}: ${cleanInfo}`);
                } else {
                    vscode.window.showInformationMessage(`No type information available for '${word}'`);
                }
            } else {
                vscode.window.showInformationMessage(`No type information available for '${word}'`);
            }
        } catch (error) {
            this.logger.error('Failed to get type information', error);
            vscode.window.showErrorMessage(`Failed to get type information: ${error.message}`);
        }
    }

    getOrCreateTerminal(name) {
        // Clean up old terminals if we have too many
        if (this.config.activeTerminals.size >= this.config.maxTerminals) {
            const oldestTerminal = this.config.activeTerminals.values().next().value;
            if (oldestTerminal) {
                oldestTerminal.dispose();
                this.config.activeTerminals.delete(oldestTerminal);
            }
        }

        const terminal = vscode.window.createTerminal({
            name,
            cwd: ExtensionUtils.getWorkspaceRoot()
        });

        this.config.activeTerminals.add(terminal);

        // Clean up when terminal is disposed
        const disposable = vscode.window.onDidCloseTerminal((closedTerminal) => {
            if (closedTerminal === terminal) {
                this.config.activeTerminals.delete(terminal);
                disposable.dispose();
            }
        });

        return terminal;
    }
}

// Main activation function
async function activate(context) {
    console.log('Bolt LSP extension is now active!');

    // Initialize components
    outputChannel = vscode.window.createOutputChannel('Bolt Extension');
    const logger = new ExtensionLogger(outputChannel);
    const config = new ExtensionConfig();
    
    // Status bar
    statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    statusBarItem.text = '$(loading~spin) Starting Bolt LSP...';
    statusBarItem.tooltip = 'Bolt Language Server Status';
    statusBarItem.command = 'bolt.showLSPOutput';
    
    // LSP Manager
    const lspManager = new LspClientManager(config, logger);
    const commandHandlers = new CommandHandlers(config, logger, lspManager);

    // Register commands
    const commands = [
        vscode.commands.registerCommand('bolt.restartLSP', async () => {
            try {
                await lspManager.restart();
            } catch (error) {
                logger.error('Failed to restart LSP', error);
            }
        }),
        
        vscode.commands.registerCommand('bolt.showLSPOutput', () => {
            outputChannel.show();
        }),
        
        vscode.commands.registerCommand('bolt.compileFile', async () => {
            await commandHandlers.compileFile();
        }),
        
        vscode.commands.registerCommand('bolt.runFile', async () => {
            await commandHandlers.runFile();
        }),
        
        vscode.commands.registerCommand('bolt.showTypeInfo', async () => {
            await commandHandlers.showTypeInfo();
        })
    ];

    // Add all commands to context subscriptions
    context.subscriptions.push(...commands, statusBarItem, outputChannel);

    // Initialize LSP client
    const success = await lspManager.initialize();
    if (!success) {
        logger.warn('LSP initialization failed, but extension will continue running');
    }

    // Store LSP manager for deactivation
    context.subscriptions.push({
        dispose: async () => {
            await lspManager.stop();
        }
    });

    logger.info('Bolt extension activated successfully');
}

async function deactivate() {
    if (client) {
        try {
            await client.stop();
        } catch (error) {
            console.error('Error stopping LSP client:', error);
        }
    }
    
    if (outputChannel) {
        outputChannel.dispose();
    }
    
    if (statusBarItem) {
        statusBarItem.dispose();
    }
}

module.exports = {
    activate,
    deactivate
};