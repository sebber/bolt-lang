use std::collections::HashMap;
use crate::ast::{Expression, BinaryOperator, UnaryOperator};

pub struct ExpressionCompiler<'a> {
    pub variables: &'a HashMap<String, String>,
}

impl<'a> ExpressionCompiler<'a> {
    pub fn new(variables: &'a HashMap<String, String>) -> Self {
        Self { variables }
    }
    
    /// Compile an expression to a C expression string
    pub fn compile_to_string(&self, expression: Expression) -> String {
        match expression {
            Expression::IntegerLiteral(n) => n.to_string(),
            Expression::StringLiteral(s) => format!("\"{}\"", s),
            Expression::BoolLiteral(b) => if b { "1".to_string() } else { "0".to_string() },
            Expression::Identifier(name) => name,
            
            Expression::BinaryOp { left, operator, right } => {
                let left_str = self.compile_to_string(*left);
                let right_str = self.compile_to_string(*right);
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
                let operand_str = self.compile_to_string(*operand);
                let op_str = match operator {
                    UnaryOperator::Not => "!",
                };
                format!("({}{})", op_str, operand_str)
            }
            
            Expression::FunctionCall { name, args } => {
                let args_str: Vec<String> = args.into_iter()
                    .map(|arg| self.compile_to_string(arg))
                    .collect();
                format!("{}({})", name, args_str.join(", "))
            }
            
            Expression::FieldAccess { object, field } => {
                let object_str = self.compile_to_string(*object);
                format!("{}.{}", object_str, field)
            }
            
            Expression::ArrayAccess { array, index } => {
                let array_str = self.compile_to_string(*array);
                let index_str = self.compile_to_string(*index);
                format!("{}[{}]", array_str, index_str)
            }
            
            Expression::StructLiteral { type_name, fields, .. } => {
                let field_assignments: Vec<String> = fields.into_iter()
                    .map(|struct_field| {
                        format!(".{} = {}", struct_field.name, self.compile_to_string(struct_field.value))
                    })
                    .collect();
                format!("(({}) {{{}}})", type_name, field_assignments.join(", "))
            }
            
            _ => "0".to_string(), // Fallback for unhandled expression types
        }
    }
    
    /// Generate print statement for an expression
    pub fn generate_print_statement(&self, arg: &Expression) -> String {
        match arg {
            Expression::StringLiteral(s) => {
                format!("    printf(\"%s\\n\", \"{}\");\n", s)
            }
            Expression::IntegerLiteral(n) => {
                format!("    printf(\"%d\\n\", {});\n", n)
            }
            Expression::BoolLiteral(b) => {
                let bool_str = if *b { "true" } else { "false" };
                format!("    printf(\"%s\\n\", \"{}\");\n", bool_str)
            }
            Expression::Identifier(name) => {
                if let Some(var_type) = self.variables.get(name) {
                    match var_type.as_str() {
                        "string" | "char*" => {
                            format!("    printf(\"%s\\n\", {});\n", name)
                        }
                        "int" => {
                            format!("    printf(\"%d\\n\", {});\n", name)
                        }
                        "bool" => {
                            format!("    printf(\"%s\\n\", {} ? \"true\" : \"false\");\n", name)
                        }
                        _ => {
                            format!("    printf(\"%d\\n\", {});\n", name)
                        }
                    }
                } else {
                    // Default to int if type unknown
                    format!("    printf(\"%d\\n\", {});\n", name)
                }
            }
            _ => {
                // Generic fallback for other expression types
                let expr_str = self.compile_to_string(arg.clone());
                format!("    printf(\"%d\\n\", {});\n", expr_str)
            }
        }
    }
}