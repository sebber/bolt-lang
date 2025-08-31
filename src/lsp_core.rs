// LSP Core - Refactored LSP server with better architecture
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::sync::{Arc, RwLock};

use crate::ast::Program;
use crate::lexer::Lexer;
use crate::module::ModuleSystem;
use crate::parser::Parser;
use crate::type_checker::TypeChecker;

// Logging levels for better error handling
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// Enhanced error types for LSP operations
#[derive(Debug, Clone)]
pub enum LspError {
    ParseError(String),
    TypeError(String),
    DocumentNotFound(String),
    InvalidRequest(String),
    InternalError(String),
}

impl std::fmt::Display for LspError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LspError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            LspError::TypeError(msg) => write!(f, "Type Error: {}", msg),
            LspError::DocumentNotFound(msg) => write!(f, "Document Not Found: {}", msg),
            LspError::InvalidRequest(msg) => write!(f, "Invalid Request: {}", msg),
            LspError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}

impl std::error::Error for LspError {}

type LspResult<T> = Result<T, LspError>;

// Document state management
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub content: String,
    pub version: i32,
    pub uri: String,
    pub parsed_ast: Option<Program>,
    pub type_info: Option<HashMap<String, String>>,
    pub last_updated: std::time::Instant,
}

impl DocumentState {
    pub fn new(uri: String, content: String, version: i32) -> Self {
        Self {
            content,
            version,
            uri,
            parsed_ast: None,
            type_info: None,
            last_updated: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self, content: String, version: i32) {
        self.content = content;
        self.version = version;
        self.parsed_ast = None;
        self.type_info = None;
        self.last_updated = std::time::Instant::now();
    }
}

// LSP Message types
#[derive(Debug, Serialize, Deserialize)]
pub struct LspMessage {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

impl LspMessage {
    pub fn new_response(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    pub fn new_error(id: Option<Value>, error: LspError) -> Self {
        let error_obj = json!({
            "code": -32603, // Internal error
            "message": error.to_string()
        });

        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: None,
            error: Some(error_obj),
        }
    }

    pub fn new_notification(method: String, params: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some(method),
            params: Some(params),
            result: None,
            error: None,
        }
    }
}

// Configuration for LSP server
#[derive(Debug, Clone)]
pub struct LspConfig {
    pub max_documents: usize,
    pub document_cleanup_interval: std::time::Duration,
    pub type_checking_enabled: bool,
    pub diagnostics_enabled: bool,
    pub completion_enabled: bool,
    pub hover_enabled: bool,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            max_documents: 100,
            document_cleanup_interval: std::time::Duration::from_secs(300), // 5 minutes
            type_checking_enabled: true,
            diagnostics_enabled: false, // Disabled by default for compatibility
            completion_enabled: true,
            hover_enabled: true,
        }
    }
}

// Logger trait for dependency injection
pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
    fn debug(&self, message: &str) {
        self.log(LogLevel::Debug, message);
    }
    fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }
    fn warn(&self, message: &str) {
        self.log(LogLevel::Warn, message);
    }
    fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }
}

// Simple stderr logger implementation
pub struct StderrLogger;

impl Logger for StderrLogger {
    fn log(&self, level: LogLevel, message: &str) {
        eprintln!("[{:?}] {}", level, message);
    }
}

// Core LSP Server with improved architecture
pub struct LspCore {
    pub config: LspConfig,
    pub documents: Arc<RwLock<HashMap<String, DocumentState>>>,
    pub type_checker: Arc<RwLock<TypeChecker>>,
    pub module_system: Arc<RwLock<ModuleSystem>>,
    pub logger: Arc<dyn Logger>,
}

