use crate::ast::{BinaryOperator, Expression, Field, Program, Statement, Type, UnaryOperator};
use crate::module::ModuleSystem;
use crate::symbol_table::SymbolTable;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MonomorphicType {
    pub base_name: String,      // e.g., "Array", "Map"
    pub type_args: Vec<String>, // e.g., ["Integer"], ["String", "Integer"]
}

impl MonomorphicType {
    pub fn new(base_name: String, type_args: Vec<String>) -> Self {
        Self {
            base_name,
            type_args,
        }
    }

    pub fn mangled_name(&self) -> String {
        if self.type_args.is_empty() {
            self.base_name.clone()
        } else {
            format!("{}_{}", self.base_name, self.type_args.join("_"))
        }
    }
}

pub struct CCodeGen {
    variables: HashMap<String, String>,
    functions: Vec<String>,
    main_code: String,
    symbol_table: SymbolTable,
    has_user_main: bool, // Track if user defined a main function
    array_lengths: HashMap<String, usize>, // Track array lengths for .length property
    // Monomorphization state
    generic_types: HashMap<String, (Vec<String>, Vec<Field>)>, // base_name -> (type_params, fields)
    required_monomorphs: HashSet<MonomorphicType>, // Track which concrete types are needed
    generated_monomorphs: HashMap<MonomorphicType, String>, // Cache generated C code
}

