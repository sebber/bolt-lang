use std::collections::HashMap;
use crate::ast::{Program, Statement, Expression, Type, BinaryOperator, UnaryOperator};
use crate::module::ModuleSystem;

pub struct CCodeGen {
    variables: HashMap<String, String>,
    functions: Vec<String>,
    main_code: String,
}

impl CCodeGen {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: Vec::new(),
            main_code: String::new(),
        }
    }

    pub fn compile_program(&mut self, program: Program) -> String {
        let mut result = String::new();
        result.push_str("#include <stdio.h>\n");
        result.push_str("#include <string.h>\n\n");
        
        // First pass: collect functions, type definitions, and separate main code
        for statement in program.statements {
            match statement {
                Statement::Function { .. } => {
                    self.compile_function(statement);
                }
                Statement::TypeDef { .. } => {
                    self.compile_type_definition(statement, &mut result);
                }
                _ => {
                    self.compile_main_statement(statement);
                }
            }
        }
        
        // Add function declarations
        for func in &self.functions {
            result.push_str(func);
            result.push('\n');
        }
        
        // Add main function
        result.push_str("int main() {\n");
        result.push_str(&self.main_code);
        result.push_str("    return 0;\n");
        result.push_str("}\n");
        
        result
    }

    pub fn compile_program_with_modules(&mut self, program: Program, module_system: &ModuleSystem) -> String {
        let mut result = String::new();
        result.push_str("#include <stdio.h>\n");
        result.push_str("#include <string.h>\n\n");
        
        // Compile functions from all modules first
        self.compile_all_module_functions(module_system, &mut result);
        
        // First pass: collect functions, type definitions, and separate main code
        for statement in program.statements {
            match statement {
                Statement::Function { .. } => {
                    self.compile_function(statement);
                }
                Statement::TypeDef { .. } => {
                    self.compile_type_definition(statement, &mut result);
                }
                Statement::Import { .. } | Statement::Export { .. } => {
                    // Skip import/export statements in code generation
                    // They're handled by the module system
                }
                _ => {
                    self.compile_main_statement(statement);
                }
            }
        }
        
        // Add function declarations
        for func in &self.functions {
            result.push_str(func);
            result.push('\n');
        }
        
        // Add main function
        result.push_str("int main() {\n");
        result.push_str(&self.main_code);
        result.push_str("    return 0;\n");
        result.push_str("}\n");
        
        result
    }

    fn compile_all_module_functions(&mut self, module_system: &ModuleSystem, _result: &mut String) {
        let all_functions = module_system.get_all_functions();
        
        for (function_name, module_path) in all_functions {
            if let Some(module_program) = module_system.get_module(&module_path) {
                for statement in &module_program.statements {
                    if let Statement::Function { name, .. } = statement {
                        if name == &function_name {
                            self.compile_function(statement.clone());
                            break;
                        }
                    }
                }
            }
        }
    }

    fn compile_main_statement(&mut self, statement: Statement) {
        match statement {
            Statement::ValDecl { name, value, .. } => {
                match &value {
                    Expression::StringLiteral(s) => {
                        self.main_code.push_str(&format!("    char {}[] = \"{}\";\n", name, s));
                        self.variables.insert(name, "string".to_string());
                    }
                    Expression::IntegerLiteral(n) => {
                        self.main_code.push_str(&format!("    int {} = {};\n", name, n));
                        self.variables.insert(name, "int".to_string());
                    }
                    Expression::BoolLiteral(b) => {
                        let c_bool = if *b { "1" } else { "0" };
                        self.main_code.push_str(&format!("    int {} = {};\n", name, c_bool));
                        self.variables.insert(name, "bool".to_string());
                    }
                    Expression::FunctionCall { name: func_name, args } => {
                        let call_str = self.compile_expression_to_string(Expression::FunctionCall { 
                            name: func_name.clone(), 
                            args: args.clone() 
                        });
                        self.main_code.push_str(&format!("    int {} = {};\n", name, call_str));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::NamespacedFunctionCall { namespace, function, args } => {
                        let call_str = self.compile_expression_to_string(Expression::NamespacedFunctionCall { 
                            namespace: namespace.clone(), 
                            function: function.clone(),
                            args: args.clone() 
                        });
                        self.main_code.push_str(&format!("    int {} = {};\n", name, call_str));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::ArrayLiteral(elements) => {
                        // For now, assume integer arrays
                        let _size = elements.len();
                        self.main_code.push_str(&format!("    int {}[] = {{", name));
                        
                        for (i, element) in elements.iter().enumerate() {
                            if i > 0 {
                                self.main_code.push_str(", ");
                            }
                            let element_str = self.compile_expression_to_string(element.clone());
                            self.main_code.push_str(&element_str);
                        }
                        
                        self.main_code.push_str("};\n");
                        self.variables.insert(name, "array".to_string());
                    }
                    Expression::BinaryOp { operator, .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        
                        // Check if this is a comparison operation (returns boolean)
                        let var_type = match operator {
                            BinaryOperator::Equal | BinaryOperator::NotEqual |
                            BinaryOperator::Less | BinaryOperator::LessEqual |
                            BinaryOperator::Greater | BinaryOperator::GreaterEqual |
                            BinaryOperator::And | BinaryOperator::Or => "bool",
                            _ => "int"
                        };
                        self.variables.insert(name, var_type.to_string());
                    }
                    Expression::Identifier(var_name) => {
                        self.main_code.push_str(&format!("    int {} = {};\n", name, var_name));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::UnaryOp { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "bool".to_string()); // unary ! always returns bool
                    }
                    Expression::StructLiteral { type_name, .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    {} {} = {};\n", type_name, name, expr_str));
                        self.variables.insert(name, type_name.clone()); // track custom type
                    }
                    Expression::FieldAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for field access
                    }
                    Expression::ArrayAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for array access
                    }
                    Expression::AddressOf { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int* {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int*".to_string()); // pointer to int
                    }
                    Expression::Dereference { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // dereferenced value
                    }
                    _ => {}
                }
            }
            Statement::VarDecl { name, value, .. } => {
                match &value {
                    Expression::StringLiteral(s) => {
                        self.main_code.push_str(&format!("    char {}[] = \"{}\";\n", name, s));
                        self.variables.insert(name, "string".to_string());
                    }
                    Expression::IntegerLiteral(n) => {
                        self.main_code.push_str(&format!("    int {} = {};\n", name, n));
                        self.variables.insert(name, "int".to_string());
                    }
                    Expression::BoolLiteral(b) => {
                        let c_bool = if *b { "1" } else { "0" };
                        self.main_code.push_str(&format!("    int {} = {};\n", name, c_bool));
                        self.variables.insert(name, "bool".to_string());
                    }
                    Expression::FunctionCall { name: func_name, args } => {
                        let call_str = self.compile_expression_to_string(Expression::FunctionCall { 
                            name: func_name.clone(), 
                            args: args.clone() 
                        });
                        self.main_code.push_str(&format!("    int {} = {};\n", name, call_str));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::NamespacedFunctionCall { namespace, function, args } => {
                        let call_str = self.compile_expression_to_string(Expression::NamespacedFunctionCall { 
                            namespace: namespace.clone(), 
                            function: function.clone(),
                            args: args.clone() 
                        });
                        self.main_code.push_str(&format!("    int {} = {};\n", name, call_str));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::ArrayLiteral(elements) => {
                        // For now, assume integer arrays
                        let _size = elements.len();
                        self.main_code.push_str(&format!("    int {}[] = {{", name));
                        
                        for (i, element) in elements.iter().enumerate() {
                            if i > 0 {
                                self.main_code.push_str(", ");
                            }
                            let element_str = self.compile_expression_to_string(element.clone());
                            self.main_code.push_str(&element_str);
                        }
                        
                        self.main_code.push_str("};\n");
                        self.variables.insert(name, "array".to_string());
                    }
                    Expression::BinaryOp { operator, .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        
                        // Check if this is a comparison operation (returns boolean)
                        let var_type = match operator {
                            BinaryOperator::Equal | BinaryOperator::NotEqual |
                            BinaryOperator::Less | BinaryOperator::LessEqual |
                            BinaryOperator::Greater | BinaryOperator::GreaterEqual |
                            BinaryOperator::And | BinaryOperator::Or => "bool",
                            _ => "int"
                        };
                        self.variables.insert(name, var_type.to_string());
                    }
                    Expression::Identifier(var_name) => {
                        self.main_code.push_str(&format!("    int {} = {};\n", name, var_name));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::UnaryOp { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "bool".to_string()); // unary ! always returns bool
                    }
                    Expression::StructLiteral { type_name, .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    {} {} = {};\n", type_name, name, expr_str));
                        self.variables.insert(name, type_name.clone()); // track custom type
                    }
                    Expression::FieldAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for field access
                    }
                    Expression::ArrayAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for array access
                    }
                    Expression::AddressOf { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int* {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int*".to_string()); // pointer to int
                    }
                    Expression::Dereference { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code.push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // dereferenced value
                    }
                    _ => {}
                }
            }
            Statement::If { condition, then_body, else_body } => {
                self.main_code.push_str("    if (");
                self.compile_condition(condition);
                self.main_code.push_str(") {\n");
                
                for stmt in then_body {
                    self.compile_main_statement(stmt);
                }
                
                if let Some(else_stmts) = else_body {
                    self.main_code.push_str("    } else {\n");
                    for stmt in else_stmts {
                        self.compile_main_statement(stmt);
                    }
                }
                
                self.main_code.push_str("    }\n");
            }
            Statement::ForCondition { condition, body } => {
                let condition_str = self.compile_expression_to_string(condition.clone());
                self.main_code.push_str(&format!("    while ({}) {{\n", condition_str));
                for statement in body {
                    self.compile_main_statement_with_indent(statement.clone(), "        ");
                }
                self.main_code.push_str("    }\n");
            }
            Statement::ForLoop { .. } => {
                panic!("C-style for loops not yet implemented in code generator");
            }
            Statement::ForIn { variable, iterable, body } => {
                match iterable {
                    Expression::ArrayLiteral(elements) => {
                        // For array literals, we can generate a simple for loop
                        let array_name = format!("_temp_array_{}", self.variables.len());
                        let size_name = format!("_temp_size_{}", self.variables.len());
                        
                        // Create temporary array
                        self.main_code.push_str(&format!("    int {}[] = {{", array_name));
                        for (i, element) in elements.iter().enumerate() {
                            if i > 0 {
                                self.main_code.push_str(", ");
                            }
                            let element_str = self.compile_expression_to_string(element.clone());
                            self.main_code.push_str(&element_str);
                        }
                        self.main_code.push_str("};\n");
                        
                        let array_size = elements.len();
                        self.main_code.push_str(&format!("    int {} = {};\n", size_name, array_size));
                        
                        // Generate for loop
                        let loop_var = format!("_i_{}", self.variables.len());
                        self.main_code.push_str(&format!("    for (int {} = 0; {} < {}; {}++) {{\n", 
                            loop_var, loop_var, size_name, loop_var));
                        
                        // Declare loop variable
                        self.main_code.push_str(&format!("        int {} = {}[{}];\n", 
                            variable, array_name, loop_var));
                        
                        // Store variable for loop body
                        self.variables.insert(variable.clone(), "int".to_string());
                        
                        // Compile loop body
                        for stmt in body {
                            self.compile_main_statement_with_indent(stmt, "        ");
                        }
                        
                        self.main_code.push_str("    }\n");
                    }
                    Expression::Identifier(array_name) => {
                        // For existing array variables, we need to calculate the size at runtime
                        // We'll use sizeof to get the array size
                        let size_name = format!("_size_of_{}", array_name);
                        let loop_var = format!("_i_for_{}", self.variables.len());
                        
                        self.main_code.push_str(&format!("    int {} = sizeof({}) / sizeof({}[0]);\n", 
                            size_name, array_name, array_name));
                        
                        // Generate for loop
                        self.main_code.push_str(&format!("    for (int {} = 0; {} < {}; {}++) {{\n", 
                            loop_var, loop_var, size_name, loop_var));
                        
                        // Declare loop variable
                        self.main_code.push_str(&format!("        int {} = {}[{}];\n", 
                            variable, array_name, loop_var));
                        
                        // Store variable for loop body
                        self.variables.insert(variable.clone(), "int".to_string());
                        
                        // Compile loop body
                        for stmt in body {
                            self.compile_main_statement_with_indent(stmt, "        ");
                        }
                        
                        self.main_code.push_str("    }\n");
                    }
                    _ => {
                        self.main_code.push_str("    // TODO: for-in with complex expression\n");
                    }
                }
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let return_val = self.compile_expression_to_string(expr);
                    self.main_code.push_str(&format!("    return {};\n", return_val));
                } else {
                    self.main_code.push_str("    return;\n");
                }
            }
            Statement::Assignment { variable, value } => {
                let value_str = self.compile_expression_to_string(value.clone());
                self.main_code.push_str(&format!("    {} = {};\n", variable, value_str));
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr);
            }
            _ => {}
        }
    }

    fn compile_main_statement_with_indent(&mut self, statement: Statement, indent: &str) {
        let old_code = self.main_code.clone();
        self.main_code.clear();
        
        self.compile_main_statement(statement);
        
        let new_code = self.main_code.clone();
        self.main_code = old_code;
        
        // Add the new code with proper indentation
        for line in new_code.lines() {
            if line.trim().is_empty() {
                self.main_code.push('\n');
            } else {
                // Replace the default 4-space indent with our custom indent
                let trimmed = line.trim_start_matches("    ");
                self.main_code.push_str(&format!("{}{}\n", indent, trimmed));
            }
        }
    }

    fn compile_expression(&mut self, expression: Expression) {
        match expression {
            Expression::BoolLiteral(b) => {
                let bool_str = if b { "true" } else { "false" };
                self.main_code.push_str(&format!("    printf(\"%s\\n\", \"{}\");\n", bool_str));
            }
            Expression::FunctionCall { name, args } => {
                let call_str = self.compile_expression_to_string(Expression::FunctionCall { name, args });
                self.main_code.push_str(&format!("    {};\n", call_str));
            }
            Expression::NamespacedFunctionCall { namespace, function, args } => {
                // Handle stdio.print specially to generate printf
                if namespace == "stdio" && function == "print" && args.len() == 1 {
                    let arg = &args[0];
                    match arg {
                        Expression::StringLiteral(s) => {
                            self.main_code.push_str(&format!("    printf(\"%s\\n\", \"{}\");\n", s));
                        }
                        Expression::IntegerLiteral(n) => {
                            self.main_code.push_str(&format!("    printf(\"%d\\n\", {});\n", n));
                        }
                        Expression::Identifier(name) => {
                            if let Some(var_type) = self.variables.get(name) {
                                match var_type.as_str() {
                                    "string" => {
                                        self.main_code.push_str(&format!("    printf(\"%s\\n\", {});\n", name));
                                    }
                                    "int" => {
                                        self.main_code.push_str(&format!("    printf(\"%d\\n\", {});\n", name));
                                    }
                                    "bool" => {
                                        self.main_code.push_str(&format!("    printf(\"%s\\n\", {} ? \"true\" : \"false\");\n", name));
                                    }
                                    _ => {
                                        self.main_code.push_str(&format!("    printf(\"%d\\n\", {});\n", name));
                                    }
                                }
                            } else {
                                // Default to int if type unknown
                                self.main_code.push_str(&format!("    printf(\"%d\\n\", {});\n", name));
                            }
                        }
                        _ => {
                            // Generic fallback for other expression types
                            let expr_str = self.compile_expression_to_string(arg.clone());
                            self.main_code.push_str(&format!("    printf(\"%d\\n\", {});\n", expr_str));
                        }
                    }
                } else {
                    // For other namespaced function calls, just call the function
                    let call_str = self.compile_expression_to_string(Expression::NamespacedFunctionCall { 
                        namespace, function, args 
                    });
                    self.main_code.push_str(&format!("    {};\n", call_str));
                }
            }
            Expression::BinaryOp { left, operator, right } => {
                let result_str = self.compile_expression_to_string(Expression::BinaryOp { 
                    left, operator, right 
                });
                self.main_code.push_str(&format!("    {};\n", result_str));
            }
            _ => {}
        }
    }

    fn compile_condition(&mut self, expression: Expression) {
        match expression {
            Expression::BoolLiteral(b) => {
                let c_bool = if b { "1" } else { "0" };
                self.main_code.push_str(c_bool);
            }
            Expression::Identifier(name) => {
                self.main_code.push_str(&name);
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_str = self.compile_expression_to_string(*left);
                let right_str = self.compile_expression_to_string(*right);
                
                let op_str = match operator {
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                    _ => {
                        // For non-comparison operators, treat as expression
                        let expr_str = self.compile_expression_to_string(Expression::BinaryOp { 
                            left: Box::new(Expression::Identifier(left_str)), 
                            operator, 
                            right: Box::new(Expression::Identifier(right_str)) 
                        });
                        self.main_code.push_str(&format!("({})", expr_str));
                        return;
                    }
                };
                
                self.main_code.push_str(&format!("{} {} {}", left_str, op_str, right_str));
            }
            _ => {
                // For other expressions, compile them as normal expressions
                let expr_str = self.compile_expression_to_string(expression);
                self.main_code.push_str(&expr_str);
            }
        }
    }

    fn compile_function(&mut self, statement: Statement) {
        if let Statement::Function { name, params, return_type, body, exported: _ } = statement {
            // Skip generating C code for stdlib functions that have special implementations
            if name == "print" || name == "println" {
                return;
            }
            
            let mut func_code = String::new();
            
            // Function signature
            let return_type_str = match return_type {
                Some(Type::Integer) => "int",
                Some(Type::String) => "char*",
                Some(Type::Bool) => "int",
                Some(Type::Array(_)) => "int*", // For now, assume int arrays
                Some(Type::Pointer(_)) => "int*", // For now, assume int pointers
                Some(Type::Custom(_)) => "void*",
                None => "void",
            };
            
            func_code.push_str(&format!("{} {}(", return_type_str, name));
            
            // Parameters
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    func_code.push_str(", ");
                }
                let param_type_str = match param.param_type {
                    Type::Integer => "int",
                    Type::String => "char*",
                    Type::Bool => "int",
                    Type::Array(_) => "int*", // For now, assume int arrays
                    Type::Pointer(_) => "int*", // For now, assume int pointers  
                    Type::Custom(_) => "void*",
                };
                func_code.push_str(&format!("{} {}", param_type_str, param.name));
            }
            
            func_code.push_str(") {\n");
            
            // Function body
            let mut temp_codegen = CCodeGen::new();
            
            // Track function parameters in the temporary codegen
            for param in &params {
                let param_type_str = match param.param_type {
                    Type::Integer => "int",
                    Type::String => "string", 
                    Type::Bool => "bool",
                    Type::Array(_) => "array",
                    Type::Pointer(_) => "pointer",
                    Type::Custom(_) => "custom",
                };
                temp_codegen.variables.insert(param.name.clone(), param_type_str.to_string());
            }
            
            for stmt in body {
                match stmt {
                    Statement::Return(expr) => {
                        if let Some(expr) = expr {
                            func_code.push_str("    return ");
                            func_code.push_str(&temp_codegen.compile_expression_to_string(expr));
                            func_code.push_str(";\n");
                        } else {
                            func_code.push_str("    return;\n");
                        }
                    }
                    _ => {
                        temp_codegen.compile_main_statement(stmt);
                        func_code.push_str(&temp_codegen.main_code);
                        temp_codegen.main_code.clear();
                    }
                }
            }
            
            func_code.push_str("}\n");
            self.functions.push(func_code);
        }
    }

    fn compile_type_definition(&mut self, statement: Statement, result: &mut String) {
        if let Statement::TypeDef { name, fields } = statement {
            result.push_str(&format!("typedef struct {{\n"));
            
            for field in &fields {
                let field_type_str = match field.field_type {
                    Type::Integer => "int",
                    Type::String => "char*",
                    Type::Bool => "int",
                    Type::Array(_) => "int*", // For now, assume int arrays
                    Type::Pointer(_) => "int*", // For now, assume int pointers
                    Type::Custom(ref type_name) => type_name, // Reference to other custom type
                };
                result.push_str(&format!("    {} {};\n", field_type_str, field.name));
            }
            
            result.push_str(&format!("}} {};\n\n", name));
        }
    }

    fn compile_expression_to_string(&mut self, expression: Expression) -> String {
        match expression {
            Expression::StringLiteral(s) => format!("\"{}\"", s),
            Expression::IntegerLiteral(n) => n.to_string(),
            Expression::BoolLiteral(b) => if b { "1" } else { "0" }.to_string(),
            Expression::Identifier(name) => name,
            Expression::FunctionCall { name, args } => {
                // Handle stdlib functions specially
                if name == "print" && args.len() == 1 {
                    let arg = args.into_iter().next().unwrap();
                    match arg {
                        Expression::StringLiteral(s) => {
                            format!("printf(\"%s\\n\", \"{}\")", s)
                        }
                        Expression::IntegerLiteral(n) => {
                            format!("printf(\"%d\\n\", {})", n)
                        }
                        Expression::BoolLiteral(b) => {
                            let bool_str = if b { "true" } else { "false" };
                            format!("printf(\"%s\\n\", \"{}\")", bool_str)
                        }
                        Expression::Identifier(var_name) => {
                            // Check variable type from our variables map
                            if let Some(var_type) = self.variables.get(&var_name) {
                                match var_type.as_str() {
                                    "int" => format!("printf(\"%d\\n\", {})", var_name),
                                    "bool" => format!("printf(\"%s\\n\", {} ? \"true\" : \"false\")", var_name),
                                    _ => format!("printf(\"%s\\n\", {})", var_name), // string or unknown
                                }
                            } else {
                                format!("printf(\"%d\\n\", {})", var_name) // default to int
                            }
                        }
                        Expression::FieldAccess { object, field } => {
                            let field_access_str = format!("{}.{}", 
                                self.compile_expression_to_string(*object.clone()), field);
                            
                            // Heuristic: detect field types by name patterns
                            // TODO: Implement proper type tracking for struct fields
                            if field.ends_with("name") || field == "title" || field == "description" {
                                // String fields
                                format!("printf(\"%s\\n\", {})", field_access_str)
                            } else if field == "active" || field == "enabled" || field.starts_with("is") || field.starts_with("has") {
                                // Boolean fields  
                                format!("printf(\"%s\\n\", {} ? \"true\" : \"false\")", field_access_str)
                            } else {
                                // Default to integer
                                format!("printf(\"%d\\n\", {})", field_access_str)
                            }
                        }
                        _ => {
                            let arg_str = self.compile_expression_to_string(arg);
                            format!("printf(\"%d\\n\", {})", arg_str) // default to int for expressions
                        }
                    }
                } else {
                    let mut call = format!("{}(", name);
                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            call.push_str(", ");
                        }
                        call.push_str(&self.compile_expression_to_string(arg));
                    }
                    call.push(')');
                    call
                }
            }
            Expression::NamespacedFunctionCall { namespace, function, args } => {
                // Handle stdio.print specially
                if namespace == "stdio" && function == "print" && args.len() == 1 {
                    let arg = args.into_iter().next().unwrap();
                    match arg {
                        Expression::StringLiteral(s) => {
                            format!("printf(\"%s\\n\", \"{}\")", s)
                        }
                        Expression::IntegerLiteral(n) => {
                            format!("printf(\"%d\\n\", {})", n)
                        }
                        Expression::BoolLiteral(b) => {
                            let bool_str = if b { "true" } else { "false" };
                            format!("printf(\"%s\\n\", \"{}\")", bool_str)
                        }
                        Expression::Identifier(var_name) => {
                            // Check variable type from our variables map
                            if let Some(var_type) = self.variables.get(&var_name) {
                                match var_type.as_str() {
                                    "int" => format!("printf(\"%d\\n\", {})", var_name),
                                    "bool" => format!("printf(\"%s\\n\", {} ? \"true\" : \"false\")", var_name),
                                    _ => format!("printf(\"%s\\n\", {})", var_name), // string or unknown
                                }
                            } else {
                                format!("printf(\"%d\\n\", {})", var_name) // default to int
                            }
                        }
                        Expression::FieldAccess { object, field } => {
                            let field_access_str = format!("{}.{}", 
                                self.compile_expression_to_string(*object.clone()), field);
                            
                            // Heuristic: detect field types by name patterns
                            // TODO: Implement proper type tracking for struct fields
                            if field.ends_with("name") || field == "title" || field == "description" {
                                // String fields
                                format!("printf(\"%s\\n\", {})", field_access_str)
                            } else if field == "active" || field == "enabled" || field.starts_with("is") || field.starts_with("has") {
                                // Boolean fields  
                                format!("printf(\"%s\\n\", {} ? \"true\" : \"false\")", field_access_str)
                            } else {
                                // Default to integer
                                format!("printf(\"%d\\n\", {})", field_access_str)
                            }
                        }
                        _ => {
                            let arg_str = self.compile_expression_to_string(arg);
                            format!("printf(\"%d\\n\", {})", arg_str) // default to int for expressions
                        }
                    }
                } else {
                    // For now, we'll just call the function directly (namespace resolution handled by imports)
                    let mut call = format!("{}(", function);
                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            call.push_str(", ");
                        }
                        call.push_str(&self.compile_expression_to_string(arg));
                    }
                    call.push(')');
                    call
                }
            }
            Expression::ArrayLiteral(elements) => {
                let mut array_str = "{".to_string();
                for (i, element) in elements.into_iter().enumerate() {
                    if i > 0 {
                        array_str.push_str(", ");
                    }
                    array_str.push_str(&self.compile_expression_to_string(element));
                }
                array_str.push('}');
                array_str
            }
            Expression::BinaryOp { left, operator, right } => {
                let left_str = self.compile_expression_to_string(*left);
                let right_str = self.compile_expression_to_string(*right);
                let op_str = match operator {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Modulo => "%",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };
                format!("({} {} {})", left_str, op_str, right_str)
            }
            Expression::UnaryOp { operator, operand } => {
                let operand_str = self.compile_expression_to_string(*operand);
                let op_str = match operator {
                    UnaryOperator::Not => "!",
                };
                format!("({}{})", op_str, operand_str)
            }
            Expression::StructLiteral { type_name, fields } => {
                let mut struct_str = format!("(({}) {{", type_name);
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        struct_str.push_str(", ");
                    }
                    let field_value = self.compile_expression_to_string(field.value.clone());
                    struct_str.push_str(&format!(".{} = {}", field.name, field_value));
                }
                struct_str.push_str("})");
                struct_str
            }
            Expression::FieldAccess { object, field } => {
                let object_str = self.compile_expression_to_string(*object);
                format!("{}.{}", object_str, field)
            }
            Expression::ArrayAccess { array, index } => {
                let array_str = self.compile_expression_to_string(*array);
                let index_str = self.compile_expression_to_string(*index);
                format!("{}[{}]", array_str, index_str)
            }
            Expression::AddressOf { operand } => {
                let operand_str = self.compile_expression_to_string(*operand);
                format!("(&{})", operand_str)
            }
            Expression::Dereference { operand } => {
                let operand_str = self.compile_expression_to_string(*operand);
                format!("(*{})", operand_str)
            }
            _ => "0".to_string(), // fallback
        }
    }
}