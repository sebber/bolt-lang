use crate::ast::{Program, Statement};
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone)]
pub struct ModuleExports {
    pub functions: Vec<String>,
    pub types: Vec<String>,
    pub variables: Vec<String>,
}

#[derive(Debug)]
pub struct ModuleSystem {
    modules: HashMap<String, Program>,
    exports: HashMap<String, ModuleExports>,
    resolved_imports: HashMap<String, Vec<String>>,
}

impl ModuleSystem {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            exports: HashMap::new(),
            resolved_imports: HashMap::new(),
        }
    }

    pub fn load_module(&mut self, module_path: &str) -> Result<(), String> {
        if self.modules.contains_key(module_path) {
            return Ok(()); // Already loaded
        }

        // Handle bolt: standard library prefix
        let file_path = if module_path.starts_with("bolt:") {
            let std_module = module_path.strip_prefix("bolt:").unwrap();
            format!("std/{}.bolt", std_module)
        } else if module_path.ends_with(".bolt") {
            module_path.to_string()
        } else {
            format!("{}.bolt", module_path)
        };

        // Read the file
        let content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read module '{}': {}", file_path, e))?;

        // Parse the module
        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize()?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        // Extract exports from the module
        let exports = self.extract_exports(&program);

        // Store the module and its exports
        self.modules.insert(module_path.to_string(), program);
        self.exports.insert(module_path.to_string(), exports);

        Ok(())
    }

    fn extract_exports(&self, program: &Program) -> ModuleExports {
        let mut exports = ModuleExports {
            functions: Vec::new(),
            types: Vec::new(),
            variables: Vec::new(),
        };

        for statement in &program.statements {
            match statement {
                Statement::Export { item } => {
                    // For now, categorize all exports as functions
                    // In a more sophisticated system, we'd track types of exported items
                    exports.functions.push(item.clone());
                }
                _ => {}
            }
        }

        exports
    }

    pub fn resolve_imports(&mut self, main_program: &Program) -> Result<(), String> {
        for statement in &main_program.statements {
            if let Statement::Import {
                module_name,
                module_path,
                items,
            } = statement
            {
                // Load the module if not already loaded
                self.load_module(module_path)?;

                // Resolve the import
                let imported_items = if let Some(specific_items) = items {
                    // Selective import: import { item1, item2 } from "path"
                    specific_items.clone()
                } else {
                    // Namespace import: import module from "path"
                    // For now, we'll still import all items but we need to handle namespacing
                    let exports = self
                        .exports
                        .get(module_path)
                        .ok_or_else(|| format!("Module '{}' not found", module_path))?;

                    let mut all_items = Vec::new();
                    all_items.extend(exports.functions.iter().cloned());
                    all_items.extend(exports.types.iter().cloned());
                    all_items.extend(exports.variables.iter().cloned());
                    all_items
                };

                // Store resolved imports with module identifier if namespace import
                let key = if module_name.is_some() {
                    format!("{}:{}", module_name.as_ref().unwrap(), module_path)
                } else {
                    module_path.clone()
                };
                self.resolved_imports.insert(key, imported_items);
            }
        }

        Ok(())
    }

    pub fn get_module(&self, module_path: &str) -> Option<&Program> {
        self.modules.get(module_path)
    }

    pub fn get_exports(&self, module_path: &str) -> Option<&ModuleExports> {
        self.exports.get(module_path)
    }

    pub fn get_all_functions(&self) -> HashMap<String, String> {
        let mut all_functions = HashMap::new();

        for (module_path, program) in &self.modules {
            for statement in &program.statements {
                if let Statement::Function { name, .. } = statement {
                    all_functions.insert(name.clone(), module_path.clone());
                }
            }
        }

        all_functions
    }
}
