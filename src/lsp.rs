use std::io::{self, BufRead, BufReader, Write, Read};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod lexer;
mod parser;
mod ast;

use lexer::Lexer;
use parser::Parser;

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

struct LspServer {
    documents: HashMap<String, String>,
}

impl LspServer {
    fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    fn run(&mut self) {
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin.lock());
        
        loop {
            // Read LSP headers
            let mut headers = HashMap::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap() == 0 {
                    return; // EOF
                }
                
                let line = line.trim();
                if line.is_empty() {
                    break; // End of headers
                }
                
                if let Some(colon_pos) = line.find(':') {
                    let key = line[..colon_pos].trim().to_lowercase();
                    let value = line[colon_pos + 1..].trim();
                    headers.insert(key, value.to_string());
                }
            }
            
            // Read content using the buffered reader directly
            if let Some(content_length) = headers.get("content-length") {
                if let Ok(length) = content_length.parse::<usize>() {
                    let mut content = vec![0u8; length];
                    match reader.read_exact(&mut content) {
                        Ok(_) => {
                            if let Ok(content_str) = String::from_utf8(content) {
                                eprintln!("LSP: Processing message of length: {}", length);
                                eprintln!("LSP: Raw content: '{}'", content_str);
                                
                                // The content should already be pure JSON at this point
                                let trimmed = content_str.trim();
                                match serde_json::from_str::<Value>(trimmed) {
                                    Ok(json_value) => {
                                        // Try to parse as our Message struct
                                        match serde_json::from_value::<Message>(json_value.clone()) {
                                            Ok(msg) => {
                                                if let Some(method) = &msg.method {
                                                    eprintln!("LSP: Successfully parsed method: {}", method);
                                                }
                                                self.handle_message(msg);
                                            }
                                            Err(e) => {
                                                eprintln!("LSP: Failed to convert to Message struct: {}", e);
                                                // Handle as raw JSON
                                                if let Some(method) = json_value.get("method").and_then(|m| m.as_str()) {
                                                    eprintln!("LSP: Handling raw JSON method: {}", method);
                                                    self.handle_raw_message(json_value);
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("LSP: Failed to parse JSON: {}", e);
                                        eprintln!("LSP: Content: '{}'", trimmed);
                                        
                                        // Check if this looks like it has headers mixed in
                                        if trimmed.contains("Content-Length:") {
                                            eprintln!("LSP: ERROR - Headers found in JSON content! Buffer issue detected.");
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("LSP: Failed to read message content: {}", e);
                        }
                    }
                }
            }
        }
    }

    fn handle_raw_message(&mut self, json_value: Value) {
        if let Some(method) = json_value.get("method").and_then(|m| m.as_str()) {
            let id = json_value.get("id");
            let params = json_value.get("params");
            
            match method {
                "textDocument/didOpen" => {
                    eprintln!("LSP: Handling didOpen from raw JSON");
                    if let Some(params) = params {
                        if let Some(uri) = params.get("textDocument").and_then(|doc| doc.get("uri")).and_then(|u| u.as_str()) {
                            if let Some(text) = params.get("textDocument").and_then(|doc| doc.get("text")).and_then(|t| t.as_str()) {
                                if let Some(lang_id) = params.get("textDocument").and_then(|doc| doc.get("languageId")).and_then(|l| l.as_str()) {
                                    eprintln!("LSP: Raw didOpen - {} (language: {})", uri, lang_id);
                                }
                                self.documents.insert(uri.to_string(), text.to_string());
                                // Skip diagnostics for now to avoid lexer crashes
                                // self.publish_diagnostics(uri, text);
                            }
                        }
                    }
                }
                "textDocument/hover" => {
                    eprintln!("LSP: Handling hover from raw JSON");
                    if let Some(id_val) = id {
                        if let Some(uri) = params.and_then(|p| p.get("textDocument")).and_then(|doc| doc.get("uri")).and_then(|u| u.as_str()) {
                            if let Some(position) = params.and_then(|p| p.get("position")) {
                                if let Some(line) = position.get("line").and_then(|l| l.as_u64()) {
                                    if let Some(character) = position.get("character").and_then(|c| c.as_u64()) {
                                        eprintln!("LSP: Raw hover at {}:{}", line, character);
                                        if let Some(doc) = self.documents.get(uri) {
                                            let hover_info = self.get_hover_info(doc, line as usize, character as usize);
                                            let response = Message {
                                                jsonrpc: "2.0".to_string(),
                                                id: Some(id_val.clone()),
                                                method: None,
                                                params: None,
                                                result: Some(json!({
                                                    "contents": {
                                                        "kind": "markdown",
                                                        "value": hover_info
                                                    }
                                                })),
                                                error: None,
                                            };
                                            eprintln!("LSP: Sending raw hover response: {}", hover_info);
                                            self.send_message(response);
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                        
                        // Fallback response
                        let response = Message {
                            jsonrpc: "2.0".to_string(),
                            id: Some(id_val.clone()),
                            method: None,
                            params: None,
                            result: Some(Value::Null),
                            error: None,
                        };
                        self.send_message(response);
                    }
                }
                "textDocument/completion" => {
                    eprintln!("LSP: Handling completion from raw JSON");
                    if let Some(id_val) = id {
                        let items = vec![
                            json!({"label": "val", "kind": 14, "detail": "Immutable variable", "insertText": "val "}),
                            json!({"label": "var", "kind": 14, "detail": "Mutable variable", "insertText": "var "}),
                            json!({"label": "fun", "kind": 14, "detail": "Function", "insertText": "fun "}),
                            json!({"label": "print", "kind": 3, "detail": "Print function", "insertText": "print("}),
                        ];
                        
                        let response = Message {
                            jsonrpc: "2.0".to_string(),
                            id: Some(id_val.clone()),
                            method: None,
                            params: None,
                            result: Some(json!(items)),
                            error: None,
                        };
                        eprintln!("LSP: Sending raw completion response");
                        self.send_message(response);
                    }
                }
                "$/cancelRequest" => {
                    eprintln!("LSP: Received cancel request - ignoring");
                    // Just ignore cancel requests
                }
                _ => {
                    eprintln!("LSP: Unknown raw method: {}", method);
                }
            }
        }
    }

    fn handle_message(&mut self, msg: Message) {
        if let Some(method) = &msg.method {
            eprintln!("LSP: Handling method: {}", method);
            match method.as_str() {
                "initialize" => {
                    eprintln!("LSP: Initializing with capabilities");
                    let response = Message {
                        jsonrpc: "2.0".to_string(),
                        id: msg.id,
                        method: None,
                        params: None,
                        result: Some(json!({
                            "capabilities": {
                                "textDocumentSync": {
                                    "openClose": true,
                                    "change": 1,
                                    "willSave": false,
                                    "willSaveWaitUntil": false,
                                    "save": false
                                },
                                "completionProvider": {
                                    "triggerCharacters": [".", ":", " "],
                                    "allCommitCharacters": [],
                                    "resolveProvider": false
                                },
                                "hoverProvider": true,
                                "definitionProvider": false,
                                "declarationProvider": false,
                                "implementationProvider": false,
                                "typeDefinitionProvider": false,
                                "referencesProvider": false,
                                "documentHighlightProvider": false,
                                "documentSymbolProvider": false,
                                "workspaceSymbolProvider": false,
                                "codeActionProvider": false,
                                "codeLensProvider": false,
                                "documentFormattingProvider": false,
                                "documentRangeFormattingProvider": false,
                                "documentOnTypeFormattingProvider": false,
                                "renameProvider": false,
                                "documentLinkProvider": false,
                                "colorProvider": false,
                                "foldingRangeProvider": false,
                                "executeCommandProvider": false,
                                "workspace": {
                                    "workspaceFolders": {
                                        "supported": false
                                    }
                                }
                            },
                            "serverInfo": {
                                "name": "Bolt Language Server",
                                "version": "0.1.0"
                            }
                        })),
                        error: None,
                    };
                    eprintln!("LSP: Sending initialization response");
                    self.send_message(response);
                }
                
                "initialized" => {
                    eprintln!("LSP: Initialized");
                }
                
                "textDocument/didOpen" => {
                    eprintln!("LSP: Document opened");
                    if let Some(params) = msg.params {
                        eprintln!("LSP: didOpen params: {}", serde_json::to_string_pretty(&params).unwrap_or_default());
                        if let Some(uri) = params["textDocument"]["uri"].as_str() {
                            if let Some(text) = params["textDocument"]["text"].as_str() {
                                if let Some(lang_id) = params["textDocument"]["languageId"].as_str() {
                                    eprintln!("LSP: Opened {} (language: {})", uri, lang_id);
                                }
                                self.documents.insert(uri.to_string(), text.to_string());
                                // Skip diagnostics for now to avoid lexer crashes
                                // self.publish_diagnostics(uri, text);
                            }
                        }
                    }
                }
                
                "textDocument/didChange" => {
                    if let Some(params) = msg.params {
                        if let Some(uri) = params["textDocument"]["uri"].as_str() {
                            if let Some(changes) = params["contentChanges"].as_array() {
                                if let Some(change) = changes.first() {
                                    if let Some(text) = change["text"].as_str() {
                                        self.documents.insert(uri.to_string(), text.to_string());
                                        // Skip diagnostics for now to avoid lexer crashes
                                        // self.publish_diagnostics(uri, text);
                                    }
                                }
                            }
                        }
                    }
                }
                
                "textDocument/completion" => {
                    eprintln!("LSP: Received completion request");
                    if let Some(params) = &msg.params {
                        eprintln!("LSP: Completion params: {}", serde_json::to_string_pretty(params).unwrap_or_default());
                    }
                    
                    // Simple completion items without complex snippets
                    let items = vec![
                        json!({"label": "val", "kind": 14, "detail": "Immutable variable", "insertText": "val "}),
                        json!({"label": "var", "kind": 14, "detail": "Mutable variable", "insertText": "var "}),
                        json!({"label": "fun", "kind": 14, "detail": "Function", "insertText": "fun "}),
                        json!({"label": "if", "kind": 14, "detail": "If statement", "insertText": "if "}),
                        json!({"label": "for", "kind": 14, "detail": "For loop", "insertText": "for "}),
                        json!({"label": "import", "kind": 14, "detail": "Import", "insertText": "import "}),
                        json!({"label": "print", "kind": 3, "detail": "Print function", "insertText": "print("}),
                        json!({"label": "Integer", "kind": 7, "detail": "Integer type", "insertText": "Integer"}),
                        json!({"label": "String", "kind": 7, "detail": "String type", "insertText": "String"}),
                        json!({"label": "Bool", "kind": 7, "detail": "Boolean type", "insertText": "Bool"}),
                    ];
                    
                    let response = Message {
                        jsonrpc: "2.0".to_string(),
                        id: msg.id,
                        method: None,
                        params: None,
                        result: Some(json!(items)),
                        error: None,
                    };
                    eprintln!("LSP: Sending completion response with {} items", items.len());
                    self.send_message(response);
                }
                
                "textDocument/hover" => {
                    eprintln!("LSP: Received hover request");
                    if let Some(params) = &msg.params {
                        eprintln!("LSP: Hover params: {}", serde_json::to_string_pretty(params).unwrap_or_default());
                        if let Some(uri) = params["textDocument"]["uri"].as_str() {
                            if let Some(position) = params["position"].as_object() {
                                if let Some(line) = position["line"].as_u64() {
                                    if let Some(character) = position["character"].as_u64() {
                                        eprintln!("LSP: Hover at {}:{}", line, character);
                                        if let Some(doc) = self.documents.get(uri) {
                                            let hover_info = self.get_hover_info(doc, line as usize, character as usize);
                                            let response = Message {
                                                jsonrpc: "2.0".to_string(),
                                                id: msg.id,
                                                method: None,
                                                params: None,
                                                result: Some(json!({
                                                    "contents": {
                                                        "kind": "markdown",
                                                        "value": hover_info
                                                    }
                                                })),
                                                error: None,
                                            };
                                            eprintln!("LSP: Sending hover response: {}", hover_info);
                                            self.send_message(response);
                                            return;
                                        } else {
                                            eprintln!("LSP: Document not found in cache");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    eprintln!("LSP: Sending null hover response");
                    // No hover info
                    let response = Message {
                        jsonrpc: "2.0".to_string(),
                        id: msg.id,
                        method: None,
                        params: None,
                        result: Some(Value::Null),
                        error: None,
                    };
                    self.send_message(response);
                }
                
                "shutdown" => {
                    let response = Message {
                        jsonrpc: "2.0".to_string(),
                        id: msg.id,
                        method: None,
                        params: None,
                        result: Some(Value::Null),
                        error: None,
                    };
                    self.send_message(response);
                }
                
                "exit" => {
                    std::process::exit(0);
                }
                
                _ => {
                    eprintln!("LSP: Unknown method: {}", method);
                    eprintln!("LSP: Full message: {}", serde_json::to_string_pretty(&msg).unwrap_or_default());
                    
                    // Send empty response for requests (messages with ID)
                    if msg.id.is_some() {
                        let response = Message {
                            jsonrpc: "2.0".to_string(),
                            id: msg.id,
                            method: None,
                            params: None,
                            result: Some(Value::Null),
                            error: None,
                        };
                        self.send_message(response);
                    }
                }
            }
        }
    }

    fn publish_diagnostics(&self, uri: &str, text: &str) {
        let mut diagnostics = Vec::new();
        
        // Try to parse the document
        let mut lexer = Lexer::new(text.to_string());
        match lexer.tokenize() {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                if let Err(e) = parser.parse() {
                    // Parser error
                    diagnostics.push(json!({
                        "range": {
                            "start": {"line": 0, "character": 0},
                            "end": {"line": 0, "character": 0}
                        },
                        "severity": 1, // Error
                        "message": format!("Parse error: {}", e)
                    }));
                }
            }
            Err(e) => {
                // Lexer error
                diagnostics.push(json!({
                    "range": {
                        "start": {"line": 0, "character": 0},
                        "end": {"line": 0, "character": 0}
                    },
                    "severity": 1, // Error
                    "message": format!("Lexer error: {}", e)
                }));
            }
        }
        
        // Check for common issues
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            // Check for print without import
            if line.contains("print(") && !text.contains("bolt:stdio") {
                diagnostics.push(json!({
                    "range": {
                        "start": {"line": i, "character": 0},
                        "end": {"line": i, "character": line.len()}
                    },
                    "severity": 2, // Warning
                    "message": "print function used but bolt:stdio not imported"
                }));
            }
            
            // Check for wrong assignment operator
            if line.contains("val ") && line.contains(" = ") && !line.contains(" := ") {
                diagnostics.push(json!({
                    "range": {
                        "start": {"line": i, "character": 0},
                        "end": {"line": i, "character": line.len()}
                    },
                    "severity": 1, // Error
                    "message": "Use := for variable declaration"
                }));
            }
        }
        
        let notification = Message {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some("textDocument/publishDiagnostics".to_string()),
            params: Some(json!({
                "uri": uri,
                "diagnostics": diagnostics
            })),
            result: None,
            error: None,
        };
        self.send_message(notification);
    }

    fn send_message(&self, msg: Message) {
        let content = serde_json::to_string(&msg).unwrap();
        print!("Content-Length: {}\r\n\r\n{}", content.len(), content);
        io::stdout().flush().unwrap();
    }

    fn get_hover_info(&self, document: &str, line: usize, character: usize) -> String {
        eprintln!("LSP: Getting hover info at line {} character {}", line, character);
        
        let lines: Vec<&str> = document.lines().collect();
        if line >= lines.len() {
            return "No information available".to_string();
        }
        
        let current_line = lines[line];
        eprintln!("LSP: Current line: '{}'", current_line);
        
        // Find the word at the cursor position
        if character >= current_line.len() {
            return "No information available".to_string();
        }
        
        // Find word boundaries
        let mut start = character;
        let mut end = character;
        
        let chars: Vec<char> = current_line.chars().collect();
        
        // Move start backwards to find beginning of word
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        // Move end forwards to find end of word
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        if start == end {
            return "No information available".to_string();
        }
        
        let word: String = chars[start..end].iter().collect();
        eprintln!("LSP: Found word: '{}'", word);
        
        // Provide specific hover information based on the word
        match word.as_str() {
            "print" => {
                "**`print(value)`**\n\n*Built-in function*\n\nPrints a value to the console.\n\n**Examples:**\n```bolt\nprint(\"Hello, World!\")\nprint(42)\nprint(true)\n```".to_string()
            }
            "val" => {
                "**`val`**\n\n*Keyword*\n\nDeclares an immutable variable (constant).\n\n**Syntax:**\n```bolt\nval name := value\nval name: Type = value\n```\n\n**Examples:**\n```bolt\nval message := \"Hello\"\nval count: Integer = 42\n```".to_string()
            }
            "var" => {
                "**`var`**\n\n*Keyword*\n\nDeclares a mutable variable.\n\n**Syntax:**\n```bolt\nvar name := value\nvar name: Type = value\n```\n\n**Examples:**\n```bolt\nvar counter := 0\nvar status: String = \"ready\"\n```".to_string()
            }
            "fun" => {
                "**`fun`**\n\n*Keyword*\n\nDeclares a function.\n\n**Syntax:**\n```bolt\nfun name(param: Type): ReturnType {\n    // function body\n    return value\n}\n```\n\n**Example:**\n```bolt\nfun greet(name: String): String {\n    return \"Hello, \" + name\n}\n```".to_string()
            }
            "if" => {
                "**`if`**\n\n*Keyword*\n\nConditional statement.\n\n**Syntax:**\n```bolt\nif condition {\n    // code\n} else if other_condition {\n    // code\n} else {\n    // code\n}\n```".to_string()
            }
            "true" | "false" => {
                format!("**`{}`**\n\n*Boolean literal*\n\nA boolean value representing {} condition.", word, if word == "true" { "a true" } else { "a false" })
            }
            _ => {
                // Check if it's a function by looking for function declarations
                if let Some(func_info) = self.find_function_declaration(document, &word) {
                    func_info
                }
                // Check if it's a variable by looking for variable declarations
                else if let Some(var_info) = self.find_variable_declaration(document, &word) {
                    var_info
                } else {
                    format!("**`{}`**\n\n*Identifier*\n\nNo additional information available.", word)
                }
            }
        }
    }
    
    fn find_variable_declaration(&self, document: &str, var_name: &str) -> Option<String> {
        eprintln!("LSP: Looking for variable declaration: {}", var_name);
        
        let lines: Vec<&str> = document.lines().collect();
        eprintln!("LSP: Document has {} lines", lines.len());
        
        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();
            
            // Look for val declarations: val name := value or val name: Type = value
            if line.starts_with("val ") {
                if let Some(rest) = line.strip_prefix("val ") {
                    if let Some(name_part) = rest.split(&[' ', ':', '=']).next() {
                        if name_part == var_name {
                            eprintln!("LSP: Found val declaration at line {}: {}", i, line);
                            
                            // Look for documentation comments above this declaration
                            let doc = self.extract_documentation(&lines, i);
                            
                            // Extract type information if available
                            let type_info = if line.contains(':') {
                                let parts: Vec<&str> = rest.split(':').collect();
                                if parts.len() > 1 {
                                    let type_part = parts[1].split('=').next().unwrap_or("").trim();
                                    format!("Type: `{}`", type_part)
                                } else {
                                    "Type: *inferred*".to_string()
                                }
                            } else {
                                "Type: *inferred*".to_string()
                            };
                            
                            let mut result = format!("**`{}`**\n\n*Immutable variable*\n\n{}", var_name, type_info);
                            
                            if let Some(documentation) = doc {
                                result = format!("**`{}`**\n\n{}\n\n*Immutable variable*\n\n{}", var_name, documentation, type_info);
                            }
                            
                            eprintln!("LSP: Returning variable info for {}", var_name);
                            return Some(result);
                        }
                    }
                }
            }
            
            // Look for var declarations: var name := value or var name: Type = value
            if line.starts_with("var ") {
                if let Some(rest) = line.strip_prefix("var ") {
                    if let Some(name_part) = rest.split(&[' ', ':', '=']).next() {
                        if name_part == var_name {
                            eprintln!("LSP: Found var declaration at line {}: {}", i, line);
                            
                            // Look for documentation comments above this declaration
                            let doc = self.extract_documentation(&lines, i);
                            
                            // Extract type information if available
                            let type_info = if line.contains(':') {
                                let parts: Vec<&str> = rest.split(':').collect();
                                if parts.len() > 1 {
                                    let type_part = parts[1].split('=').next().unwrap_or("").trim();
                                    format!("Type: `{}`", type_part)
                                } else {
                                    "Type: *inferred*".to_string()
                                }
                            } else {
                                "Type: *inferred*".to_string()
                            };
                            
                            let mut result = format!("**`{}`**\n\n*Mutable variable*\n\n{}", var_name, type_info);
                            
                            if let Some(documentation) = doc {
                                result = format!("**`{}`**\n\n{}\n\n*Mutable variable*\n\n{}", var_name, documentation, type_info);
                            }
                            
                            eprintln!("LSP: Returning variable info for {}", var_name);
                            return Some(result);
                        }
                    }
                }
            }
        }
        
        eprintln!("LSP: No variable declaration found for {}", var_name);
        None
    }
    
    fn find_function_declaration(&self, document: &str, func_name: &str) -> Option<String> {
        eprintln!("LSP: Looking for function declaration: {}", func_name);
        
        let lines: Vec<&str> = document.lines().collect();
        eprintln!("LSP: Document has {} lines", lines.len());
        
        for (i, line) in lines.iter().enumerate() {
            let line = line.trim();
            
            // Look for function declarations: fun name(params): ReturnType {
            if line.starts_with("fun ") {
                if let Some(rest) = line.strip_prefix("fun ") {
                    // Extract function name (everything before '(')
                    if let Some(paren_pos) = rest.find('(') {
                        let name_part = rest[..paren_pos].trim();
                        if name_part == func_name {
                            eprintln!("LSP: Found function declaration at line {}: {}", i, line);
                            
                            // Look for documentation comments above this declaration
                            let doc = self.extract_documentation(&lines, i);
                            
                            // Extract function signature - be more careful here
                            let mut signature = String::new();
                            let mut brace_count = 0;
                            let mut in_function = false;
                            
                            // Build the complete function signature (may span multiple lines)
                            for j in i..lines.len() {
                                if j >= lines.len() {
                                    eprintln!("LSP: Breaking - line index {} out of bounds", j);
                                    break;
                                }
                                
                                let func_line = lines[j].trim();
                                signature.push_str(func_line);
                                
                                if func_line.contains('{') {
                                    brace_count += func_line.matches('{').count();
                                    in_function = true;
                                }
                                if func_line.contains('}') {
                                    brace_count -= func_line.matches('}').count();
                                }
                                
                                if in_function && brace_count == 0 {
                                    break;
                                }
                                
                                if !func_line.ends_with('{') && j < lines.len() - 1 {
                                    signature.push(' ');
                                }
                            }
                            
                            // Extract just the signature part (before the opening brace)
                            if let Some(brace_pos) = signature.find('{') {
                                signature = signature[..brace_pos].trim().to_string();
                            }
                            
                            eprintln!("LSP: Function signature: {}", signature);
                            
                            let mut result = format!("**`{}`**\n\n*Function*\n\n```bolt\n{}\n```", func_name, signature);
                            
                            if let Some(documentation) = doc {
                                result = format!("**`{}`**\n\n{}\n\n*Function*\n\n```bolt\n{}\n```", func_name, documentation, signature);
                            }
                            
                            eprintln!("LSP: Returning function info for {}", func_name);
                            return Some(result);
                        }
                    }
                }
            }
        }
        
        eprintln!("LSP: No function declaration found for {}", func_name);
        None
    }
    
    fn extract_documentation(&self, lines: &[&str], declaration_line: usize) -> Option<String> {
        eprintln!("LSP: Extracting docs for declaration at line {}", declaration_line);
        
        if lines.is_empty() || declaration_line >= lines.len() {
            eprintln!("LSP: Invalid lines or declaration_line");
            return None;
        }
        
        let mut docs = Vec::new();
        let mut i = declaration_line;
        let mut found_end = false;
        
        // Go backwards to find documentation comments
        while i > 0 {
            i -= 1;
            
            if i >= lines.len() {
                eprintln!("LSP: Index {} out of bounds for {} lines", i, lines.len());
                break;
            }
            
            let line = lines[i].trim();
            eprintln!("LSP: Checking line {}: '{}'", i, line);
            
            // Skip empty lines
            if line.is_empty() {
                eprintln!("LSP: Skipping empty line");
                continue;
            }
            
            // Handle end of multiline comment */
            if line.ends_with("*/") && !found_end {
                found_end = true;
                eprintln!("LSP: Found comment end");
                
                // Handle single-line /** content */ 
                if line.starts_with("/**") {
                    let content = line.strip_prefix("/**").unwrap_or("").strip_suffix("*/").unwrap_or("").trim();
                    eprintln!("LSP: Single-line comment content: '{}'", content);
                    if !content.is_empty() {
                        docs.insert(0, content.to_string());
                    }
                    break;
                }
                
                // Handle last line of multiline comment: * content */
                if line.starts_with("*") {
                    let content = line.strip_prefix("*").unwrap_or("").strip_suffix("*/").unwrap_or("").trim();
                    eprintln!("LSP: End comment line content: '{}'", content);
                    if !content.is_empty() {
                        docs.insert(0, content.to_string());
                    }
                }
                continue;
            }
            
            // Only process lines after we found the end
            if !found_end {
                eprintln!("LSP: No comment end found yet, stopping");
                break;
            }
            
            // Handle middle lines of multiline comment: * content
            if line.starts_with("*") && !line.starts_with("/**") {
                let content = line.strip_prefix("*").unwrap_or("").trim();
                eprintln!("LSP: Middle comment line content: '{}'", content);
                if !content.is_empty() {
                    docs.insert(0, content.to_string());
                }
                continue;
            }
            
            // Handle start of multiline comment: /**
            if line.starts_with("/**") && !line.ends_with("*/") {
                let content = line.strip_prefix("/**").unwrap_or("").trim();
                eprintln!("LSP: Start comment line content: '{}'", content);
                if !content.is_empty() {
                    docs.insert(0, content.to_string());
                }
                break;
            }
            
            // If we hit a non-comment line, stop
            eprintln!("LSP: Hit non-comment line, stopping");
            break;
        }
        
        eprintln!("LSP: Extracted {} documentation lines", docs.len());
        if docs.is_empty() {
            None
        } else {
            let result = docs.join("\n\n");
            eprintln!("LSP: Final documentation: '{}'", result);
            Some(result)
        }
    }
}

fn main() {
    eprintln!("Bolt LSP Server starting...");
    let mut server = LspServer::new();
    server.run();
}