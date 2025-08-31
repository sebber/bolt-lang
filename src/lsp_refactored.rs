// Refactored LSP Server - More stable and maintainable architecture
use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::Arc;

mod ast;
mod error;
mod lexer;
mod lsp_core;
mod module;
mod parser;
mod symbol_table;
mod type_checker;

use lsp_core::{LogLevel, Logger, LspConfig, LspCore, LspError, LspMessage};

// Request handlers organized by functionality
struct LspRequestHandlers {
    core: Arc<LspCore>,
}

impl LspRequestHandlers {
    fn new(core: Arc<LspCore>) -> Self {
        Self { core }
    }

    // Initialize LSP capabilities
    fn handle_initialize(&self, _params: Option<&Value>) -> Value {
        self.core.logger.info("LSP server initializing...");

        json!({
            "capabilities": {
                "textDocumentSync": {
                    "openClose": true,
                    "change": 1, // Full document sync
                    "willSave": false,
                    "willSaveWaitUntil": false,
                    "save": {
                        "includeText": true
                    }
                },
                "hoverProvider": true,
                "completionProvider": {
                    "resolveProvider": false,
                    "triggerCharacters": [".", "(", "{", ":", " "]
                },
                "definitionProvider": false, // TODO: Implement
                "referencesProvider": false, // TODO: Implement
                "documentHighlightProvider": false, // TODO: Implement
                "documentSymbolProvider": false, // TODO: Implement
                "workspaceSymbolProvider": false, // TODO: Implement
                "codeActionProvider": false, // TODO: Implement
                "codeLensProvider": false, // TODO: Implement
                "documentFormattingProvider": false, // TODO: Implement
                "documentRangeFormattingProvider": false, // TODO: Implement
                "documentOnTypeFormattingProvider": null,
                "renameProvider": false, // TODO: Implement
                "foldingRangeProvider": false, // TODO: Implement
                "selectionRangeProvider": false // TODO: Implement
            },
            "serverInfo": {
                "name": "Bolt Language Server",
                "version": "0.5.0"
            }
        })
    }

    // Handle text document lifecycle
    fn handle_did_open(&self, params: Option<&Value>) -> Result<(), LspError> {
        let params =
            params.ok_or_else(|| LspError::InvalidRequest("Missing params".to_string()))?;

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidRequest("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid URI".to_string()))?
            .to_string();

        let text = text_document
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid text content".to_string()))?
            .to_string();