impl CCodeGen {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            functions: Vec::new(),
            main_code: String::new(),
            symbol_table: SymbolTable::new(),
            has_user_main: false,
            array_lengths: HashMap::new(),
            generic_types: HashMap::new(),
            required_monomorphs: HashSet::new(),
            generated_monomorphs: HashMap::new(),
        }
    }

    pub fn with_symbol_table(symbol_table: SymbolTable) -> Self {
        let variables = symbol_table.to_legacy_variables();
        Self {
            variables,
            functions: Vec::new(),
            main_code: String::new(),
            symbol_table,
            has_user_main: false,
            array_lengths: HashMap::new(),
            generic_types: HashMap::new(),
            required_monomorphs: HashSet::new(),
            generated_monomorphs: HashMap::new(),
        }
    }

    // Add a generic type definition to the registry
    fn register_generic_type(
        &mut self,
        name: String,
        type_params: Vec<String>,
        fields: Vec<Field>,
    ) {
        self.generic_types.insert(name, (type_params, fields));
    }

    // Helper function to check if an expression represents a string
    fn is_string_expression(&self, expr: &Expression) -> bool {
        match expr {
            Expression::StringLiteral(_) => true,
            Expression::Identifier(name) => {
                // Check if the variable is known to be a string type
                self.variables
                    .get(name)
                    .map_or(false, |t| t == "string" || t == "char*")
            }
            _ => false, // For now, only handle literals and variables
        }
    }

    // Mark a concrete generic type as needed for generation
    fn require_monomorph(&mut self, base_name: String, type_args: Vec<String>) {
        let monomorph = MonomorphicType::new(base_name, type_args);
        self.required_monomorphs.insert(monomorph);
    }

    // Generate C struct for a specific monomorphic type
    fn generate_monomorphic_struct(&mut self, monomorph: &MonomorphicType) -> String {
        if let Some(cached) = self.generated_monomorphs.get(monomorph) {
            return cached.clone();
        }

        if let Some((type_params, fields)) = self.generic_types.get(&monomorph.base_name) {
            let mut result = String::new();
            let struct_name = monomorph.mangled_name();

            result.push_str("typedef struct {\n");

            for field in fields {
                let concrete_type = self.substitute_type_params(
                    &field.field_type,
                    type_params,
                    &monomorph.type_args,
                );
                let field_type_str = self.type_to_c_string(&concrete_type);
                result.push_str(&format!("    {} {};\n", field_type_str, field.name));
            }

            result.push_str(&format!("}} {};\n\n", struct_name));

            self.generated_monomorphs
                .insert(monomorph.clone(), result.clone());
            result
        } else {
            panic!("Unknown generic type: {}", monomorph.base_name);
        }
    }

    // Substitute type parameters with concrete types
    fn substitute_type_params(
        &self,
        generic_type: &Type,
        type_params: &[String],
        concrete_types: &[String],
    ) -> Type {
        match generic_type {
            Type::Custom(name) => {
                // Check if this is a type parameter
                if let Some(index) = type_params.iter().position(|param| param == name) {
                    if let Some(concrete_type) = concrete_types.get(index) {
                        // Convert concrete type name to Type enum
                        match concrete_type.as_str() {
                            "Integer" => Type::Integer,
                            "String" => Type::String,
                            "Bool" => Type::Bool,
                            _ => Type::Custom(concrete_type.clone()),
                        }
                    } else {
                        generic_type.clone()
                    }
                } else {
                    generic_type.clone()
                }
            }
            Type::Pointer(inner) => {
                let substituted_inner =
                    self.substitute_type_params(inner.as_ref(), type_params, concrete_types);
                Type::Pointer(Box::new(substituted_inner))
            }
            Type::Generic {
                name,
                type_params: inner_params,
            } => {
                let substituted_params: Vec<Type> = inner_params
                    .iter()
                    .map(|param| self.substitute_type_params(param, type_params, concrete_types))
                    .collect();
                Type::Generic {
                    name: name.clone(),
                    type_params: substituted_params,
                }
            }
            _ => generic_type.clone(),
        }
    }

    // Convert Type to C type string
    fn type_to_c_string(&self, t: &Type) -> String {
        match t {
            Type::Integer => "int".to_string(),
            Type::String => "char*".to_string(),
            Type::Bool => "int".to_string(),
            Type::Pointer(inner) => format!("{}*", self.type_to_c_string(inner.as_ref())),
            Type::Custom(name) => name.clone(),
            Type::Generic { name, type_params } => {
                // Generate monomorphic type name
                let type_arg_names: Vec<String> = type_params
                    .iter()
                    .map(|t| match t {
                        Type::Integer => "Integer".to_string(),
                        Type::String => "String".to_string(),
                        Type::Bool => "Bool".to_string(),
                        Type::Custom(n) => n.clone(),
                        _ => "Unknown".to_string(), // TODO: Handle nested generics
                    })
                    .collect();
                MonomorphicType::new(name.clone(), type_arg_names).mangled_name()
            }
            _ => "void*".to_string(),
        }
    }

    // Analyze statement for generic type usage
    fn analyze_statement_for_generic_usage(&mut self, statement: &Statement) {
        match statement {
            Statement::VarDecl {
                type_annotation: Some(t),
                value,
                ..
            }
            | Statement::ValDecl {
                type_annotation: Some(t),
                value,
                ..
            } => {
                self.analyze_type_for_generic_usage(t);
                self.analyze_expression_for_generic_usage(value);
            }
            Statement::VarDecl { value, .. } | Statement::ValDecl { value, .. } => {
                self.analyze_expression_for_generic_usage(value);
            }
            Statement::Function {
                params,
                return_type,
                ..
            } => {
                for param in params {
                    self.analyze_type_for_generic_usage(&param.param_type);
                }
                if let Some(ret_type) = return_type {
                    self.analyze_type_for_generic_usage(ret_type);
                }
            }
            Statement::Expression(expr) => {
                self.analyze_expression_for_generic_usage(expr);
            }
            _ => {
                // Other statement types don't contain type information
            }
        }
    }

    // Analyze expression for generic type usage
    fn analyze_expression_for_generic_usage(&mut self, expr: &Expression) {
        match expr {
            Expression::StructLiteral {
                type_name,
                type_args,
                fields,
            } => {
                // If this is a generic struct literal, register the monomorphic type
                if let Some(args) = type_args {
                    let type_arg_names: Vec<String> = args
                        .iter()
                        .map(|t| match t {
                            Type::Integer => "Integer".to_string(),
                            Type::String => "String".to_string(),
                            Type::Bool => "Bool".to_string(),
                            Type::Custom(n) => n.clone(),
                            _ => "Unknown".to_string(),
                        })
                        .collect();
                    self.require_monomorph(type_name.clone(), type_arg_names);
                }

                // Recursively analyze field expressions
                for field in fields {
                    self.analyze_expression_for_generic_usage(&field.value);
                }
            }
            Expression::FunctionCall { args, .. } => {
                for arg in args {
                    self.analyze_expression_for_generic_usage(arg);
                }
            }
            Expression::NamespacedFunctionCall { args, .. } => {
                for arg in args {
                    self.analyze_expression_for_generic_usage(arg);
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.analyze_expression_for_generic_usage(left);
                self.analyze_expression_for_generic_usage(right);
            }
            Expression::UnaryOp { operand, .. } => {
                self.analyze_expression_for_generic_usage(operand);
            }
            Expression::FieldAccess { object, .. } => {
                self.analyze_expression_for_generic_usage(object);
            }
            Expression::ArrayAccess { array, index } => {
                self.analyze_expression_for_generic_usage(array);
                self.analyze_expression_for_generic_usage(index);
            }
            Expression::AddressOf { operand } => {
                self.analyze_expression_for_generic_usage(operand);
            }
            Expression::Dereference { operand } => {
                self.analyze_expression_for_generic_usage(operand);
            }
            _ => {
                // Other expression types don't contain type information
            }
        }
    }

    // Analyze type for generic usage and register required monomorphs
    fn analyze_type_for_generic_usage(&mut self, t: &Type) {
        match t {
            Type::Generic { name, type_params } => {
                let type_arg_names: Vec<String> = type_params
                    .iter()
                    .map(|param_type| match param_type {
                        Type::Integer => "Integer".to_string(),
                        Type::String => "String".to_string(),
                        Type::Bool => "Bool".to_string(),
                        Type::Custom(n) => n.clone(),
                        Type::Generic { .. } => {
                            // Recursively analyze nested generic types
                            self.analyze_type_for_generic_usage(param_type);
                            self.type_to_c_string(param_type) // Use the mangled name
                        }
                        _ => "Unknown".to_string(),
                    })
                    .collect();

                self.require_monomorph(name.clone(), type_arg_names);
            }
            Type::Pointer(inner) => {
                self.analyze_type_for_generic_usage(inner.as_ref());
            }
            _ => {
                // Other types don't need monomorphization
            }
        }
    }

    // Generate all required monomorphic types
    fn generate_all_monomorphs(&mut self) -> String {
        let mut result = String::new();
        let required_types: Vec<MonomorphicType> =
            self.required_monomorphs.iter().cloned().collect();

        for monomorph in required_types {
            let struct_code = self.generate_monomorphic_struct(&monomorph);
            result.push_str(&struct_code);
        }

        result
    }

    pub fn compile_program(&mut self, program: Program) -> String {
        let mut result = String::new();
        result.push_str("#include <stdio.h>\n");
        result.push_str("#include <string.h>\n");
        result.push_str("#include <stdlib.h>\n\n");

        // Helper function for string concatenation
        result.push_str("char* string_concat(const char* str1, const char* str2) {\n");
        result.push_str("    size_t len1 = strlen(str1);\n");
        result.push_str("    size_t len2 = strlen(str2);\n");
        result.push_str("    char* result = malloc(len1 + len2 + 1);\n");
        result.push_str("    strcpy(result, str1);\n");
        result.push_str("    strcat(result, str2);\n");
        result.push_str("    return result;\n");
        result.push_str("}\n\n");

        // Helper function for integer to string conversion
        result.push_str("char* toString(int value) {\n");
        result.push_str("    char* result = malloc(32); // enough for any 32-bit int\n");
        result.push_str("    snprintf(result, 32, \"%d\", value);\n");
        result.push_str("    return result;\n");
        result.push_str("}\n\n");

        // Global variables for command line arguments
        result.push_str("int bolt_argc;\n");
        result.push_str("char** bolt_argv;\n\n");

        // Helper functions for command line arguments
        result.push_str("char** getArgs() {\n");
        result.push_str("    return bolt_argv;\n");
        result.push_str("}\n\n");
        result.push_str("int getArgsLength() {\n");
        result.push_str("    return bolt_argc;\n");
        result.push_str("}\n\n");

        // Compile functions from all modules first

        // Pass 1: Collect type definitions and analyze usage
        let mut remaining_statements = Vec::new();
        for statement in program.statements {
            match statement {
                Statement::TypeDef { .. } => {
                    self.compile_type_definition(statement, &mut result);
                }
                _ => {
                    remaining_statements.push(statement);
                }
            }
        }

        // Pass 2: Analyze remaining statements for generic type usage
        for statement in &remaining_statements {
            self.analyze_statement_for_generic_usage(statement);
        }

        // Pass 3: Generate all required monomorphic types
        let monomorphic_code = self.generate_all_monomorphs();
        result.push_str(&monomorphic_code);

        // Pass 4: Generate functions and main code
        for statement in remaining_statements {
            match statement {
                Statement::Function { .. } => {
                    self.compile_function(statement);
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
        if self.has_user_main {
            // User defined main function exists, create wrapper that calls it
            result.push_str("int main(int argc, char* argv[]) {\n");
            result.push_str("    bolt_argc = argc;\n");
            result.push_str("    bolt_argv = argv;\n");
            result.push_str("    bolt_main();\n");
            result.push_str("    return 0;\n");
            result.push_str("}\n");
        } else {
            // No user main function, put top-level code in main
            result.push_str("int main(int argc, char* argv[]) {\n");
            result.push_str("    bolt_argc = argc;\n");
            result.push_str("    bolt_argv = argv;\n");
            result.push_str(&self.main_code);
            result.push_str("    return 0;\n");
            result.push_str("}\n");
        }

        result
    }

    pub fn compile_program_with_modules(
        &mut self,
        program: Program,
        module_system: &ModuleSystem,
    ) -> String {
        let mut result = String::new();
        result.push_str("#include <stdio.h>\n");
        result.push_str("#include <string.h>\n");
        result.push_str("#include <stdlib.h>\n\n");

        // Helper function for string concatenation
        result.push_str("char* string_concat(const char* str1, const char* str2) {\n");
        result.push_str("    size_t len1 = strlen(str1);\n");
        result.push_str("    size_t len2 = strlen(str2);\n");
        result.push_str("    char* result = malloc(len1 + len2 + 1);\n");
        result.push_str("    strcpy(result, str1);\n");
        result.push_str("    strcat(result, str2);\n");
        result.push_str("    return result;\n");
        result.push_str("}\n\n");

        // Helper function for integer to string conversion
        result.push_str("char* toString(int value) {\n");
        result.push_str("    char* result = malloc(32); // enough for any 32-bit int\n");
        result.push_str("    snprintf(result, 32, \"%d\", value);\n");
        result.push_str("    return result;\n");
        result.push_str("}\n\n");

        // Global variables for command line arguments
        result.push_str("int bolt_argc;\n");
        result.push_str("char** bolt_argv;\n\n");

        // Helper functions for command line arguments
        result.push_str("char** getArgs() {\n");
        result.push_str("    return bolt_argv;\n");
        result.push_str("}\n\n");
        result.push_str("int getArgsLength() {\n");
        result.push_str("    return bolt_argc;\n");
        result.push_str("}\n\n");

        // Compile functions from all modules first
        self.compile_all_module_functions(module_system, &mut result);

        // Pass 1: Collect type definitions and analyze usage
        let mut remaining_statements = Vec::new();
        for statement in program.statements {
            match statement {
                Statement::TypeDef { .. } => {
                    self.compile_type_definition(statement, &mut result);
                }
                Statement::Import { .. } | Statement::Export { .. } => {
                    // Skip import/export statements in code generation
                    // They're handled by the module system
                }
                _ => {
                    remaining_statements.push(statement);
                }
            }
        }

        // Pass 2: Analyze remaining statements for generic type usage
        for statement in &remaining_statements {
            self.analyze_statement_for_generic_usage(statement);
        }

        // Pass 3: Generate all required monomorphic types
        let monomorphic_code = self.generate_all_monomorphs();
        result.push_str(&monomorphic_code);

        // Pass 4: Generate functions and main code
        for statement in remaining_statements {
            match statement {
                Statement::Function { .. } => {
                    self.compile_function(statement);
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
        if self.has_user_main {
            // User defined main function exists, create wrapper that calls it
            result.push_str("int main(int argc, char* argv[]) {\n");
            result.push_str("    bolt_argc = argc;\n");
            result.push_str("    bolt_argv = argv;\n");
            result.push_str("    bolt_main();\n");
            result.push_str("    return 0;\n");
            result.push_str("}\n");
        } else {
            // No user main function, put top-level code in main
            result.push_str("int main(int argc, char* argv[]) {\n");
            result.push_str("    bolt_argc = argc;\n");
            result.push_str("    bolt_argv = argv;\n");
            result.push_str(&self.main_code);
            result.push_str("    return 0;\n");
            result.push_str("}\n");
        }

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
                        self.main_code
                            .push_str(&format!("    char {}[] = \"{}\";\n", name, s));
                        self.variables.insert(name, "string".to_string());
                    }
                    Expression::IntegerLiteral(n) => {
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, n));
                        self.variables.insert(name, "int".to_string());
                    }
                    Expression::BoolLiteral(b) => {
                        let c_bool = if *b { "1" } else { "0" };
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, c_bool));
                        self.variables.insert(name, "bool".to_string());
                    }
                    Expression::FunctionCall {
                        name: func_name,
                        args,
                    } => {
                        let call_str =
                            self.compile_expression_to_string(Expression::FunctionCall {
                                name: func_name.clone(),
                                args: args.clone(),
                            });
                        // Check if this is toString function call
                        if func_name == "toString" {
                            self.main_code
                                .push_str(&format!("    char* {} = {};\n", name, call_str));
                            self.variables.insert(name, "string".to_string());
                        } else if func_name == "getArgs" {
                            self.main_code
                                .push_str(&format!("    char** {} = {};\n", name, call_str));
                            self.variables.insert(name, "getargs".to_string());
                        } else {
                            self.main_code
                                .push_str(&format!("    int {} = {};\n", name, call_str));
                            self.variables.insert(name, "int".to_string()); // assume int for now
                        }
                    }
                    Expression::NamespacedFunctionCall {
                        namespace,
                        function,
                        args,
                    } => {
                        let call_str =
                            self.compile_expression_to_string(Expression::NamespacedFunctionCall {
                                namespace: namespace.clone(),
                                function: function.clone(),
                                args: args.clone(),
                            });
                        // Check if this is toString function call
                        if function == "toString" {
                            self.main_code
                                .push_str(&format!("    char* {} = {};\n", name, call_str));
                            self.variables.insert(name, "string".to_string());
                        } else if function == "getArgs" {
                            self.main_code
                                .push_str(&format!("    char** {} = {};\n", name, call_str));
                            self.variables.insert(name, "getargs".to_string());
                        } else {
                            self.main_code
                                .push_str(&format!("    int {} = {};\n", name, call_str));
                            self.variables.insert(name, "int".to_string()); // assume int for now
                        }
                    }
                    Expression::ArrayLiteral(elements) => {
                        // For now, assume integer arrays
                        let size = elements.len();
                        self.array_lengths.insert(name.clone(), size); // Store array length
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
                    Expression::BinaryOp {
                        operator,
                        left,
                        right,
                    } => {
                        let expr_str = self.compile_expression_to_string(value.clone());

                        // Determine the result type and C type
                        let (c_type, var_type) = if *operator == BinaryOperator::Add
                            && (self.is_string_expression(left) || self.is_string_expression(right))
                        {
                            ("char*", "string")
                        } else {
                            match operator {
                                BinaryOperator::Equal
                                | BinaryOperator::NotEqual
                                | BinaryOperator::Less
                                | BinaryOperator::LessEqual
                                | BinaryOperator::Greater
                                | BinaryOperator::GreaterEqual
                                | BinaryOperator::And
                                | BinaryOperator::Or => ("int", "bool"),
                                _ => ("int", "int"),
                            }
                        };

                        self.main_code
                            .push_str(&format!("    {} {} = {};\n", c_type, name, expr_str));
                        self.variables.insert(name, var_type.to_string());
                    }
                    Expression::Identifier(var_name) => {
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, var_name));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::UnaryOp { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "bool".to_string()); // unary ! always returns bool
                    }
                    Expression::StructLiteral {
                        type_name,
                        type_args,
                        ..
                    } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        let var_type = if let Some(args) = type_args {
                            // Generate monomorphic type name for generic structs
                            let type_arg_names: Vec<String> = args
                                .iter()
                                .map(|t| match t {
                                    Type::Integer => "Integer".to_string(),
                                    Type::String => "String".to_string(),
                                    Type::Bool => "Bool".to_string(),
                                    Type::Custom(n) => n.clone(),
                                    _ => "Unknown".to_string(),
                                })
                                .collect();
                            MonomorphicType::new(type_name.clone(), type_arg_names).mangled_name()
                        } else {
                            type_name.clone()
                        };
                        self.main_code
                            .push_str(&format!("    {} {} = {};\n", var_type, name, expr_str));
                        self.variables.insert(name, var_type); // track custom type
                    }
                    Expression::FieldAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for field access
                    }
                    Expression::ArrayAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for array access
                    }
                    Expression::AddressOf { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int* {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int*".to_string()); // pointer to int
                    }
                    Expression::Dereference { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // dereferenced value
                    }
                    _ => {}
                }
            }
            Statement::VarDecl { name, value, .. } => {
                match &value {
                    Expression::StringLiteral(s) => {
                        self.main_code
                            .push_str(&format!("    char {}[] = \"{}\";\n", name, s));
                        self.variables.insert(name, "string".to_string());
                    }
                    Expression::IntegerLiteral(n) => {
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, n));
                        self.variables.insert(name, "int".to_string());
                    }
                    Expression::BoolLiteral(b) => {
                        let c_bool = if *b { "1" } else { "0" };
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, c_bool));
                        self.variables.insert(name, "bool".to_string());
                    }
                    Expression::FunctionCall {
                        name: func_name,
                        args,
                    } => {
                        let call_str =
                            self.compile_expression_to_string(Expression::FunctionCall {
                                name: func_name.clone(),
                                args: args.clone(),
                            });
                        // Check if this is toString function call
                        if func_name == "toString" {
                            self.main_code
                                .push_str(&format!("    char* {} = {};\n", name, call_str));
                            self.variables.insert(name, "string".to_string());
                        } else if func_name == "getArgs" {
                            self.main_code
                                .push_str(&format!("    char** {} = {};\n", name, call_str));
                            self.variables.insert(name, "getargs".to_string());
                        } else {
                            self.main_code
                                .push_str(&format!("    int {} = {};\n", name, call_str));
                            self.variables.insert(name, "int".to_string()); // assume int for now
                        }
                    }
                    Expression::NamespacedFunctionCall {
                        namespace,
                        function,
                        args,
                    } => {
                        let call_str =
                            self.compile_expression_to_string(Expression::NamespacedFunctionCall {
                                namespace: namespace.clone(),
                                function: function.clone(),
                                args: args.clone(),
                            });
                        // Check if this is toString function call
                        if function == "toString" {
                            self.main_code
                                .push_str(&format!("    char* {} = {};\n", name, call_str));
                            self.variables.insert(name, "string".to_string());
                        } else if function == "getArgs" {
                            self.main_code
                                .push_str(&format!("    char** {} = {};\n", name, call_str));
                            self.variables.insert(name, "getargs".to_string());
                        } else {
                            self.main_code
                                .push_str(&format!("    int {} = {};\n", name, call_str));
                            self.variables.insert(name, "int".to_string()); // assume int for now
                        }
                    }
                    Expression::ArrayLiteral(elements) => {
                        // For now, assume integer arrays
                        let size = elements.len();
                        self.array_lengths.insert(name.clone(), size); // Store array length
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
                    Expression::BinaryOp {
                        operator,
                        left,
                        right,
                    } => {
                        let expr_str = self.compile_expression_to_string(value.clone());

                        // Determine the result type and C type
                        let (c_type, var_type) = if *operator == BinaryOperator::Add
                            && (self.is_string_expression(left) || self.is_string_expression(right))
                        {
                            ("char*", "string")
                        } else {
                            match operator {
                                BinaryOperator::Equal
                                | BinaryOperator::NotEqual
                                | BinaryOperator::Less
                                | BinaryOperator::LessEqual
                                | BinaryOperator::Greater
                                | BinaryOperator::GreaterEqual
                                | BinaryOperator::And
                                | BinaryOperator::Or => ("int", "bool"),
                                _ => ("int", "int"),
                            }
                        };

                        self.main_code
                            .push_str(&format!("    {} {} = {};\n", c_type, name, expr_str));
                        self.variables.insert(name, var_type.to_string());
                    }
                    Expression::Identifier(var_name) => {
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, var_name));
                        self.variables.insert(name, "int".to_string()); // assume int for now
                    }
                    Expression::UnaryOp { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "bool".to_string()); // unary ! always returns bool
                    }
                    Expression::StructLiteral {
                        type_name,
                        type_args,
                        ..
                    } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        let var_type = if let Some(args) = type_args {
                            // Generate monomorphic type name for generic structs
                            let type_arg_names: Vec<String> = args
                                .iter()
                                .map(|t| match t {
                                    Type::Integer => "Integer".to_string(),
                                    Type::String => "String".to_string(),
                                    Type::Bool => "Bool".to_string(),
                                    Type::Custom(n) => n.clone(),
                                    _ => "Unknown".to_string(),
                                })
                                .collect();
                            MonomorphicType::new(type_name.clone(), type_arg_names).mangled_name()
                        } else {
                            type_name.clone()
                        };
                        self.main_code
                            .push_str(&format!("    {} {} = {};\n", var_type, name, expr_str));
                        self.variables.insert(name, var_type); // track custom type
                    }
                    Expression::FieldAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for field access
                    }
                    Expression::ArrayAccess { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // assume int for array access
                    }
                    Expression::AddressOf { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int* {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int*".to_string()); // pointer to int
                    }
                    Expression::Dereference { .. } => {
                        let expr_str = self.compile_expression_to_string(value.clone());
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", name, expr_str));
                        self.variables.insert(name, "int".to_string()); // dereferenced value
                    }
                    _ => {}
                }
            }
            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
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
                self.main_code
                    .push_str(&format!("    while ({}) {{\n", condition_str));
                for statement in body {
                    self.compile_main_statement_with_indent(statement.clone(), "        ");
                }
                self.main_code.push_str("    }\n");
            }
            Statement::ForLoop { .. } => {
                panic!("C-style for loops not yet implemented in code generator");
            }
            Statement::ForIn {
                variable,
                iterable,
                body,
            } => {
                match iterable {
                    Expression::ArrayLiteral(elements) => {
                        // For array literals, we can generate a simple for loop
                        let array_name = format!("_temp_array_{}", self.variables.len());
                        let size_name = format!("_temp_size_{}", self.variables.len());

                        // Create temporary array
                        self.main_code
                            .push_str(&format!("    int {}[] = {{", array_name));
                        for (i, element) in elements.iter().enumerate() {
                            if i > 0 {
                                self.main_code.push_str(", ");
                            }
                            let element_str = self.compile_expression_to_string(element.clone());
                            self.main_code.push_str(&element_str);
                        }
                        self.main_code.push_str("};\n");

                        let array_size = elements.len();
                        self.main_code
                            .push_str(&format!("    int {} = {};\n", size_name, array_size));

                        // Generate for loop
                        let loop_var = format!("_i_{}", self.variables.len());
                        self.main_code.push_str(&format!(
                            "    for (int {} = 0; {} < {}; {}++) {{\n",
                            loop_var, loop_var, size_name, loop_var
                        ));

                        // Declare loop variable
                        self.main_code.push_str(&format!(
                            "        int {} = {}[{}];\n",
                            variable, array_name, loop_var
                        ));

                        // Store variable for loop body
                        self.variables.insert(variable.clone(), "int".to_string());

                        // Compile loop body
                        for stmt in body {
                            self.compile_main_statement_with_indent(stmt, "        ");
                        }

                        self.main_code.push_str("    }\n");
                    }
                    Expression::Identifier(array_name) => {
                        let array_type = self
                            .variables
                            .get(&array_name)
                            .unwrap_or(&"unknown".to_string())
                            .clone();

                        let loop_var = format!("_i_for_{}", self.variables.len());

                        // Check if this is an Array[T] type
                        if array_type.contains("Array_") {
                            // For Array[T] types, use array.length
                            self.main_code.push_str(&format!(
                                "    for (int {} = 0; {} < {}.length; {}++) {{\n",
                                loop_var, loop_var, array_name, loop_var
                            ));

                            // Extract element type from Array_Type name
                            let element_type = if array_type.contains("Array_Integer") {
                                "int"
                            } else if array_type.contains("Array_String") {
                                "char*"
                            } else if array_type.contains("Array_Bool") {
                                "int" // bool as int
                            } else if array_type.starts_with("Array_") {
                                // Custom type like Array_Person -> Person
                                &array_type[6..] // Remove "Array_" prefix
                            } else {
                                "int" // default
                            };

                            // Declare loop variable - access via array.data[index]
                            self.main_code.push_str(&format!(
                                "        {} {} = {}.data[{}];\n",
                                element_type, variable, array_name, loop_var
                            ));

                            self.variables
                                .insert(variable.clone(), element_type.to_string());
                        } else if array_type == "getargs" {
                            // Special handling for getArgs array (char**)
                            self.main_code.push_str(&format!(
                                "    for (int {} = 0; {} < getArgsLength(); {}++) {{\n",
                                loop_var, loop_var, loop_var
                            ));

                            // Declare loop variable as char*
                            self.main_code.push_str(&format!(
                                "        char* {} = {}[{}];\n",
                                variable, array_name, loop_var
                            ));

                            self.variables
                                .insert(variable.clone(), "string".to_string());
                        } else {
                            // For regular arrays, use sizeof
                            let size_name = format!("_size_of_{}", array_name);

                            self.main_code.push_str(&format!(
                                "    int {} = sizeof({}) / sizeof({}[0]);\n",
                                size_name, array_name, array_name
                            ));

                            // Generate for loop
                            self.main_code.push_str(&format!(
                                "    for (int {} = 0; {} < {}; {}++) {{\n",
                                loop_var, loop_var, size_name, loop_var
                            ));

                            // Declare loop variable
                            self.main_code.push_str(&format!(
                                "        int {} = {}[{}];\n",
                                variable, array_name, loop_var
                            ));

                            // Store variable for loop body
                            self.variables.insert(variable.clone(), "int".to_string());
                        }

                        // Compile loop body
                        for stmt in body {
                            self.compile_main_statement_with_indent(stmt, "        ");
                        }

                        self.main_code.push_str("    }\n");
                    }
                    Expression::FunctionCall { name, args } => {
                        if name == "iterate" && args.len() == 1 {
                            // Handle iterate(array) pattern
                            match &args[0] {
                                Expression::Identifier(array_name) => {
                                    let array_type = self
                                        .variables
                                        .get(array_name)
                                        .unwrap_or(&"int*".to_string())
                                        .clone();

                                    let loop_var = format!("_i_iter_{}", self.variables.len());

                                    // For Array[T] types, use array.length
                                    if array_type.contains("Array_") {
                                        self.main_code.push_str(&format!(
                                            "    for (int {} = 0; {} < {}.length; {}++) {{\n",
                                            loop_var, loop_var, array_name, loop_var
                                        ));

                                        // Extract element type from Array_Type name
                                        let element_type = if array_type.contains("Array_Integer") {
                                            "int"
                                        } else if array_type.contains("Array_String") {
                                            "char*"
                                        } else if array_type.contains("Array_Bool") {
                                            "int" // bool as int
                                        } else if array_type.starts_with("Array_") {
                                            // Custom type like Array_Person -> Person
                                            &array_type[6..] // Remove "Array_" prefix
                                        } else {
                                            "int" // default
                                        };

                                        // Declare loop variable - access via array.data[index]
                                        self.main_code.push_str(&format!(
                                            "        {} {} = {}.data[{}];\n",
                                            element_type, variable, array_name, loop_var
                                        ));

                                        self.variables
                                            .insert(variable.clone(), element_type.to_string());
                                    } else {
                                        // For regular arrays, use sizeof
                                        let size_name = format!("_size_of_{}", array_name);
                                        self.main_code.push_str(&format!(
                                            "    int {} = sizeof({}) / sizeof({}[0]);\n",
                                            size_name, array_name, array_name
                                        ));

                                        self.main_code.push_str(&format!(
                                            "    for (int {} = 0; {} < {}; {}++) {{\n",
                                            loop_var, loop_var, size_name, loop_var
                                        ));

                                        self.main_code.push_str(&format!(
                                            "        int {} = {}[{}];\n",
                                            variable, array_name, loop_var
                                        ));

                                        self.variables.insert(variable.clone(), "int".to_string());
                                    }

                                    // Compile loop body
                                    for stmt in body {
                                        self.compile_main_statement_with_indent(stmt, "        ");
                                    }

                                    self.main_code.push_str("    }\n");
                                }
                                _ => {
                                    panic!("iterate() only supports identifier arguments for now");
                                }
                            }
                        } else {
                            panic!(
                                "For-in loops with function calls only support iterate() for now"
                            );
                        }
                    }
                    _ => {
                        self.main_code
                            .push_str("    // TODO: for-in with complex expression\n");
                    }
                }
            }
            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    let return_val = self.compile_expression_to_string(expr);
                    self.main_code
                        .push_str(&format!("    return {};\n", return_val));
                } else {
                    self.main_code.push_str("    return;\n");
                }
            }
            Statement::Assignment { variable, value } => {
                let value_str = self.compile_expression_to_string(value.clone());
                self.main_code
                    .push_str(&format!("    {} = {};\n", variable, value_str));
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
                self.main_code
                    .push_str(&format!("    printf(\"%s\\n\", \"{}\");\n", bool_str));
            }
            Expression::FunctionCall { name, args } => {
                let call_str =
                    self.compile_expression_to_string(Expression::FunctionCall { name, args });
                self.main_code.push_str(&format!("    {};\n", call_str));
            }
            Expression::NamespacedFunctionCall {
                namespace,
                function,
                args,
            } => {
                // Handle stdio.print specially to generate printf
                if namespace == "stdio" && function == "print" && args.len() == 1 {
                    let arg = &args[0];
                    match arg {
                        Expression::StringLiteral(s) => {
                            self.main_code
                                .push_str(&format!("    printf(\"%s\\n\", \"{}\");\n", s));
                        }
                        Expression::IntegerLiteral(n) => {
                            self.main_code
                                .push_str(&format!("    printf(\"%d\\n\", {});\n", n));
                        }
                        Expression::Identifier(name) => {
                            if let Some(var_type) = self.variables.get(name) {
                                match var_type.as_str() {
                                    "string" => {
                                        self.main_code.push_str(&format!(
                                            "    printf(\"%s\\n\", {});\n",
                                            name
                                        ));
                                    }
                                    "int" => {
                                        self.main_code.push_str(&format!(
                                            "    printf(\"%d\\n\", {});\n",
                                            name
                                        ));
                                    }
                                    "bool" => {
                                        self.main_code.push_str(&format!(
                                            "    printf(\"%s\\n\", {} ? \"true\" : \"false\");\n",
                                            name
                                        ));
                                    }
                                    _ => {
                                        self.main_code.push_str(&format!(
                                            "    printf(\"%d\\n\", {});\n",
                                            name
                                        ));
                                    }
                                }
                            } else {
                                // Default to int if type unknown
                                self.main_code
                                    .push_str(&format!("    printf(\"%d\\n\", {});\n", name));
                            }
                        }
                        _ => {
                            // Generic fallback for other expression types
                            let expr_str = self.compile_expression_to_string(arg.clone());
                            self.main_code
                                .push_str(&format!("    printf(\"%d\\n\", {});\n", expr_str));
                        }
                    }
                } else {
                    // For other namespaced function calls, just call the function
                    let call_str =
                        self.compile_expression_to_string(Expression::NamespacedFunctionCall {
                            namespace,
                            function,
                            args,
                        });
                    self.main_code.push_str(&format!("    {};\n", call_str));
                }
            }
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let result_str = self.compile_expression_to_string(Expression::BinaryOp {
                    left,
                    operator,
                    right,
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
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
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
                            right: Box::new(Expression::Identifier(right_str)),
                        });
                        self.main_code.push_str(&format!("({})", expr_str));
                        return;
                    }
                };

                self.main_code
                    .push_str(&format!("{} {} {}", left_str, op_str, right_str));
            }
            _ => {
                // For other expressions, compile them as normal expressions
                let expr_str = self.compile_expression_to_string(expression);
                self.main_code.push_str(&expr_str);
            }
        }
    }

    fn compile_function(&mut self, statement: Statement) {
        if let Statement::Function {
            name,
            params,
            return_type,
            body,
            exported: _,
        } = statement
        {
            // Skip generating C code for stdlib functions that have special implementations
            if name == "print" || name == "println" {
                return;
            }

            // Check if this is a user-defined main function
            if name == "main" {
                self.has_user_main = true;
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
                Some(Type::Generic { .. }) => "void*", // TODO: Implement generic return types
                Some(Type::TypeParameter(_)) => "void*", // TODO: Implement type parameter return types
                None => "void",
            };

            // Rename user's main function to avoid conflict with C main
            let c_function_name = if name == "main" { "bolt_main" } else { &name };
            func_code.push_str(&format!("{} {}(", return_type_str, c_function_name));

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
                    Type::Generic { .. } => "void*", // TODO: Implement generic type handling
                    Type::TypeParameter(_) => "void*", // TODO: Implement type parameter handling
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
                    Type::Generic { .. } => "generic", // TODO: Implement generic type handling
                    Type::TypeParameter(_) => "typeparam", // TODO: Implement type parameter handling
                };
                temp_codegen
                    .variables
                    .insert(param.name.clone(), param_type_str.to_string());
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
        if let Statement::TypeDef {
            name,
            type_params,
            fields,
        } = statement
        {
            if !type_params.is_empty() {
                // This is a generic type definition - register it for monomorphization
                self.register_generic_type(name.clone(), type_params, fields);
                // Don't generate C code yet - wait for concrete instantiations
            } else {
                // This is a regular (non-generic) type definition
                result.push_str("typedef struct {\n");

                for field in &fields {
                    let field_type_str = self.type_to_c_string(&field.field_type);
                    result.push_str(&format!("    {} {};\n", field_type_str, field.name));
                }

                result.push_str(&format!("}} {};\n\n", name));
            }
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
                if name == "toString" && args.len() == 1 {
                    let arg = args.into_iter().next().unwrap();
                    let arg_str = self.compile_expression_to_string(arg);
                    format!("toString({})", arg_str)
                } else if name == "getArgs" && args.is_empty() {
                    "getArgs()".to_string()
                } else if name == "print" && args.len() == 1 {
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
                                    "bool" => format!(
                                        "printf(\"%s\\n\", {} ? \"true\" : \"false\")",
                                        var_name
                                    ),
                                    _ => format!("printf(\"%s\\n\", {})", var_name), // string or unknown
                                }
                            } else {
                                format!("printf(\"%d\\n\", {})", var_name) // default to int
                            }
                        }
                        Expression::FieldAccess { object, field } => {
                            let field_access_str =
                                self.compile_expression_to_string(Expression::FieldAccess {
                                    object: object.clone(),
                                    field: field.clone(),
                                });

                            // Heuristic: detect field types by name patterns
                            // TODO: Implement proper type tracking for struct fields
                            if field.ends_with("name") || field == "title" || field == "description"
                            {
                                // String fields
                                format!("printf(\"%s\\n\", {})", field_access_str)
                            } else if field == "active"
                                || field == "enabled"
                                || field.starts_with("is")
                                || field.starts_with("has")
                            {
                                // Boolean fields
                                format!(
                                    "printf(\"%s\\n\", {} ? \"true\" : \"false\")",
                                    field_access_str
                                )
                            } else {
                                // Default to integer
                                format!("printf(\"%d\\n\", {})", field_access_str)
                            }
                        }
                        _ => {
                            // Check if this is a toString function call
                            if let Expression::FunctionCall { name, .. } = &arg {
                                if name == "toString" {
                                    let arg_str = self.compile_expression_to_string(arg);
                                    return format!("printf(\"%s\\n\", {})", arg_str);
                                }
                            }
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
            Expression::NamespacedFunctionCall {
                namespace,
                function,
                args,
            } => {
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
                                    "bool" => format!(
                                        "printf(\"%s\\n\", {} ? \"true\" : \"false\")",
                                        var_name
                                    ),
                                    _ => format!("printf(\"%s\\n\", {})", var_name), // string or unknown
                                }
                            } else {
                                format!("printf(\"%d\\n\", {})", var_name) // default to int
                            }
                        }
                        Expression::FieldAccess { object, field } => {
                            let field_access_str =
                                self.compile_expression_to_string(Expression::FieldAccess {
                                    object: object.clone(),
                                    field: field.clone(),
                                });

                            // Heuristic: detect field types by name patterns
                            // TODO: Implement proper type tracking for struct fields
                            if field.ends_with("name") || field == "title" || field == "description"
                            {
                                // String fields
                                format!("printf(\"%s\\n\", {})", field_access_str)
                            } else if field == "active"
                                || field == "enabled"
                                || field.starts_with("is")
                                || field.starts_with("has")
                            {
                                // Boolean fields
                                format!(
                                    "printf(\"%s\\n\", {} ? \"true\" : \"false\")",
                                    field_access_str
                                )
                            } else {
                                // Default to integer
                                format!("printf(\"%d\\n\", {})", field_access_str)
                            }
                        }
                        _ => {
                            // Check if this is a toString function call
                            if let Expression::FunctionCall { name, .. } = &arg {
                                if name == "toString" {
                                    let arg_str = self.compile_expression_to_string(arg);
                                    return format!("printf(\"%s\\n\", {})", arg_str);
                                }
                            }
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
            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_str = self.compile_expression_to_string(*left.clone());
                let right_str = self.compile_expression_to_string(*right.clone());

                // Check for string concatenation
                if operator == BinaryOperator::Add
                    && (self.is_string_expression(&left) || self.is_string_expression(&right))
                {
                    // Generate string concatenation call
                    return format!("string_concat({}, {})", left_str, right_str);
                }

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
            Expression::StructLiteral {
                type_name,
                type_args,
                fields,
            } => {
                let struct_type = if let Some(args) = type_args {
                    // Generate monomorphic type name for generic structs
                    let type_arg_names: Vec<String> = args
                        .iter()
                        .map(|t| match t {
                            Type::Integer => "Integer".to_string(),
                            Type::String => "String".to_string(),
                            Type::Bool => "Bool".to_string(),
                            Type::Custom(n) => n.clone(),
                            _ => "Unknown".to_string(),
                        })
                        .collect();
                    MonomorphicType::new(type_name.clone(), type_arg_names).mangled_name()
                } else {
                    type_name.clone()
                };
                let mut struct_str = format!("(({}) {{", struct_type);
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
                let object_str = self.compile_expression_to_string(*object.clone());

                // Special handling for .length property
                if field == "length" {
                    // Check if this is accessing length on a known identifier
                    if let Expression::Identifier(var_name) = object.as_ref() {
                        // Check if it's an array with known length
                        if let Some(&array_len) = self.array_lengths.get(var_name) {
                            return array_len.to_string();
                        }
                        // Check if it's a string variable
                        if let Some(var_type) = self.variables.get(var_name) {
                            if var_type == "char*" || var_type == "string" {
                                return format!("strlen({})", object_str);
                            } else if var_type == "getargs" {
                                // Check if this is the getArgs array
                                return "getArgsLength()".to_string();
                            }
                        }
                    }
                    // For string literals, calculate length at compile time
                    if let Expression::StringLiteral(ref string_val) = object.as_ref() {
                        return string_val.len().to_string();
                    }
                }

                // Default struct field access
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expression, Program, Statement, Type};
    use std::collections::HashMap;

    fn setup_codegen() -> CCodeGen {
        CCodeGen::new()
    }

    #[test]
    fn test_getargs_function_call() {
        let mut codegen = setup_codegen();

        let expr = Expression::FunctionCall {
            name: "getArgs".to_string(),
            args: vec![],
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "getArgs()");
    }

    #[test]
    fn test_tostring_function_call() {
        let mut codegen = setup_codegen();

        let expr = Expression::FunctionCall {
            name: "toString".to_string(),
            args: vec![Expression::IntegerLiteral(42)],
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "toString(42)");
    }

    #[test]
    fn test_getargs_length_property() {
        let mut codegen = setup_codegen();
        // Register getArgs as getargs type
        codegen
            .variables
            .insert("args".to_string(), "getargs".to_string());

        let expr = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("args".to_string())),
            field: "length".to_string(),
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "getArgsLength()");
    }

    #[test]
    fn test_array_length_property() {
        let mut codegen = setup_codegen();
        // Register array with known length
        codegen.array_lengths.insert("numbers".to_string(), 5);

        let expr = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("numbers".to_string())),
            field: "length".to_string(),
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "5");
    }

    #[test]
    fn test_string_length_property() {
        let mut codegen = setup_codegen();
        // Register string variable
        codegen
            .variables
            .insert("name".to_string(), "string".to_string());

        let expr = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("name".to_string())),
            field: "length".to_string(),
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "strlen(name)");
    }

    #[test]
    fn test_string_literal_length() {
        let mut codegen = setup_codegen();

        let expr = Expression::FieldAccess {
            object: Box::new(Expression::StringLiteral("hello".to_string())),
            field: "length".to_string(),
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "5");
    }

    #[test]
    fn test_compile_program_includes_getargs_helpers() {
        let mut codegen = setup_codegen();
        let program = Program { statements: vec![] };

        let result = codegen.compile_program(program);

        // Check that helper functions are included
        assert!(result.contains("int bolt_argc;"));
        assert!(result.contains("char** bolt_argv;"));
        assert!(result.contains("char** getArgs() {"));
        assert!(result.contains("int getArgsLength() {"));
        assert!(result.contains("return bolt_argv;"));
        assert!(result.contains("return bolt_argc;"));
    }

    #[test]
    fn test_main_function_sets_globals() {
        let mut codegen = setup_codegen();
        let program = Program { statements: vec![] };

        let result = codegen.compile_program(program);

        // Check that main function sets global variables
        assert!(result.contains("int main(int argc, char* argv[]) {"));
        assert!(result.contains("bolt_argc = argc;"));
        assert!(result.contains("bolt_argv = argv;"));
    }

    #[test]
    fn test_getargs_variable_type_tracking() {
        let mut codegen = setup_codegen();

        let val_decl = Statement::ValDecl {
            name: "args".to_string(),
            type_annotation: None,
            value: Expression::FunctionCall {
                name: "getArgs".to_string(),
                args: vec![],
            },
        };

        codegen.compile_main_statement(val_decl);

        // Check that getArgs result is tracked as getargs type
        assert_eq!(codegen.variables.get("args"), Some(&"getargs".to_string()));
        assert!(codegen.main_code.contains("char** args = getArgs();"));
    }

    #[test]
    fn test_for_in_getargs_compilation() {
        let mut codegen = setup_codegen();
        // First declare args variable
        codegen
            .variables
            .insert("args".to_string(), "getargs".to_string());

        let for_in = Statement::ForIn {
            variable: "arg".to_string(),
            iterable: Expression::Identifier("args".to_string()),
            body: vec![Statement::Expression(Expression::NamespacedFunctionCall {
                namespace: "stdio".to_string(),
                function: "print".to_string(),
                args: vec![Expression::Identifier("arg".to_string())],
            })],
        };

        codegen.compile_main_statement(for_in);

        // Check for-in loop with getArgsLength()
        assert!(codegen.main_code.contains("getArgsLength()"));
        assert!(codegen.main_code.contains("char* arg = args["));
        assert!(codegen.main_code.contains("printf(\"%s\\n\", arg);"));
    }

    #[test]
    fn test_toString_variable_type_tracking() {
        let mut codegen = setup_codegen();

        let val_decl = Statement::ValDecl {
            name: "numStr".to_string(),
            type_annotation: None,
            value: Expression::FunctionCall {
                name: "toString".to_string(),
                args: vec![Expression::IntegerLiteral(42)],
            },
        };

        codegen.compile_main_statement(val_decl);

        // Check that toString result is tracked as string type
        assert_eq!(codegen.variables.get("numStr"), Some(&"string".to_string()));
        assert!(codegen.main_code.contains("char* numStr = toString(42);"));
    }

    #[test]
    fn test_nested_length_property() {
        let mut codegen = setup_codegen();
        codegen
            .variables
            .insert("args".to_string(), "getargs".to_string());

        // Test toString(args.length)
        let expr = Expression::FunctionCall {
            name: "toString".to_string(),
            args: vec![Expression::FieldAccess {
                object: Box::new(Expression::Identifier("args".to_string())),
                field: "length".to_string(),
            }],
        };

        let result = codegen.compile_expression_to_string(expr);
        assert_eq!(result, "toString(getArgsLength())");
    }
}
