use std::collections::HashMap;
use crate::ast::Type;
use crate::error::{CompileError, ParseError, SourceLocation};

pub type ScopeId = usize;
pub type SymbolId = usize;

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable { mutable: bool, initialized: bool },
    Function { params: Vec<Type>, return_type: Option<Type> },
    Type { definition: Type },
    Parameter,
    Import { module_path: String },
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
    pub kind: SymbolKind,
    pub scope_id: ScopeId,
    pub declared_at: Option<SourceLocation>,
    pub used: bool,
}

impl Symbol {
    pub fn new_variable(
        name: String, 
        symbol_type: Type, 
        mutable: bool, 
        scope_id: ScopeId,
        location: Option<SourceLocation>
    ) -> Self {
        Self {
            name,
            symbol_type,
            kind: SymbolKind::Variable { mutable, initialized: false },
            scope_id,
            declared_at: location,
            used: false,
        }
    }
    
    pub fn new_function(
        name: String,
        params: Vec<Type>,
        return_type: Option<Type>,
        scope_id: ScopeId,
        location: Option<SourceLocation>
    ) -> Self {
        Self {
            name,
            symbol_type: Type::Custom("function".to_string()), // We might need a Function type later
            kind: SymbolKind::Function { params, return_type },
            scope_id,
            declared_at: location,
            used: false,
        }
    }
    
    pub fn new_type(
        name: String,
        definition: Type,
        scope_id: ScopeId,
        location: Option<SourceLocation>
    ) -> Self {
        Self {
            name: name.clone(),
            symbol_type: definition.clone(),
            kind: SymbolKind::Type { definition },
            scope_id,
            declared_at: location,
            used: false,
        }
    }
    
    pub fn new_parameter(
        name: String,
        symbol_type: Type,
        scope_id: ScopeId,
        location: Option<SourceLocation>
    ) -> Self {
        Self {
            name,
            symbol_type,
            kind: SymbolKind::Parameter,
            scope_id,
            declared_at: location,
            used: false,
        }
    }
    
    pub fn is_mutable(&self) -> bool {
        match &self.kind {
            SymbolKind::Variable { mutable, .. } => *mutable,
            _ => false,
        }
    }
    
    pub fn is_initialized(&self) -> bool {
        match &self.kind {
            SymbolKind::Variable { initialized, .. } => *initialized,
            SymbolKind::Parameter => true, // Parameters are always initialized
            SymbolKind::Function { .. } => true,
            SymbolKind::Type { .. } => true,
            SymbolKind::Import { .. } => true,
        }
    }
    
    pub fn mark_initialized(&mut self) {
        if let SymbolKind::Variable { initialized, .. } = &mut self.kind {
            *initialized = true;
        }
    }
    
    pub fn mark_used(&mut self) {
        self.used = true;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScopeKind {
    Global,
    Function { name: String },
    Block,
    ForLoop,
    IfStatement,
    Module { path: String },
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: ScopeId,
    pub kind: ScopeKind,
    pub parent: Option<ScopeId>,
    pub children: Vec<ScopeId>,
    pub symbols: HashMap<String, SymbolId>,
    pub depth: usize,
}

impl Scope {
    pub fn new(id: ScopeId, kind: ScopeKind, parent: Option<ScopeId>) -> Self {
        let depth = parent.map(|_| 1).unwrap_or(0); // Will be updated by SymbolTable
        Self {
            id,
            kind,
            parent,
            children: Vec::new(),
            symbols: HashMap::new(),
            depth,
        }
    }
    
    pub fn add_child(&mut self, child_id: ScopeId) {
        self.children.push(child_id);
    }
    
    pub fn add_symbol(&mut self, name: String, symbol_id: SymbolId) -> Result<(), String> {
        if self.symbols.contains_key(&name) {
            return Err(format!("Symbol '{}' is already defined in this scope", name));
        }
        self.symbols.insert(name, symbol_id);
        Ok(())
    }
    
    pub fn get_symbol(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).copied()
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    symbols: Vec<Symbol>,
    current_scope: ScopeId,
    global_scope: ScopeId,
    next_scope_id: ScopeId,
    next_symbol_id: SymbolId,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut symbol_table = Self {
            scopes: Vec::new(),
            symbols: Vec::new(),
            current_scope: 0,
            global_scope: 0,
            next_scope_id: 0,
            next_symbol_id: 0,
        };
        
        // Create global scope
        let global_scope = Scope::new(0, ScopeKind::Global, None);
        symbol_table.scopes.push(global_scope);
        symbol_table.next_scope_id = 1;
        
        symbol_table
    }
    
    pub fn enter_scope(&mut self, kind: ScopeKind) -> ScopeId {
        let scope_id = self.next_scope_id;
        self.next_scope_id += 1;
        
        let parent_depth = self.scopes[self.current_scope].depth;
        let mut new_scope = Scope::new(scope_id, kind, Some(self.current_scope));
        new_scope.depth = parent_depth + 1;
        
        // Add this scope as a child of the current scope
        self.scopes[self.current_scope].add_child(scope_id);
        self.scopes.push(new_scope);
        
        self.current_scope = scope_id;
        scope_id
    }
    
    pub fn exit_scope(&mut self) -> Result<ScopeId, CompileError> {
        if self.current_scope == self.global_scope {
            return Err(CompileError::CodegenError("Cannot exit global scope".to_string()));
        }
        
        let parent = self.scopes[self.current_scope].parent
            .ok_or_else(|| CompileError::CodegenError("Current scope has no parent".to_string()))?;
        
        self.current_scope = parent;
        Ok(parent)
    }
    
    pub fn current_scope(&self) -> ScopeId {
        self.current_scope
    }
    
    pub fn global_scope(&self) -> ScopeId {
        self.global_scope
    }
    
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(id)
    }
    