        let version = text_document
            .get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        self.core.open_document(uri, text, version)?;
        Ok(())
    }

    fn handle_did_change(&self, params: Option<&Value>) -> Result<(), LspError> {
        let params =
            params.ok_or_else(|| LspError::InvalidRequest("Missing params".to_string()))?;

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidRequest("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid URI".to_string()))?
            .to_string();

        let version = text_document
            .get("version")
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as i32;

        let content_changes = params
            .get("contentChanges")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                LspError::InvalidRequest("Missing or invalid contentChanges".to_string())
            })?;

        // For full document sync, we expect one change with the full text
        if let Some(change) = content_changes.first() {
            if let Some(text) = change.get("text").and_then(|v| v.as_str()) {
                self.core.update_document(uri, text.to_string(), version)?;
            }
        }

        Ok(())
    }

    fn handle_did_close(&self, params: Option<&Value>) -> Result<(), LspError> {
        let params =
            params.ok_or_else(|| LspError::InvalidRequest("Missing params".to_string()))?;

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidRequest("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid URI".to_string()))?;

        self.core.close_document(uri)?;
        Ok(())
    }

    // Handle hover requests
    fn handle_hover(&self, params: Option<&Value>) -> Result<Value, LspError> {
        let params =
            params.ok_or_else(|| LspError::InvalidRequest("Missing params".to_string()))?;

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidRequest("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid URI".to_string()))?;

        let position = params
            .get("position")
            .ok_or_else(|| LspError::InvalidRequest("Missing position".to_string()))?;

        let line = position
            .get("line")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid line".to_string()))?
            as usize;

        let character = position
            .get("character")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid character".to_string()))?
            as usize;

        // Get the word at position (simplified - should use proper word boundary detection)
        let word = self.extract_word_at_position(uri, line, character)?;

        if let Some(type_info) = self.core.get_type_info(uri, &word, line)? {
            Ok(json!({
                "contents": {
                    "kind": "markdown",
                    "value": type_info
                }
            }))
        } else {
            // Fallback to static hover information
            let static_info = self.get_static_hover_info(&word);
            Ok(json!({
                "contents": {
                    "kind": "markdown",
                    "value": static_info
                }
            }))
        }
    }

    // Handle completion requests
    fn handle_completion(&self, params: Option<&Value>) -> Result<Value, LspError> {
        let params =
            params.ok_or_else(|| LspError::InvalidRequest("Missing params".to_string()))?;

        let text_document = params
            .get("textDocument")
            .ok_or_else(|| LspError::InvalidRequest("Missing textDocument".to_string()))?;

        let uri = text_document
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid URI".to_string()))?;

        let position = params
            .get("position")
            .ok_or_else(|| LspError::InvalidRequest("Missing position".to_string()))?;

        let line = position
            .get("line")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid line".to_string()))?
            as usize;

        let character = position
            .get("character")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| LspError::InvalidRequest("Missing or invalid character".to_string()))?
            as usize;

        let completions = self.core.get_completions(uri, line, character)?;
        Ok(json!(completions))
    }

    // Helper methods
    fn extract_word_at_position(
        &self,
        uri: &str,
        line: usize,
        character: usize,
    ) -> Result<String, LspError> {
        let documents =
            self.core.documents.read().map_err(|_| {
                LspError::InternalError("Failed to acquire documents lock".to_string())
            })?;

        let document = documents
            .get(uri)
            .ok_or_else(|| LspError::DocumentNotFound(uri.to_string()))?;

        let lines: Vec<&str> = document.content.lines().collect();

        if line >= lines.len() {
            return Ok("".to_string());
        }

        let line_text = lines[line];

        if character >= line_text.len() {
            return Ok("".to_string());
        }

        // Find word boundaries around the character position
        let chars: Vec<char> = line_text.chars().collect();

        if character >= chars.len() {
            return Ok("".to_string());
        }

        // Check if we're on a valid identifier character
        if !chars[character].is_alphanumeric() && chars[character] != '_' {
            return Ok("".to_string());
        }

        // Find start of word
        let mut start = character;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Find end of word
        let mut end = character;
        while end < chars.len() - 1 && (chars[end + 1].is_alphanumeric() || chars[end + 1] == '_') {
            end += 1;
        }

        // Extract the word
        let word: String = chars[start..=end].iter().collect();
        Ok(word)
    }

    fn get_static_hover_info(&self, word: &str) -> String {
        match word {
            "print" => {
                "**`print(value)`**\n\n*Built-in function*\n\nPrints a value to the console."
                    .to_string()
            }
            "val" => {
                "**`val`**\n\n*Keyword*\n\nDeclares an immutable variable with type inference."
                    .to_string()
            }
            "var" => "**`var`**\n\n*Keyword*\n\nDeclares a mutable variable with type inference."
                .to_string(),
            "fun" => {
                "**`fun`**\n\n*Keyword*\n\nDefines a function with parameters and return type."
                    .to_string()
            }
            "Integer" => {
                "**`Integer`**\n\n*Built-in type*\n\n64-bit signed integer type.".to_string()
            }
            "String" => "**`String`**\n\n*Built-in type*\n\nUTF-8 string type.".to_string(),
            "Bool" => {
                "**`Bool`**\n\n*Built-in type*\n\nBoolean type with values `true` and `false`."
                    .to_string()
            }
            _ => format!("**`{}`**", word),
        }
    }
}

// Main LSP server
pub struct RefactoredLspServer {
    handlers: LspRequestHandlers,
    logger: Arc<dyn Logger>,
}

impl RefactoredLspServer {
    pub fn new() -> Self {
        let config = LspConfig::default();
        let logger = Arc::new(lsp_core::StderrLogger);
        let core = Arc::new(LspCore::new(config, logger.clone()));
        let handlers = LspRequestHandlers::new(core);

        Self { handlers, logger }
    }

