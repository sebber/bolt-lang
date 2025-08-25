use crate::ast::{Expression, Program, Statement, Type, BinaryOperator, UnaryOperator};
use crate::module::ModuleSystem;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub is_native: bool,
    pub is_extern: bool,
}

pub struct TypeChecker {
    // Function signatures from all sources
    functions: HashMap<String, FunctionSignature>,
    // Variable types in current scope
    variables: HashMap<String, Type>,
    // Type errors found during checking
    errors: Vec<TypeError>,
    // Module system for resolving imports
    module_system: Option<ModuleSystem>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            functions: HashMap::new(),
            variables: HashMap::new(),
            errors: Vec::new(),
            module_system: None,
        };
        
        // Add built-in functions
        checker.add_builtin_functions();
        checker
    }

    pub fn with_module_system(mut self, module_system: ModuleSystem) -> Self {
        self.module_system = Some(module_system);
        self
    }

    fn add_builtin_functions(&mut self) {
        // Built-in print functions
        self.functions.insert("print".to_string(), FunctionSignature {
            name: "print".to_string(),
            params: vec![("value".to_string(), Type::String)],
            return_type: None,
            is_native: false,
            is_extern: false,
        });

        self.functions.insert("toString".to_string(), FunctionSignature {
            name: "toString".to_string(),
            params: vec![("value".to_string(), Type::Integer)],
            return_type: Some(Type::String),
            is_native: false,
            is_extern: false,
        });

        // Standard library functions (these should come from modules)
        // bolt:string functions
        self.functions.insert("length".to_string(), FunctionSignature {
            name: "length".to_string(),
            params: vec![("s".to_string(), Type::String)],
            return_type: Some(Type::Integer),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("concat".to_string(), FunctionSignature {
            name: "concat".to_string(),
            params: vec![
                ("a".to_string(), Type::String),
                ("b".to_string(), Type::String)
            ],
            return_type: Some(Type::String),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("indexOf".to_string(), FunctionSignature {
            name: "indexOf".to_string(),
            params: vec![
                ("s".to_string(), Type::String),
                ("substr".to_string(), Type::String)
            ],
            return_type: Some(Type::Integer),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("contains".to_string(), FunctionSignature {
            name: "contains".to_string(),
            params: vec![
                ("s".to_string(), Type::String),
                ("substr".to_string(), Type::String)
            ],
            return_type: Some(Type::Bool),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("trim".to_string(), FunctionSignature {
            name: "trim".to_string(),
            params: vec![("s".to_string(), Type::String)],
            return_type: Some(Type::String),
            is_native: true,
            is_extern: false,
        });

        // bolt:io functions  
        self.functions.insert("readFile".to_string(), FunctionSignature {
            name: "readFile".to_string(),
            params: vec![("path".to_string(), Type::String)],
            return_type: Some(Type::String),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("writeFile".to_string(), FunctionSignature {
            name: "writeFile".to_string(),
            params: vec![
                ("path".to_string(), Type::String),
                ("content".to_string(), Type::String)
            ],
            return_type: Some(Type::Bool),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("appendFile".to_string(), FunctionSignature {
            name: "appendFile".to_string(),
            params: vec![
                ("path".to_string(), Type::String),
                ("content".to_string(), Type::String)
            ],
            return_type: Some(Type::Bool),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("fileExists".to_string(), FunctionSignature {
            name: "fileExists".to_string(),
            params: vec![("path".to_string(), Type::String)],
            return_type: Some(Type::Bool),
            is_native: true,
            is_extern: false,
        });

        self.functions.insert("deleteFile".to_string(), FunctionSignature {
            name: "deleteFile".to_string(),
            params: vec![("path".to_string(), Type::String)],
            return_type: Some(Type::Bool),
            is_native: true,
            is_extern: false,
        });
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        // First pass: collect all function signatures
        for statement in &program.statements {
            self.collect_function_signature(statement);
        }

        // Second pass: type check all statements
        for statement in &program.statements {
            self.check_statement(statement);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn collect_function_signature(&mut self, statement: &Statement) {
        match statement {
            Statement::Function { 
                name, 
                params, 
                return_type, 
                exported: _,
                body: _ 
            } => {
                let param_types: Vec<(String, Type)> = params.iter()
                    .map(|p| (p.name.clone(), p.param_type.clone()))
                    .collect();

                self.functions.insert(name.clone(), FunctionSignature {
                    name: name.clone(),
                    params: param_types,
                    return_type: return_type.clone(),
                    is_native: false,
                    is_extern: false,
                });
            }
            Statement::NativeBlock { functions, .. } => {
                for native_func in functions {
                    let param_types: Vec<(String, Type)> = native_func.params.iter()
                        .map(|p| (p.name.clone(), p.param_type.clone()))
                        .collect();

                    self.functions.insert(native_func.name.clone(), FunctionSignature {
                        name: native_func.name.clone(),
                        params: param_types,
                        return_type: native_func.return_type.clone(),
                        is_native: true,
                        is_extern: false,
                    });
                }
            }
            Statement::ExternBlock { functions, .. } => {
                for extern_func in functions {
                    let param_types: Vec<(String, Type)> = extern_func.params.iter()
                        .map(|p| (p.name.clone(), p.param_type.clone()))
                        .collect();

                    self.functions.insert(extern_func.name.clone(), FunctionSignature {
                        name: extern_func.name.clone(),
                        params: param_types,
                        return_type: extern_func.return_type.clone(),
                        is_native: false,
                        is_extern: true,
                    });
                }
            }
            _ => {}
        }
    }

    fn check_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VarDecl { name, type_annotation, value } => {
                let inferred_type = self.infer_expression_type(value);
                
                if let Some(declared_type) = type_annotation {
                    if !self.types_compatible(declared_type, &inferred_type) {
                        self.add_error(format!(
                            "Type mismatch: variable '{}' declared as {:?}, but assigned {:?}",
                            name, declared_type, inferred_type
                        ), 0, 0);
                        return;
                    }
                }

                self.variables.insert(name.clone(), inferred_type);
            }
            Statement::ValDecl { name, type_annotation, value } => {
                let inferred_type = self.infer_expression_type(value);
                
                if let Some(declared_type) = type_annotation {
                    if !self.types_compatible(declared_type, &inferred_type) {
                        self.add_error(format!(
                            "Type mismatch: variable '{}' declared as {:?}, but assigned {:?}",
                            name, declared_type, inferred_type
                        ), 0, 0);
                        return;
                    }
                }

                self.variables.insert(name.clone(), inferred_type);
            }
            Statement::Assignment { variable, value } => {
                if let Some(var_type) = self.variables.get(variable).cloned() {
                    let value_type = self.infer_expression_type(value);
                    if !self.types_compatible(&var_type, &value_type) {
                        self.add_error(format!(
                            "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                            value_type, variable, var_type
                        ), 0, 0);
                    }
                } else {
                    self.add_error(format!("Undefined variable: '{}'", variable), 0, 0);
                }
            }
            Statement::If { condition, then_body, else_body } => {
                let condition_type = self.infer_expression_type(condition);
                if !self.types_compatible(&Type::Bool, &condition_type) {
                    self.add_error(format!(
                        "If condition must be boolean, found {:?}",
                        condition_type
                    ), 0, 0);
                }

                for stmt in then_body {
                    self.check_statement(stmt);
                }

                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.check_statement(stmt);
                    }
                }
            }
            Statement::Function { body, .. } => {
                // Check function body in a new scope
                // For now, we'll check the body directly
                for stmt in body {
                    self.check_statement(stmt);
                }
            }
            Statement::Return(value) => {
                if let Some(expr) = value {
                    // Type check return expression
                    self.infer_expression_type(expr);
                    // TODO: Check against function's declared return type
                }
            }
            Statement::ForIn { variable: _, iterable, body } => {
                // Check that iterable is actually iterable (array)
                let iterable_type = self.infer_expression_type(iterable);
                match iterable_type {
                    Type::Array(_) => {
                        // TODO: Add loop variable to scope
                        for stmt in body {
                            self.check_statement(stmt);
                        }
                    }
                    _ => {
                        self.add_error(format!(
                            "Cannot iterate over {:?}, expected Array",
                            iterable_type
                        ), 0, 0);
                    }
                }
            }
            Statement::Expression(expr) => {
                self.infer_expression_type(expr);
            }
            _ => {
                // Handle other statements
            }
        }
    }

    fn infer_expression_type(&mut self, expression: &Expression) -> Type {
        match expression {
            Expression::StringLiteral(_) => Type::String,
            Expression::IntegerLiteral(_) => Type::Integer,
            Expression::BoolLiteral(_) => Type::Bool,
            Expression::Identifier(name) => {
                if let Some(var_type) = self.variables.get(name) {
                    var_type.clone()
                } else {
                    self.add_error(format!("Undefined variable: '{}'", name), 0, 0);
                    Type::String // Return a default type to continue checking
                }
            }
            Expression::FunctionCall { name, args } => {
                if let Some(func_sig) = self.functions.get(name).cloned() {
                    // Check argument types
                    if args.len() != func_sig.params.len() {
                        self.add_error(format!(
                            "Function '{}' expects {} arguments, found {}",
                            name, func_sig.params.len(), args.len()
                        ), 0, 0);
                    } else {
                        for (arg, (param_name, param_type)) in args.iter().zip(&func_sig.params) {
                            let arg_type = self.infer_expression_type(arg);
                            if !self.types_compatible(param_type, &arg_type) {
                                self.add_error(format!(
                                    "Function '{}' parameter '{}' expects {:?}, found {:?}",
                                    name, param_name, param_type, arg_type
                                ), 0, 0);
                            }
                        }
                    }

                    func_sig.return_type.clone().unwrap_or(Type::String)
                } else {
                    self.add_error(format!("Undefined function: '{}'", name), 0, 0);
                    Type::String // Return a default type to continue checking
                }
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_type = self.infer_expression_type(left);
                let right_type = self.infer_expression_type(right);

                self.check_binary_operation(&left_type, operator, &right_type)
            }
            Expression::UnaryOp { operator, operand } => {
                let operand_type = self.infer_expression_type(operand);
                self.check_unary_operation(operator, &operand_type)
            }
            Expression::ArrayLiteral(elements) => {
                if elements.is_empty() {
                    // Empty array - we can't infer the type
                    Type::Array(Box::new(Type::String)) // Default to string array
                } else {
                    // Infer type from first element, check others are compatible
                    let element_type = self.infer_expression_type(&elements[0]);
                    for (i, element) in elements.iter().enumerate().skip(1) {
                        let elem_type = self.infer_expression_type(element);
                        if !self.types_compatible(&element_type, &elem_type) {
                            self.add_error(format!(
                                "Array element {} has type {:?}, expected {:?}",
                                i, elem_type, element_type
                            ), 0, 0);
                        }
                    }
                    Type::Array(Box::new(element_type))
                }
            }
            Expression::ArrayAccess { array, index } => {
                let array_type = self.infer_expression_type(array);
                let index_type = self.infer_expression_type(index);

                if !self.types_compatible(&Type::Integer, &index_type) {
                    self.add_error(format!(
                        "Array index must be Integer, found {:?}",
                        index_type
                    ), 0, 0);
                }

                match array_type {
                    Type::Array(element_type) => *element_type,
                    _ => {
                        self.add_error(format!(
                            "Cannot index into {:?}, expected Array",
                            array_type
                        ), 0, 0);
                        Type::String // Default type
                    }
                }
            }
            Expression::AddressOf { operand } => {
                let expr_type = self.infer_expression_type(operand);
                Type::Pointer(Box::new(expr_type))
            }
            Expression::Dereference { operand } => {
                let expr_type = self.infer_expression_type(operand);
                match expr_type {
                    Type::Pointer(pointee_type) => *pointee_type,
                    _ => {
                        self.add_error(format!(
                            "Cannot dereference {:?}, expected Pointer",
                            expr_type
                        ), 0, 0);
                        Type::String // Default type
                    }
                }
            }
            _ => {
                // Handle other expression types
                Type::String // Default type for now
            }
        }
    }

    fn check_binary_operation(&mut self, left: &Type, operator: &BinaryOperator, right: &Type) -> Type {
        use BinaryOperator::*;

        match operator {
            Add | Subtract | Multiply | Divide | Modulo => {
                if self.types_compatible(&Type::Integer, left) && self.types_compatible(&Type::Integer, right) {
                    Type::Integer
                } else {
                    self.add_error(format!(
                        "Arithmetic operation requires Integer operands, found {:?} and {:?}",
                        left, right
                    ), 0, 0);
                    Type::Integer
                }
            }
            Equal | NotEqual => {
                if !self.types_compatible(left, right) {
                    self.add_error(format!(
                        "Cannot compare different types: {:?} and {:?}",
                        left, right
                    ), 0, 0);
                }
                Type::Bool
            }
            Less | LessEqual | Greater | GreaterEqual => {
                if self.types_compatible(&Type::Integer, left) && self.types_compatible(&Type::Integer, right) {
                    Type::Bool
                } else {
                    self.add_error(format!(
                        "Comparison requires Integer operands, found {:?} and {:?}",
                        left, right
                    ), 0, 0);
                    Type::Bool
                }
            }
            And | Or => {
                if self.types_compatible(&Type::Bool, left) && self.types_compatible(&Type::Bool, right) {
                    Type::Bool
                } else {
                    self.add_error(format!(
                        "Logical operation requires Bool operands, found {:?} and {:?}",
                        left, right
                    ), 0, 0);
                    Type::Bool
                }
            }
        }
    }

    fn check_unary_operation(&mut self, operator: &UnaryOperator, operand: &Type) -> Type {
        use UnaryOperator::*;

        match operator {
            Not => {
                if self.types_compatible(&Type::Bool, operand) {
                    Type::Bool
                } else {
                    self.add_error(format!(
                        "Logical NOT requires Bool operand, found {:?}",
                        operand
                    ), 0, 0);
                    Type::Bool
                }
            }
        }
    }

    fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        // Basic type compatibility - can be extended for more complex rules
        match (expected, actual) {
            (Type::String, Type::String) => true,
            (Type::Integer, Type::Integer) => true,
            (Type::Bool, Type::Bool) => true,
            (Type::Array(expected_elem), Type::Array(actual_elem)) => {
                self.types_compatible(expected_elem, actual_elem)
            }
            (Type::Pointer(expected_pointee), Type::Pointer(actual_pointee)) => {
                self.types_compatible(expected_pointee, actual_pointee)
            }
            _ => false,
        }
    }

    fn add_error(&mut self, message: String, line: usize, column: usize) {
        self.errors.push(TypeError {
            message,
            line,
            column,
        });
    }

    pub fn get_function_signature(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn get_variable_type(&self, name: &str) -> Option<&Type> {
        self.variables.get(name)
    }

    pub fn get_errors(&self) -> &[TypeError] {
        &self.errors
    }

    pub fn get_all_functions(&self) -> &HashMap<String, FunctionSignature> {
        &self.functions
    }

    pub fn get_all_variables(&self) -> &HashMap<String, Type> {
        &self.variables
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}