use std::collections::HashMap;
use crate::ast::{Statement, Expression, Type};
use super::expressions::ExpressionCompiler;
use super::types::type_to_c_string;

pub struct StatementCompiler<'a> {
    pub variables: &'a mut HashMap<String, String>,
    pub main_code: &'a mut String,
}

impl<'a> StatementCompiler<'a> {
    pub fn new(variables: &'a mut HashMap<String, String>, main_code: &'a mut String) -> Self {
        Self { variables, main_code }
    }
    
    /// Compile a variable declaration statement
    pub fn compile_var_decl(&mut self, name: &str, var_type: &Option<Type>, value: &Expression) {
        let expr_compiler = ExpressionCompiler::new(self.variables);
        let value_str = expr_compiler.compile_to_string(value.clone());
        
        if let Some(t) = var_type {
            let type_str = type_to_c_string(t);
            self.variables.insert(name.to_string(), type_str.clone());
            self.main_code.push_str(&format!("    {} {} = {};\n", type_str, name, value_str));
        } else {
            // Type inference - for now, assume int for literals, string for string literals
            let inferred_type = match value {
                Expression::IntegerLiteral(_) => "int",
                Expression::StringLiteral(_) => "char*",
                Expression::BoolLiteral(_) => "int",
                _ => "int", // Default fallback
            };
            self.variables.insert(name.to_string(), inferred_type.to_string());
            self.main_code.push_str(&format!("    {} {} = {};\n", inferred_type, name, value_str));
        }
    }
    
    /// Compile a value declaration statement (same as var for C)
    pub fn compile_val_decl(&mut self, name: &str, var_type: &Option<Type>, value: &Expression) {
        // In C, const variables are handled the same way for our purposes
        self.compile_var_decl(name, var_type, value);
    }
    
    /// Compile an assignment statement
    pub fn compile_assignment(&mut self, variable: &str, value: &Expression) {
        let expr_compiler = ExpressionCompiler::new(self.variables);
        let value_str = expr_compiler.compile_to_string(value.clone());
        self.main_code.push_str(&format!("    {} = {};\n", variable, value_str));
    }
    
    /// Compile an if statement
    pub fn compile_if(&mut self, condition: &Expression, then_statements: &[Statement], else_statements: &[Statement]) {
        let expr_compiler = ExpressionCompiler::new(self.variables);
        let condition_str = expr_compiler.compile_to_string(condition.clone());
        
        self.main_code.push_str(&format!("    if ({}) {{\n", condition_str));
        
        // Compile then branch
        for stmt in then_statements {
            self.compile_statement_with_indent(stmt, "    ");
        }
        
        if !else_statements.is_empty() {
            self.main_code.push_str("    } else {\n");
            for stmt in else_statements {
                self.compile_statement_with_indent(stmt, "    ");
            }
        }
        
        self.main_code.push_str("    }\n");
    }
    
    /// Compile a return statement
    pub fn compile_return(&mut self, value: &Option<Expression>) {
        if let Some(expr) = value {
            let expr_compiler = ExpressionCompiler::new(self.variables);
            let value_str = expr_compiler.compile_to_string(expr.clone());
            self.main_code.push_str(&format!("    return {};\n", value_str));
        } else {
            self.main_code.push_str("    return;\n");
        }
    }
    
    /// Compile a statement with additional indentation
    pub fn compile_statement_with_indent(&mut self, statement: &Statement, base_indent: &str) {
        let indent = format!("{}    ", base_indent);
        
        match statement {
            Statement::VarDecl { name, var_type, value } => {
                let expr_compiler = ExpressionCompiler::new(self.variables);
                let value_str = expr_compiler.compile_to_string(value.clone());
                
                if let Some(t) = var_type {
                    let type_str = type_to_c_string(t);
                    self.variables.insert(name.to_string(), type_str.clone());
                    self.main_code.push_str(&format!("{}{} {} = {};\n", indent, type_str, name, value_str));
                } else {
                    let inferred_type = match value.as_ref() {
                        Expression::IntegerLiteral(_) => "int",
                        Expression::StringLiteral(_) => "char*", 
                        Expression::BoolLiteral(_) => "int",
                        _ => "int",
                    };
                    self.variables.insert(name.to_string(), inferred_type.to_string());
                    self.main_code.push_str(&format!("{}{} {} = {};\n", indent, inferred_type, name, value_str));
                }
            }
            
            Statement::ValDecl { name, var_type, value } => {
                self.compile_statement_with_indent(&Statement::VarDecl { 
                    name: name.clone(), 
                    var_type: var_type.clone(), 
                    value: value.clone() 
                }, base_indent);
            }
            
            Statement::Assignment { variable, value } => {
                let expr_compiler = ExpressionCompiler::new(self.variables);
                let value_str = expr_compiler.compile_to_string(value.clone());
                self.main_code.push_str(&format!("{}{} = {};\n", indent, variable, value_str));
            }
            
            Statement::Expression(expr) => {
                match expr {
                    Expression::NamespacedFunctionCall { namespace, function, args } => {
                        if namespace == "stdio" && function == "print" && args.len() == 1 {
                            let expr_compiler = ExpressionCompiler::new(self.variables);
                            let print_stmt = expr_compiler.generate_print_statement(&args[0]);
                            self.main_code.push_str(&format!("{}{}", indent.trim_end(), print_stmt));
                        }
                    }
                    _ => {
                        let expr_compiler = ExpressionCompiler::new(self.variables);
                        let expr_str = expr_compiler.compile_to_string(expr.clone());
                        self.main_code.push_str(&format!("{}{};\n", indent, expr_str));
                    }
                }
            }
            
            Statement::Return { value } => {
                if let Some(expr) = value {
                    let expr_compiler = ExpressionCompiler::new(self.variables);
                    let value_str = expr_compiler.compile_to_string(expr.clone());
                    self.main_code.push_str(&format!("{}return {};\n", indent, value_str));
                } else {
                    self.main_code.push_str(&format!("{}return;\n", indent));
                }
            }
            
            _ => {
                // For other statement types, we'd need to implement them
                self.main_code.push_str(&format!("{}/* Unhandled statement type */;\n", indent));
            }
        }
    }
}