    pub fn run(&mut self) {
        self.logger.info("Bolt Language Server starting...");

        let stdin = io::stdin();
        let mut stdin_lock = stdin.lock();
        let mut stdout = io::stdout();

        loop {
            match self.read_message(&mut stdin_lock) {
                Ok(Some(message)) => {
                    if let Some(response) = self.handle_message(message) {
                        if let Err(e) = self.send_message(&mut stdout, response) {
                            self.logger
                                .error(&format!("Failed to send response: {}", e));
                        }
                    }
                }
                Ok(None) => {
                    // EOF reached
                    self.logger.info("LSP server shutting down...");
                    break;
                }
                Err(e) => {
                    self.logger.error(&format!("Failed to read message: {}", e));
                    break;
                }
            }
        }
    }

    fn read_message(
        &self,
        reader: &mut dyn BufRead,
    ) -> Result<Option<LspMessage>, Box<dyn std::error::Error>> {
        let mut headers = HashMap::new();
        let mut line = String::new();

        // Read headers
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Ok(None); // EOF
            }

            let line = line.trim();
            if line.is_empty() {
                break; // End of headers
            }

            if let Some((key, value)) = line.split_once(": ") {
                headers.insert(key.to_lowercase(), value.to_string());
            }
        }

        // Get content length
        let content_length = headers
            .get("content-length")
            .and_then(|s| s.parse::<usize>().ok())
            .ok_or("Missing or invalid Content-Length header")?;

        // Read content
        let mut content = vec![0; content_length];
        reader.read_exact(&mut content)?;

        let content_str = String::from_utf8(content)?;
        let message: LspMessage = serde_json::from_str(&content_str)?;

        Ok(Some(message))
    }

    fn send_message(
        &self,
        writer: &mut dyn Write,
        message: LspMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string(&message)?;
        let content_length = content.len();

        write!(
            writer,
            "Content-Length: {}\r\n\r\n{}",
            content_length, content
        )?;
        writer.flush()?;

        Ok(())
    }

    fn handle_message(&self, message: LspMessage) -> Option<LspMessage> {
        let method = message.method.as_ref()?;

        self.logger.debug(&format!("Handling request: {}", method));

        match method.as_str() {
            "initialize" => {
                let result = self.handlers.handle_initialize(message.params.as_ref());
                Some(LspMessage::new_response(message.id, result))
            }

            "initialized" => {
                self.logger.info("LSP server initialized successfully");
                None // No response needed
            }

            "textDocument/didOpen" => {
                match self.handlers.handle_did_open(message.params.as_ref()) {
                    Ok(_) => None,
                    Err(e) => {
                        self.logger
                            .error(&format!("Failed to handle didOpen: {}", e));
                        None
                    }
                }
            }

            "textDocument/didChange" => {
                match self.handlers.handle_did_change(message.params.as_ref()) {
                    Ok(_) => None,
                    Err(e) => {
                        self.logger
                            .error(&format!("Failed to handle didChange: {}", e));
                        None
                    }
                }
            }

            "textDocument/didClose" => {
                match self.handlers.handle_did_close(message.params.as_ref()) {
                    Ok(_) => None,
                    Err(e) => {
                        self.logger
                            .error(&format!("Failed to handle didClose: {}", e));
                        None
                    }
                }
            }

            "textDocument/hover" => match self.handlers.handle_hover(message.params.as_ref()) {
                Ok(result) => Some(LspMessage::new_response(message.id, result)),
                Err(e) => Some(LspMessage::new_error(message.id, e)),
            },

            "textDocument/completion" => {
                match self.handlers.handle_completion(message.params.as_ref()) {
                    Ok(result) => Some(LspMessage::new_response(message.id, result)),
                    Err(e) => Some(LspMessage::new_error(message.id, e)),
                }
            }

            "shutdown" => {
                self.logger.info("Received shutdown request");
                Some(LspMessage::new_response(message.id, json!(null)))
            }

            "exit" => {
                self.logger.info("Received exit notification");
                std::process::exit(0);
            }

            _ => {
                self.logger.warn(&format!("Unhandled method: {}", method));
                let error = LspError::InvalidRequest(format!("Method not found: {}", method));
                Some(LspMessage::new_error(message.id, error))
            }
        }
    }
}

use std::collections::HashMap;

fn main() {
    let mut server = RefactoredLspServer::new();
    server.run();
}