impl LspCore {
    pub fn new(config: LspConfig, logger: Arc<dyn Logger>) -> Self {
        let module_system = ModuleSystem::new();
        let type_checker = TypeChecker::new().with_module_system(module_system.clone());

        Self {
            config,
            documents: Arc::new(RwLock::new(HashMap::new())),
            type_checker: Arc::new(RwLock::new(type_checker)),
            module_system: Arc::new(RwLock::new(module_system)),
            logger,
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(LspConfig::default(), Arc::new(StderrLogger))
    }

    // Document management
    pub fn open_document(&self, uri: String, content: String, version: i32) -> LspResult<()> {
        let mut documents = self
            .documents
            .write()
            .map_err(|_| LspError::InternalError("Failed to acquire documents lock".to_string()))?;

        // Cleanup old documents if we're at the limit
        if documents.len() >= self.config.max_documents {
            self.cleanup_old_documents(&mut documents);
        }

        let document = DocumentState::new(uri.clone(), content, version);
        documents.insert(uri.clone(), document);

        self.logger.info(&format!("Opened document: {}", uri));
        Ok(())
    }

    pub fn update_document(&self, uri: String, content: String, version: i32) -> LspResult<()> {
        let mut documents = self
            .documents
            .write()
            .map_err(|_| LspError::InternalError("Failed to acquire documents lock".to_string()))?;

        match documents.get_mut(&uri) {
            Some(document) => {
                document.update(content, version);
                self.logger
                    .debug(&format!("Updated document: {} (version {})", uri, version));
                Ok(())
            }
            None => Err(LspError::DocumentNotFound(uri)),
        }
    }

    pub fn close_document(&self, uri: &str) -> LspResult<()> {
        let mut documents = self
            .documents
            .write()
            .map_err(|_| LspError::InternalError("Failed to acquire documents lock".to_string()))?;

        documents.remove(uri);
        self.logger.info(&format!("Closed document: {}", uri));
        Ok(())
    }

    fn cleanup_old_documents(&self, documents: &mut HashMap<String, DocumentState>) {
        let cutoff = std::time::Instant::now() - self.config.document_cleanup_interval;
        let to_remove: Vec<String> = documents
            .iter()
            .filter(|(_, doc)| doc.last_updated < cutoff)
            .take(documents.len() / 4) // Remove 25% of documents
            .map(|(uri, _)| uri.clone())
            .collect();

        for uri in to_remove {
            documents.remove(&uri);
            self.logger
                .debug(&format!("Cleaned up old document: {}", uri));
        }
    }

    // Parse document and cache AST
    fn ensure_parsed(&self, uri: &str) -> LspResult<()> {
        let mut documents = self
            .documents
            .write()
            .map_err(|_| LspError::InternalError("Failed to acquire documents lock".to_string()))?;

        let document = documents
            .get_mut(uri)
            .ok_or_else(|| LspError::DocumentNotFound(uri.to_string()))?;

        if document.parsed_ast.is_none() {
            match self.parse_document_content(&document.content) {
                Ok(ast) => {
                    document.parsed_ast = Some(ast);
                    self.logger.debug(&format!("Parsed document: {}", uri));
                }
                Err(e) => {
                    self.logger.warn(&format!("Failed to parse {}: {}", uri, e));
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn parse_document_content(&self, content: &str) -> LspResult<Program> {
        let mut lexer = Lexer::new(content.to_string());
        let tokens = lexer
            .tokenize()
            .map_err(|e| LspError::ParseError(format!("Lexer error: {}", e)))?;

        let mut parser = Parser::new(tokens);
        let ast = parser
            .parse()
            .map_err(|e| LspError::ParseError(format!("Parser error: {}", e)))?;

        Ok(ast)
    }

    // Get cached or compute type information
    pub fn get_type_info(&self, uri: &str, word: &str, line: usize) -> LspResult<Option<String>> {
        self.ensure_parsed(uri)?;

        let documents = self
            .documents
            .read()
            .map_err(|_| LspError::InternalError("Failed to acquire documents lock".to_string()))?;

        let document = documents
            .get(uri)
            .ok_or_else(|| LspError::DocumentNotFound(uri.to_string()))?;

        if let Some(ast) = &document.parsed_ast {
            if self.config.type_checking_enabled {
                let type_checker = self.type_checker.read().map_err(|_| {
                    LspError::InternalError("Failed to acquire type checker lock".to_string())
                })?;

                // Try to get function signature
                if let Some(func_sig) = type_checker.get_function_signature(word) {
                    let params_str: Vec<String> = func_sig
                        .params
                        .iter()
                        .map(|(name, ty)| format!("{}: {:?}", name, ty))
                        .collect();

                    let return_str = func_sig
                        .return_type
                        .as_ref()
                        .map(|t| format!("{:?}", t))
                        .unwrap_or_else(|| "void".to_string());

                    let func_type = if func_sig.is_native {
                        "Native Function"
                    } else if func_sig.is_extern {
                        "Extern Function"
                    } else {
                        "Function"
                    };

                    let info = format!(
                        "**`{}({}): {}`**\n\n*{}*",
                        word,
                        params_str.join(", "),
                        return_str,
                        func_type
                    );

                    return Ok(Some(info));
                }

                // Try to get variable type
                if let Some(var_type) = type_checker.get_variable_type(word) {
                    let info = format!("**`{}: {:?}`**\n\n*Variable*", word, var_type);

                    return Ok(Some(info));
                }
            }
        }

        Ok(None)
    }

    // Get completion items with type information
    pub fn get_completions(
        &self,
        uri: &str,
        _line: usize,
        _character: usize,
    ) -> LspResult<Vec<Value>> {
        if !self.config.completion_enabled {
            return Ok(vec![]);
        }

        let mut items = Vec::new();

        // Add basic language keywords
        items.extend(self.get_keyword_completions());

        // Add type-aware completions if available
        if self.config.type_checking_enabled {
            if let Ok(type_completions) = self.get_type_aware_completions(uri) {
                items.extend(type_completions);
            }
        }

        // Add standard library completions
        items.extend(self.get_stdlib_completions());

        Ok(items)
    }

    fn get_keyword_completions(&self) -> Vec<Value> {
        vec![
            json!({"label": "val", "kind": 14, "detail": "Immutable variable", "insertText": "val "}),
            json!({"label": "var", "kind": 14, "detail": "Mutable variable", "insertText": "var "}),
            json!({"label": "fun", "kind": 14, "detail": "Function", "insertText": "fun "}),
            json!({"label": "if", "kind": 14, "detail": "If statement", "insertText": "if "}),
            json!({"label": "else", "kind": 14, "detail": "Else clause", "insertText": "else "}),
            json!({"label": "for", "kind": 14, "detail": "For loop", "insertText": "for "}),
            json!({"label": "return", "kind": 14, "detail": "Return statement", "insertText": "return "}),
            json!({"label": "import", "kind": 14, "detail": "Import statement", "insertText": "import "}),
            json!({"label": "export", "kind": 14, "detail": "Export declaration", "insertText": "export "}),
            json!({"label": "match", "kind": 14, "detail": "Pattern matching", "insertText": "match "}),
            json!({"label": "Success", "kind": 4, "detail": "Success constructor", "insertText": "Success("}),
            json!({"label": "Failure", "kind": 4, "detail": "Failure constructor", "insertText": "Failure("}),
            json!({"label": "Result", "kind": 7, "detail": "Result type", "insertText": "Result<"}),
            json!({"label": "true", "kind": 12, "detail": "Boolean literal", "insertText": "true"}),
            json!({"label": "false", "kind": 12, "detail": "Boolean literal", "insertText": "false"}),
        ]
    }

    fn get_type_aware_completions(&self, uri: &str) -> LspResult<Vec<Value>> {
        let mut items = Vec::new();

        // Try to get completions from current document's type information
        if let Ok(_) = self.ensure_parsed(uri) {
            let type_checker = self.type_checker.read().map_err(|_| {
                LspError::InternalError("Failed to acquire type checker lock".to_string())
            })?;

            // Add functions
            for (func_name, signature) in type_checker.get_all_functions() {
                let detail = format!(
                    "({}) -> {:?}",
                    signature
                        .params
                        .iter()
                        .map(|(name, param_type)| format!("{}: {:?}", name, param_type))
                        .collect::<Vec<_>>()
                        .join(", "),
                    signature.return_type
                );

                items.push(json!({
                    "label": func_name,
                    "kind": 3, // Function
                    "detail": detail,
                    "insertText": format!("{}(", func_name)
                }));
            }

            // Add variables
            for (var_name, var_type) in type_checker.get_all_variables() {
                items.push(json!({
                    "label": var_name,
                    "kind": 6, // Variable
                    "detail": format!("{:?}", var_type),
                    "insertText": var_name
                }));
            }
        }

        Ok(items)
    }

    fn get_stdlib_completions(&self) -> Vec<Value> {
        vec![
            json!({"label": "Integer", "kind": 7, "detail": "Integer type", "insertText": "Integer"}),
            json!({"label": "String", "kind": 7, "detail": "String type", "insertText": "String"}),
            json!({"label": "Bool", "kind": 7, "detail": "Boolean type", "insertText": "Bool"}),
            json!({"label": "\"bolt:stdio\"", "kind": 9, "detail": "Standard I/O module", "insertText": "\"bolt:stdio\""}),
            json!({"label": "\"bolt:math\"", "kind": 9, "detail": "Math utilities module", "insertText": "\"bolt:math\""}),
            json!({"label": "\"bolt:io\"", "kind": 9, "detail": "File I/O operations module", "insertText": "\"bolt:io\""}),
            json!({"label": "\"bolt:string\"", "kind": 9, "detail": "String utilities module", "insertText": "\"bolt:string\""}),
        ]
    }
}
