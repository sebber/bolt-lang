use crate::ast::{BinaryOperator, Expression, UnaryOperator};
use std::collections::HashMap;

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
            Expression::BoolLiteral(b) => {
                if b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            Expression::Identifier(name) => name,

            Expression::BinaryOp {
                left,
                operator,
                right,
            } => {
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
                let args_str: Vec<String> = args
                    .into_iter()
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

            Expression::StructLiteral {
                type_name, fields, ..
            } => {
                let field_assignments: Vec<String> = fields
                    .into_iter()
                    .map(|struct_field| {
                        format!(
                            ".{} = {}",
                            struct_field.name,
                            self.compile_to_string(struct_field.value)
                        )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, StructField};
    use std::collections::HashMap;

    fn setup_compiler() -> ExpressionCompiler<'static> {
        let mut variables = HashMap::new();
        variables.insert("x".to_string(), "int".to_string());
        variables.insert("name".to_string(), "char*".to_string());
        variables.insert("flag".to_string(), "bool".to_string());

        // Leak the HashMap so we can use it in tests
        let variables = Box::leak(Box::new(variables));
        ExpressionCompiler::new(variables)
    }

    #[test]
    fn test_literal_compilation() {
        let compiler = setup_compiler();

        assert_eq!(
            compiler.compile_to_string(Expression::IntegerLiteral(42)),
            "42"
        );
        assert_eq!(
            compiler.compile_to_string(Expression::StringLiteral("hello".to_string())),
            r#""hello""#
        );
        assert_eq!(
            compiler.compile_to_string(Expression::BoolLiteral(true)),
            "1"
        );
        assert_eq!(
            compiler.compile_to_string(Expression::BoolLiteral(false)),
            "0"
        );
    }

    #[test]
    fn test_identifier_compilation() {
        let compiler = setup_compiler();

        assert_eq!(
            compiler.compile_to_string(Expression::Identifier("myVar".to_string())),
            "myVar"
        );
    }

    #[test]
    fn test_binary_operation_compilation() {
        let compiler = setup_compiler();

        let expr = Expression::BinaryOp {
            left: Box::new(Expression::IntegerLiteral(5)),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::IntegerLiteral(3)),
        };

        assert_eq!(compiler.compile_to_string(expr), "(5 + 3)");
    }

    #[test]
    fn test_all_binary_operators() {
        let compiler = setup_compiler();

        let test_cases = vec![
            (BinaryOperator::Add, "+"),
            (BinaryOperator::Subtract, "-"),
            (BinaryOperator::Multiply, "*"),
            (BinaryOperator::Divide, "/"),
            (BinaryOperator::Modulo, "%"),
            (BinaryOperator::Equal, "=="),
            (BinaryOperator::NotEqual, "!="),
            (BinaryOperator::Less, "<"),
            (BinaryOperator::LessEqual, "<="),
            (BinaryOperator::Greater, ">"),
            (BinaryOperator::GreaterEqual, ">="),
            (BinaryOperator::And, "&&"),
            (BinaryOperator::Or, "||"),
        ];

        for (op, expected_str) in test_cases {
            let expr = Expression::BinaryOp {
                left: Box::new(Expression::IntegerLiteral(1)),
                operator: op,
                right: Box::new(Expression::IntegerLiteral(2)),
            };
            let result = compiler.compile_to_string(expr);
            assert!(
                result.contains(expected_str),
                "Expected {} in {}",
                expected_str,
                result
            );
        }
    }

    #[test]
    fn test_unary_operation_compilation() {
        let compiler = setup_compiler();

        let expr = Expression::UnaryOp {
            operator: UnaryOperator::Not,
            operand: Box::new(Expression::BoolLiteral(true)),
        };

        assert_eq!(compiler.compile_to_string(expr), "(!1)");
    }

    #[test]
    fn test_function_call_compilation() {
        let compiler = setup_compiler();

        let expr = Expression::FunctionCall {
            name: "add".to_string(),
            args: vec![Expression::IntegerLiteral(1), Expression::IntegerLiteral(2)],
        };

        assert_eq!(compiler.compile_to_string(expr), "add(1, 2)");
    }

    #[test]
    fn test_field_access_compilation() {
        let compiler = setup_compiler();

        let expr = Expression::FieldAccess {
            object: Box::new(Expression::Identifier("obj".to_string())),
            field: "name".to_string(),
        };

        assert_eq!(compiler.compile_to_string(expr), "obj.name");
    }

    #[test]
    fn test_array_access_compilation() {
        let compiler = setup_compiler();

        let expr = Expression::ArrayAccess {
            array: Box::new(Expression::Identifier("arr".to_string())),
            index: Box::new(Expression::IntegerLiteral(0)),
        };

        assert_eq!(compiler.compile_to_string(expr), "arr[0]");
    }

    #[test]
    fn test_struct_literal_compilation() {
        let compiler = setup_compiler();

        let expr = Expression::StructLiteral {
            type_name: "Point".to_string(),
            type_args: None,
            fields: vec![
                StructField {
                    name: "x".to_string(),
                    value: Expression::IntegerLiteral(10),
                },
                StructField {
                    name: "y".to_string(),
                    value: Expression::IntegerLiteral(20),
                },
            ],
        };

        let result = compiler.compile_to_string(expr);
        assert!(result.contains("Point"));
        assert!(result.contains(".x = 10"));
        assert!(result.contains(".y = 20"));
    }

    #[test]
    fn test_print_statement_generation() {
        let compiler = setup_compiler();

        // String literal
        let result =
            compiler.generate_print_statement(&Expression::StringLiteral("hello".to_string()));
        assert!(result.contains(r#"printf("%s\n", "hello")"#));

        // Integer literal
        let result = compiler.generate_print_statement(&Expression::IntegerLiteral(42));
        assert!(result.contains("printf(\"%d\\n\", 42)"));

        // Boolean literal
        let result = compiler.generate_print_statement(&Expression::BoolLiteral(true));
        assert!(result.contains(r#"printf("%s\n", "true")"#));
    }

    #[test]
    fn test_print_statement_with_variables() {
        let compiler = setup_compiler();

        // Integer variable
        let result = compiler.generate_print_statement(&Expression::Identifier("x".to_string()));
        assert!(result.contains("printf(\"%d\\n\", x)"));

        // String variable
        let result = compiler.generate_print_statement(&Expression::Identifier("name".to_string()));
        assert!(result.contains("printf(\"%s\\n\", name)"));

        // Boolean variable (should use ternary operator for bool variables)
        let result = compiler.generate_print_statement(&Expression::Identifier("flag".to_string()));
        assert!(result.contains("flag ? \"true\" : \"false\""));
    }
}