    pub fn get_scope_mut(&mut self, id: ScopeId) -> Option<&mut Scope> {
        self.scopes.get_mut(id)
    }
    
    pub fn get_symbol(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id)
    }
    
    pub fn get_symbol_mut(&mut self, id: SymbolId) -> Option<&mut Symbol> {
        self.symbols.get_mut(id)
    }
    
    pub fn declare_variable(
        &mut self, 
        name: String, 
        symbol_type: Type, 
        mutable: bool,
        location: Option<SourceLocation>
    ) -> Result<SymbolId, CompileError> {
        let symbol_id = self.next_symbol_id;
        self.next_symbol_id += 1;
        
        let symbol = Symbol::new_variable(name.clone(), symbol_type, mutable, self.current_scope, location.clone());
        
        // Check for redeclaration in current scope
        if let Some(scope) = self.get_scope_mut(self.current_scope) {
            scope.add_symbol(name.clone(), symbol_id)
                .map_err(|e| CompileError::ParseError(ParseError::InvalidSyntax { 
                    message: e, 
                    location 
                }))?;
        }
        
        self.symbols.push(symbol);
        Ok(symbol_id)
    }
    
    pub fn declare_function(
        &mut self,
        name: String,
        params: Vec<Type>,
        return_type: Option<Type>,
        location: Option<SourceLocation>
    ) -> Result<SymbolId, CompileError> {
        let symbol_id = self.next_symbol_id;
        self.next_symbol_id += 1;
        
        let symbol = Symbol::new_function(name.clone(), params, return_type, self.current_scope, location.clone());
        
        if let Some(scope) = self.get_scope_mut(self.current_scope) {
            scope.add_symbol(name.clone(), symbol_id)
                .map_err(|e| CompileError::ParseError(ParseError::InvalidSyntax { 
                    message: e, 
                    location 
                }))?;
        }
        
        self.symbols.push(symbol);
        Ok(symbol_id)
    }
    
    pub fn declare_type(
        &mut self,
        name: String,
        definition: Type,
        location: Option<SourceLocation>
    ) -> Result<SymbolId, CompileError> {
        let symbol_id = self.next_symbol_id;
        self.next_symbol_id += 1;
        
        let symbol = Symbol::new_type(name.clone(), definition, self.current_scope, location.clone());
        
        if let Some(scope) = self.get_scope_mut(self.current_scope) {
            scope.add_symbol(name.clone(), symbol_id)
                .map_err(|e| CompileError::ParseError(ParseError::InvalidSyntax { 
                    message: e, 
                    location 
                }))?;
        }
        
        self.symbols.push(symbol);
        Ok(symbol_id)
    }
    
    pub fn declare_parameter(
        &mut self,
        name: String,
        symbol_type: Type,
        location: Option<SourceLocation>
    ) -> Result<SymbolId, CompileError> {
        let symbol_id = self.next_symbol_id;
        self.next_symbol_id += 1;
        
        let symbol = Symbol::new_parameter(name.clone(), symbol_type, self.current_scope, location.clone());
        
        if let Some(scope) = self.get_scope_mut(self.current_scope) {
            scope.add_symbol(name.clone(), symbol_id)
                .map_err(|e| CompileError::ParseError(ParseError::InvalidSyntax { 
                    message: e, 
                    location 
                }))?;
        }
        
        self.symbols.push(symbol);
        Ok(symbol_id)
    }
    
    /// Look up a symbol by name, searching from current scope up to global scope
    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        let mut current_scope_id = self.current_scope;
        
        loop {
            if let Some(scope) = self.get_scope(current_scope_id) {
                if let Some(symbol_id) = scope.get_symbol(name) {
                    return Some(symbol_id);
                }
                
                // Move to parent scope
                if let Some(parent_id) = scope.parent {
                    current_scope_id = parent_id;
                } else {
                    break; // Reached global scope
                }
            } else {
                break;
            }
        }
        
        None
    }
    
    /// Look up a symbol by name in the current scope only
    pub fn lookup_current_scope(&self, name: &str) -> Option<SymbolId> {
        if let Some(scope) = self.get_scope(self.current_scope) {
            scope.get_symbol(name)
        } else {
            None
        }
    }
    
    /// Mark a symbol as used
    pub fn use_symbol(&mut self, symbol_id: SymbolId) {
        if let Some(symbol) = self.get_symbol_mut(symbol_id) {
            symbol.mark_used();
        }
    }
    
    /// Mark a variable as initialized
    pub fn initialize_symbol(&mut self, symbol_id: SymbolId) {
        if let Some(symbol) = self.get_symbol_mut(symbol_id) {
            symbol.mark_initialized();
        }
    }
    
    /// Get all symbols in a given scope
    pub fn get_symbols_in_scope(&self, scope_id: ScopeId) -> Vec<SymbolId> {
        if let Some(scope) = self.get_scope(scope_id) {
            scope.symbols.values().copied().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Get all unused symbols (for warnings)
    pub fn get_unused_symbols(&self) -> Vec<SymbolId> {
        self.symbols.iter()
            .enumerate()
            .filter(|(_, symbol)| !symbol.used && !matches!(symbol.kind, SymbolKind::Function { .. }))
            .map(|(id, _)| id)
            .collect()
    }
    
    /// Get all uninitialized variables (for warnings)
    pub fn get_uninitialized_variables(&self) -> Vec<SymbolId> {
        self.symbols.iter()
            .enumerate()
            .filter(|(_, symbol)| !symbol.is_initialized())
            .map(|(id, _)| id)
            .collect()
    }
    
    /// Convert symbol table to the legacy HashMap format for compatibility
    /// TODO: Remove this once all codegen is updated
    pub fn to_legacy_variables(&self) -> HashMap<String, String> {
        let mut variables = HashMap::new();
        
        for symbol in &self.symbols {
            if matches!(symbol.kind, SymbolKind::Variable { .. } | SymbolKind::Parameter) {
                let type_str = match &symbol.symbol_type {
                    Type::Integer => "int".to_string(),
                    Type::String => "char*".to_string(),
                    Type::Bool => "bool".to_string(),
                    Type::Custom(name) => name.clone(),
                    _ => "unknown".to_string(),
                };
                variables.insert(symbol.name.clone(), type_str);
            }
        }
        
        variables
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_table_creation() {
        let symbol_table = SymbolTable::new();
        assert_eq!(symbol_table.current_scope, 0);
        assert_eq!(symbol_table.global_scope, 0);
        assert_eq!(symbol_table.scopes.len(), 1);
        assert!(matches!(symbol_table.scopes[0].kind, ScopeKind::Global));
    }

    #[test]
    fn test_scope_management() {
        let mut symbol_table = SymbolTable::new();
        
        // Enter function scope
        let func_scope = symbol_table.enter_scope(ScopeKind::Function { name: "test".to_string() });
        assert_eq!(func_scope, 1);
        assert_eq!(symbol_table.current_scope, 1);
        assert_eq!(symbol_table.scopes.len(), 2);
        
        // Enter block scope
        let block_scope = symbol_table.enter_scope(ScopeKind::Block);
        assert_eq!(block_scope, 2);
        assert_eq!(symbol_table.current_scope, 2);
        
        // Exit back to function scope
        symbol_table.exit_scope().unwrap();
        assert_eq!(symbol_table.current_scope, 1);
        
        // Exit back to global scope
        symbol_table.exit_scope().unwrap();
        assert_eq!(symbol_table.current_scope, 0);
    }

    #[test]
    fn test_variable_declaration() {
        let mut symbol_table = SymbolTable::new();
        
        // Declare a variable
        let symbol_id = symbol_table.declare_variable(
            "x".to_string(),
            Type::Integer,
            true,
            None
        ).unwrap();
        
        assert_eq!(symbol_id, 0);
        
        // Look it up
        let found_id = symbol_table.lookup("x").unwrap();
        assert_eq!(found_id, symbol_id);
        
        let symbol = symbol_table.get_symbol(found_id).unwrap();
        assert_eq!(symbol.name, "x");
        assert_eq!(symbol.symbol_type, Type::Integer);
        assert!(symbol.is_mutable());
        assert!(!symbol.is_initialized());
    }

    #[test]
    fn test_scope_shadowing() {
        let mut symbol_table = SymbolTable::new();
        
        // Declare variable in global scope
        let global_var = symbol_table.declare_variable(
            "x".to_string(),
            Type::Integer,
            false,
            None
        ).unwrap();
        
        // Enter function scope
        symbol_table.enter_scope(ScopeKind::Function { name: "test".to_string() });
        
        // Declare variable with same name in function scope
        let local_var = symbol_table.declare_variable(
            "x".to_string(),
            Type::String,
            true,
            None
        ).unwrap();
        
        // Lookup should find local variable
        let found_id = symbol_table.lookup("x").unwrap();
        assert_eq!(found_id, local_var);
        assert_ne!(found_id, global_var);
        
        let found_symbol = symbol_table.get_symbol(found_id).unwrap();
        assert_eq!(found_symbol.symbol_type, Type::String);
        
        // Exit scope
        symbol_table.exit_scope().unwrap();
        
        // Now lookup should find global variable
        let found_id = symbol_table.lookup("x").unwrap();
        assert_eq!(found_id, global_var);
        
        let found_symbol = symbol_table.get_symbol(found_id).unwrap();
        assert_eq!(found_symbol.symbol_type, Type::Integer);
    }

    #[test]
    fn test_redeclaration_error() {
        let mut symbol_table = SymbolTable::new();
        
        // Declare a variable
        symbol_table.declare_variable(
            "x".to_string(),
            Type::Integer,
            false,
            None
        ).unwrap();
        
        // Try to declare same variable again in same scope
        let result = symbol_table.declare_variable(
            "x".to_string(),
            Type::String,
            false,
            None
        );
        
        assert!(result.is_err());
    }

    #[test]
    fn test_function_declaration() {
        let mut symbol_table = SymbolTable::new();
        
        let symbol_id = symbol_table.declare_function(
            "test".to_string(),
            vec![Type::Integer, Type::String],
            Some(Type::Bool),
            None
        ).unwrap();
        
        let symbol = symbol_table.get_symbol(symbol_id).unwrap();
        assert_eq!(symbol.name, "test");
        
        if let SymbolKind::Function { params, return_type } = &symbol.kind {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], Type::Integer);
            assert_eq!(params[1], Type::String);
            assert_eq!(*return_type, Some(Type::Bool));
        } else {
            panic!("Expected function symbol");
        }
    }

    #[test]
    fn test_legacy_variables_conversion() {
        let mut symbol_table = SymbolTable::new();
        
        symbol_table.declare_variable("x".to_string(), Type::Integer, true, None).unwrap();
        symbol_table.declare_variable("name".to_string(), Type::String, false, None).unwrap();
        symbol_table.declare_variable("flag".to_string(), Type::Bool, true, None).unwrap();
        
        let legacy = symbol_table.to_legacy_variables();
        
        assert_eq!(legacy.get("x"), Some(&"int".to_string()));
        assert_eq!(legacy.get("name"), Some(&"char*".to_string()));
        assert_eq!(legacy.get("flag"), Some(&"bool".to_string()));
    }
    
    #[test]
    fn test_unused_symbols() {
        let mut symbol_table = SymbolTable::new();
        
        let used_var = symbol_table.declare_variable("used".to_string(), Type::Integer, true, None).unwrap();
        let unused_var = symbol_table.declare_variable("unused".to_string(), Type::String, false, None).unwrap();
        
        // Mark one as used
        symbol_table.use_symbol(used_var);
        
        let unused = symbol_table.get_unused_symbols();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0], unused_var);
    }
}