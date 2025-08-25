use crate::ast::{
    BinaryOperator, Expression, NativeFunction, Field, Parameter, Program, Statement, StructField, Type,
    UnaryOperator,
};
use crate::lexer::{Token, TokenType};
use crate::symbol_table::{ScopeKind, SymbolTable};

pub type ParseResult<T> = std::result::Result<T, String>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    symbol_table: SymbolTable,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            symbol_table: SymbolTable::new(),
        }
    }

    /// Get a reference to the symbol table after parsing
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Take ownership of the symbol table (consumes the parser)
    pub fn into_symbol_table(self) -> SymbolTable {
        self.symbol_table
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if self.peek().token_type == TokenType::Newline {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement());
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Statement {
        match &self.peek().token_type {
            TokenType::Var => self.parse_var_decl(),
            TokenType::Val => self.parse_val_decl(),
            TokenType::Type => self.parse_type_def(),
            TokenType::If => self.parse_if_statement(),
            TokenType::For => self.parse_for_loop(),
            TokenType::Fun => self.parse_function(false),
            TokenType::Return => self.parse_return(),
            TokenType::Import => self.parse_import(),
            TokenType::Export => self.parse_export(),
            TokenType::Native => self.parse_native_block(),
            _ => {
                // Could be assignment or expression
                // Look ahead to see if it's an assignment
                if matches!(self.peek().token_type, TokenType::Identifier(_)) {
                    let next_idx = self.current + 1;
                    if next_idx < self.tokens.len()
                        && self.tokens[next_idx].token_type == TokenType::Equal
                    {
                        // It's an assignment
                        let name = match &self.advance().token_type {
                            TokenType::Identifier(n) => n.clone(),
                            _ => unreachable!(),
                        };
                        self.advance(); // consume '='
                        let value = self.parse_expression();
                        Statement::Assignment {
                            variable: name,
                            value,
                        }
                    } else {
                        Statement::Expression(self.parse_expression())
                    }
                } else {
                    Statement::Expression(self.parse_expression())
                }
            }
        }
    }

    fn parse_var_decl(&mut self) -> Statement {
        self.advance(); // consume 'var'

        let name = match &self.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => panic!("Expected identifier after 'var'"),
        };

        let mut type_annotation = None;
        let value;

        if self.peek().token_type == TokenType::Colon {
            self.advance(); // consume ':'
            type_annotation = Some(self.parse_type());

            if self.peek().token_type == TokenType::Equal {
                self.advance(); // consume '='
                value = self.parse_expression();
            } else {
                panic!("Expected '=' after type annotation");
            }
        } else if self.peek().token_type == TokenType::ColonEqual {
            self.advance(); // consume ':='
            value = self.parse_expression();
        } else {
            panic!("Expected ':' or ':=' after variable name");
        }

        // Register variable in symbol table
        let var_type = if let Some(ref explicit_type) = type_annotation {
            explicit_type.clone()
        } else {
            // For now, we'll infer type from value during semantic analysis
            // For parsing stage, we'll use a placeholder
            Type::Custom("inferred".to_string())
        };

        // Register the variable as mutable in the symbol table
        if let Err(e) = self
            .symbol_table
            .declare_variable(name.clone(), var_type, true, None)
        {
            panic!("Error declaring variable '{}': {}", name, e);
        }

        Statement::VarDecl {
            name,
            type_annotation,
            value,
        }
    }

    fn parse_val_decl(&mut self) -> Statement {
        self.advance(); // consume 'val'

        let name = match &self.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => panic!("Expected identifier after 'val'"),
        };

        let mut type_annotation = None;
        let value;

        if self.peek().token_type == TokenType::Colon {
            self.advance(); // consume ':'
            type_annotation = Some(self.parse_type());

            if self.peek().token_type == TokenType::Equal {
                self.advance(); // consume '='
                value = self.parse_expression();
            } else {
                panic!("Expected '=' after type annotation");
            }
        } else if self.peek().token_type == TokenType::ColonEqual {
            self.advance(); // consume ':='
            value = self.parse_expression();
        } else {
            panic!("Expected ':' or ':=' after variable name");
        }

        // Register variable in symbol table
        let var_type = if let Some(ref explicit_type) = type_annotation {
            explicit_type.clone()
        } else {
            // For now, we'll infer type from value during semantic analysis
            Type::Custom("inferred".to_string())
        };

        // Register the variable as immutable in the symbol table
        if let Err(e) = self
            .symbol_table
            .declare_variable(name.clone(), var_type, false, None)
        {
            panic!("Error declaring variable '{}': {}", name, e);
        }

        Statement::ValDecl {
            name,
            type_annotation,
            value,
        }
    }

    fn parse_type_def(&mut self) -> Statement {
        self.advance(); // consume 'type'

        let name = match &self.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => panic!("Expected identifier after 'type'"),
        };

        // Parse generic type parameters: type Name[T, K] = { ... }
        let mut type_params = Vec::new();
        if self.peek().token_type == TokenType::LeftBracket {
            self.advance(); // consume '['

            while self.peek().token_type != TokenType::RightBracket && !self.is_at_end() {
                match &self.advance().token_type {
                    TokenType::Identifier(param_name) => {
                        type_params.push(param_name.clone());
                    }
                    _ => panic!("Expected type parameter name"),
                }

                if self.peek().token_type == TokenType::Comma {
                    self.advance(); // consume ','
                } else if self.peek().token_type != TokenType::RightBracket {
                    panic!("Expected ',' or ']' in type parameter list");
                }
            }

            if self.peek().token_type != TokenType::RightBracket {
                panic!("Expected ']' after type parameters");
            }
            self.advance(); // consume ']'
        }

        if self.peek().token_type != TokenType::Equal {
            panic!("Expected '=' after type name");
        }
        self.advance(); // consume '='

        if self.peek().token_type != TokenType::LeftBrace {
            panic!("Expected '{{' after '='");
        }
        self.advance(); // consume '{'

        let mut fields = Vec::new();

        while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
            // Skip newlines
            if self.peek().token_type == TokenType::Newline {
                self.advance();
                continue;
            }

            let field_name = match &self.advance().token_type {
                TokenType::Identifier(name) => name.clone(),
                _ => panic!("Expected field name"),
            };

            if self.peek().token_type != TokenType::Colon {
                panic!("Expected ':' after field name");
            }
            self.advance(); // consume ':'

            let field_type = self.parse_type();

            fields.push(Field {
                name: field_name,
                field_type,
            });

            if self.peek().token_type == TokenType::Comma {
                self.advance(); // consume ','
            }
        }

        if self.peek().token_type != TokenType::RightBrace {
            panic!("Expected '}}'");
        }
        self.advance(); // consume '}'

        Statement::TypeDef {
            name,
            type_params,
            fields,
        }
    }

    fn parse_type(&mut self) -> Type {
        // Check for pointer type prefix (^)
        if self.peek().token_type == TokenType::Caret {
            self.advance(); // consume '^'
            let pointee_type = self.parse_type();
            return Type::Pointer(Box::new(pointee_type));
        }

        let token = self.advance().clone();
        match &token.token_type {
            TokenType::Identifier(name) => {
                match name.as_str() {
                    "String" => Type::String,
                    "Integer" => Type::Integer,
                    "Bool" => Type::Bool,
                    _ => {
                        // Check if this is a generic type like Array[T] or Map[K, V]
                        if self.peek().token_type == TokenType::LeftBracket {
                            self.advance(); // consume '['

                            let mut type_params = Vec::new();
                            while self.peek().token_type != TokenType::RightBracket
                                && !self.is_at_end()
                            {
                                let param_type = self.parse_type();
                                type_params.push(param_type);

                                if self.peek().token_type == TokenType::Comma {
                                    self.advance(); // consume ','
                                } else if self.peek().token_type != TokenType::RightBracket {
                                    panic!("Expected ',' or ']' in generic type parameter list");
                                }
                            }

                            if self.peek().token_type != TokenType::RightBracket {
                                panic!("Expected ']' after generic type parameters");
                            }
                            self.advance(); // consume ']'

                            Type::Generic {
                                name: name.clone(),
                                type_params,
                            }
                        } else {
                            // Could be a custom type or a type parameter
                            Type::Custom(name.clone())
                        }
                    }
                }
            }
            _ => panic!("Expected type name"),
        }
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Expression {
        let mut expr = self.parse_logical_and();

        while matches!(self.peek().token_type, TokenType::OrOr) {
            self.advance(); // consume '||'
            let right = self.parse_logical_and();
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_logical_and(&mut self) -> Expression {
        let mut expr = self.parse_comparison();

        while matches!(self.peek().token_type, TokenType::AndAnd) {
            self.advance(); // consume '&&'
            let right = self.parse_comparison();
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_comparison(&mut self) -> Expression {
        let mut expr = self.parse_additive();

        while matches!(
            self.peek().token_type,
            TokenType::EqualEqual
                | TokenType::NotEqual
                | TokenType::Less
                | TokenType::LessEqual
                | TokenType::Greater
                | TokenType::GreaterEqual
        ) {
            let operator = match self.advance().token_type {
                TokenType::EqualEqual => BinaryOperator::Equal,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_additive();
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_additive(&mut self) -> Expression {
        let mut expr = self.parse_multiplicative();

        while matches!(self.peek().token_type, TokenType::Plus | TokenType::Minus) {
            let operator = match self.advance().token_type {
                TokenType::Plus => BinaryOperator::Add,
                TokenType::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative();
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_multiplicative(&mut self) -> Expression {
        let mut expr = self.parse_primary();

        while matches!(
            self.peek().token_type,
            TokenType::Star | TokenType::Slash | TokenType::Percent
        ) {
            let operator = match self.advance().token_type {
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_primary();
            expr = Expression::BinaryOp {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn parse_primary(&mut self) -> Expression {
        let mut expr = self.parse_primary_base();

        // Handle postfix operations like field access and array indexing
        loop {
            match self.peek().token_type {
                TokenType::Dot => {
                    self.advance(); // consume '.'
                    let field_name = match &self.advance().token_type {
                        TokenType::Identifier(name) => name.clone(),
                        _ => panic!("Expected field name after '.'"),
                    };
                    expr = Expression::FieldAccess {
                        object: Box::new(expr),
                        field: field_name,
                    };
                }
                TokenType::LeftBracket => {
                    self.advance(); // consume '['
                    let index = self.parse_expression();
                    if self.peek().token_type != TokenType::RightBracket {
                        panic!("Expected ']' after array index");
                    }
                    self.advance(); // consume ']'
                    expr = Expression::ArrayAccess {
                        array: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                TokenType::Caret => {
                    self.advance(); // consume '^'
                    expr = Expression::Dereference {
                        operand: Box::new(expr),
                    };
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_primary_base(&mut self) -> Expression {
        match &self.peek().token_type {
            TokenType::Bang => {
                self.advance(); // consume '!'
                let operand = self.parse_primary();
                Expression::UnaryOp {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                }
            }
            TokenType::Ampersand => {
                self.advance(); // consume '&'
                let operand = self.parse_primary();
                Expression::AddressOf {
                    operand: Box::new(operand),
                }
            }
            TokenType::String(value) => {
                let val = value.clone();
                self.advance();
                Expression::StringLiteral(val)
            }
            TokenType::Integer(value) => {
                let val = *value;
                self.advance();
                Expression::IntegerLiteral(val)
            }
            TokenType::Identifier(name) => {
                let val = name.clone();
                self.advance();

                // Only parse as generic type constructor if followed by [Type] { ... }
                // We need to look ahead to distinguish from array access like numbers[0]
                if self.peek().token_type == TokenType::LeftBracket {
                    // Look ahead to see if this looks like a generic type
                    // Save current position in case we need to backtrack
                    let saved_pos = self.current;
                    self.advance(); // consume '['

                    // Try to determine if this is a type or expression
                    // If the first token after [ is a type keyword or uppercase identifier, it's likely a generic
                    let is_generic = match &self.peek().token_type {
                        TokenType::Identifier(n) => {
                            n.chars().next().map_or(false, |c| c.is_uppercase())
                        }
                        _ => false,
                    };

                    if is_generic {
                        // Parse as generic type
                        let mut type_args = Vec::new();

                        while self.peek().token_type != TokenType::RightBracket && !self.is_at_end()
                        {
                            type_args.push(self.parse_type());
                            if self.peek().token_type == TokenType::Comma {
                                self.advance(); // consume ','
                            }
                        }

                        if self.peek().token_type != TokenType::RightBracket {
                            panic!("Expected ']' after generic type arguments");
                        }
                        self.advance(); // consume ']'

                        // Check if this is followed by a struct literal
                        if self.peek().token_type == TokenType::LeftBrace {
                            // Parse generic struct literal like Array[Integer] { ... }
                            self.advance(); // consume '{'
                            let mut fields = Vec::new();

                            while self.peek().token_type != TokenType::RightBrace
                                && !self.is_at_end()
                            {
                                // Skip newlines
                                if self.peek().token_type == TokenType::Newline {
                                    self.advance();
                                    continue;
                                }

                                let field_name = match &self.advance().token_type {
                                    TokenType::Identifier(name) => name.clone(),
                                    _ => panic!("Expected field name in struct literal"),
                                };

                                if self.peek().token_type != TokenType::Colon {
                                    panic!("Expected ':' after field name in struct literal");
                                }
                                self.advance(); // consume ':'

                                let field_value = self.parse_expression();

                                fields.push(StructField {
                                    name: field_name,
                                    value: field_value,
                                });

                                if self.peek().token_type == TokenType::Comma {
                                    self.advance(); // consume ','
                                }
                            }

                            if self.peek().token_type != TokenType::RightBrace {
                                panic!("Expected '}}' after struct fields");
                            }
                            self.advance(); // consume '}'

                            // Pass both the base name and type arguments to the code generator
                            Expression::StructLiteral {
                                type_name: val,
                                type_args: Some(type_args),
                                fields,
                            }
                        } else {
                            // Just a generic type reference, not a constructor
                            Expression::Identifier(val) // TODO: handle generic type expressions properly
                        }
                    } else {
                        // Not a generic type, backtrack and let it be handled as array access
                        self.current = saved_pos;
                        Expression::Identifier(val)
                    }
                } else if self.peek().token_type == TokenType::LeftParen {
                    self.advance(); // consume '('
                    let mut args = Vec::new();

                    while self.peek().token_type != TokenType::RightParen && !self.is_at_end() {
                        args.push(self.parse_expression());
                        if self.peek().token_type == TokenType::Comma {
                            self.advance(); // consume ','
                        }
                    }

                    if self.peek().token_type != TokenType::RightParen {
                        panic!("Expected ')' after function arguments");
                    }
                    self.advance(); // consume ')'

                    Expression::FunctionCall { name: val, args }
                } else if self.peek().token_type == TokenType::Dot {
                    // Handle namespace.function() calls
                    self.advance(); // consume '.'
                    let function_name = match &self.advance().token_type {
                        TokenType::Identifier(name) => name.clone(),
                        _ => panic!("Expected function name after '.'"),
                    };

                    if self.peek().token_type == TokenType::LeftParen {
                        self.advance(); // consume '('
                        let mut args = Vec::new();

                        while self.peek().token_type != TokenType::RightParen && !self.is_at_end() {
                            args.push(self.parse_expression());
                            if self.peek().token_type == TokenType::Comma {
                                self.advance(); // consume ','
                            }
                        }

                        if self.peek().token_type != TokenType::RightParen {
                            panic!("Expected ')' after function arguments");
                        }
                        self.advance(); // consume ')'

                        Expression::NamespacedFunctionCall {
                            namespace: val,
                            function: function_name,
                            args,
                        }
                    } else {
                        // Just a field access, not a function call
                        Expression::FieldAccess {
                            object: Box::new(Expression::Identifier(val)),
                            field: function_name,
                        }
                    }
                } else if self.peek().token_type == TokenType::LeftBrace {
                    // Parse struct literal
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();

                    while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
                        // Skip newlines
                        if self.peek().token_type == TokenType::Newline {
                            self.advance();
                            continue;
                        }

                        let field_name = match &self.advance().token_type {
                            TokenType::Identifier(name) => name.clone(),
                            _ => panic!("Expected field name in struct literal"),
                        };

                        if self.peek().token_type != TokenType::Colon {
                            panic!("Expected ':' after field name in struct literal");
                        }
                        self.advance(); // consume ':'

                        let field_value = self.parse_expression();

                        fields.push(StructField {
                            name: field_name,
                            value: field_value,
                        });

                        if self.peek().token_type == TokenType::Comma {
                            self.advance(); // consume ','
                        }
                    }

                    if self.peek().token_type != TokenType::RightBrace {
                        panic!("Expected '}}' after struct fields");
                    }
                    self.advance(); // consume '}'

                    Expression::StructLiteral {
                        type_name: val,
                        type_args: None,
                        fields,
                    }
                } else {
                    Expression::Identifier(val)
                }
            }
            TokenType::True => {
                self.advance();
                Expression::BoolLiteral(true)
            }
            TokenType::False => {
                self.advance();
                Expression::BoolLiteral(false)
            }
            TokenType::LeftBracket => {
                self.advance(); // consume '['
                let mut elements = Vec::new();

                while self.peek().token_type != TokenType::RightBracket && !self.is_at_end() {
                    elements.push(self.parse_expression());
                    if self.peek().token_type == TokenType::Comma {
                        self.advance(); // consume ','
                    }
                }

                if self.peek().token_type != TokenType::RightBracket {
                    panic!("Expected ']' after array elements");
                }
                self.advance(); // consume ']'

                Expression::ArrayLiteral(elements)
            }
            TokenType::LeftParen => {
                self.advance(); // consume '('
                let expr = self.parse_expression();
                if self.peek().token_type != TokenType::RightParen {
                    panic!("Expected ')' after expression");
                }
                self.advance(); // consume ')'
                expr
            }
            _ => panic!("Expected expression, got {:?}", self.peek().token_type),
        }
    }

    fn parse_if_statement(&mut self) -> Statement {
        self.advance(); // consume 'if'

        if self.peek().token_type != TokenType::LeftParen {
            panic!("Expected '(' after 'if'");
        }
        self.advance(); // consume '('

        let condition = self.parse_expression();

        if self.peek().token_type != TokenType::RightParen {
            panic!("Expected ')' after if condition");
        }
        self.advance(); // consume ')'

        if self.peek().token_type != TokenType::LeftBrace {
            panic!("Expected '{{' after if condition");
        }
        self.advance(); // consume '{'

        let mut then_body = Vec::new();
        while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
            if self.peek().token_type == TokenType::Newline {
                self.advance();
                continue;
            }
            then_body.push(self.parse_statement());
        }

        if self.peek().token_type != TokenType::RightBrace {
            panic!("Expected '}}'");
        }
        self.advance(); // consume '}'

        let mut else_body = None;
        if self.peek().token_type == TokenType::Else {
            self.advance(); // consume 'else'

            if self.peek().token_type == TokenType::If {
                // else if case
                else_body = Some(vec![self.parse_if_statement()]);
            } else if self.peek().token_type == TokenType::LeftBrace {
                self.advance(); // consume '{'

                let mut statements = Vec::new();
                while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
                    if self.peek().token_type == TokenType::Newline {
                        self.advance();
                        continue;
                    }
                    statements.push(self.parse_statement());
                }

                if self.peek().token_type != TokenType::RightBrace {
                    panic!("Expected '}}'");
                }
                self.advance(); // consume '}'

                else_body = Some(statements);
            } else {
                panic!("Expected '{{' or 'if' after 'else'");
            }
        }

        Statement::If {
            condition,
            then_body,
            else_body,
        }
    }

    fn parse_for_loop(&mut self) -> Statement {
        self.advance(); // consume 'for'

        // Check what comes next
        match &self.peek().token_type {
            // Infinite loop: for {
            TokenType::LeftBrace => {
                self.advance(); // consume '{'
                let mut body = Vec::new();
                while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
                    if self.peek().token_type == TokenType::Newline {
                        self.advance();
                        continue;
                    }
                    body.push(self.parse_statement());
                }
                if self.peek().token_type != TokenType::RightBrace {
                    panic!("Expected '}}' after for body");
                }
                self.advance(); // consume '}'

                // Infinite loop is just for (true)
                Statement::ForCondition {
                    condition: Expression::BoolLiteral(true),
                    body,
                }
            }
            // Could be for-in or for with condition
            TokenType::LeftParen => {
                self.advance(); // consume '('

                // Check if it's a condition-only loop or C-style
                // Look ahead to see if we have semicolons
                let mut lookahead = self.current;
                let mut paren_depth = 1;
                let mut has_semicolon = false;

                while lookahead < self.tokens.len() && paren_depth > 0 {
                    match &self.tokens[lookahead].token_type {
                        TokenType::LeftParen => paren_depth += 1,
                        TokenType::RightParen => paren_depth -= 1,
                        TokenType::Semicolon => has_semicolon = true,
                        _ => {}
                    }
                    lookahead += 1;
                }

                if has_semicolon {
                    // C-style for loop: for (init; condition; update)
                    panic!("C-style for loops not yet implemented");
                } else {
                    // Condition-only loop: for (condition)
                    let condition = self.parse_expression();

                    if self.peek().token_type != TokenType::RightParen {
                        panic!("Expected ')' after for condition");
                    }
                    self.advance(); // consume ')'

                    if self.peek().token_type != TokenType::LeftBrace {
                        panic!("Expected '{{' after for condition");
                    }
                    self.advance(); // consume '{'

                    let mut body = Vec::new();
                    while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
                        if self.peek().token_type == TokenType::Newline {
                            self.advance();
                            continue;
                        }
                        body.push(self.parse_statement());
                    }

                    if self.peek().token_type != TokenType::RightBrace {
                        panic!("Expected '}}' after for body");
                    }
                    self.advance(); // consume '}'

                    Statement::ForCondition { condition, body }
                }
            }
            // For-in loop: for item in items
            TokenType::Identifier(variable) => {
                let variable = variable.clone();
                self.advance(); // consume identifier

                if self.peek().token_type != TokenType::In {
                    panic!("Expected 'in' after for loop variable");
                }
                self.advance(); // consume 'in'

                let iterable = self.parse_for_in_iterable();

                if self.peek().token_type != TokenType::LeftBrace {
                    panic!("Expected '{{' after for-in expression");
                }
                self.advance(); // consume '{'

                let mut body = Vec::new();
                while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
                    if self.peek().token_type == TokenType::Newline {
                        self.advance();
                        continue;
                    }
                    body.push(self.parse_statement());
                }

                if self.peek().token_type != TokenType::RightBrace {
                    panic!("Expected '}}' after for-in body");
                }
                self.advance(); // consume '}'

                Statement::ForIn {
                    variable,
                    iterable,
                    body,
                }
            }
            _ => panic!("Unexpected token after 'for'"),
        }
    }

    fn parse_for_in_iterable(&mut self) -> Expression {
        match &self.peek().token_type {
            TokenType::Identifier(name) => {
                let val = name.clone();
                self.advance();
                Expression::Identifier(val)
            }
            TokenType::LeftBracket => {
                self.advance(); // consume '['
                let mut elements = Vec::new();

                while self.peek().token_type != TokenType::RightBracket && !self.is_at_end() {
                    elements.push(self.parse_expression());
                    if self.peek().token_type == TokenType::Comma {
                        self.advance(); // consume ','
                    }
                }

                if self.peek().token_type != TokenType::RightBracket {
                    panic!("Expected ']' after array elements");
                }
                self.advance(); // consume ']'

                Expression::ArrayLiteral(elements)
            }
            _ => {
                // For other expressions, use the full expression parser
                // but this might still have the same issue
                self.parse_logical_or()
            }
        }
    }

    fn parse_function(&mut self, exported: bool) -> Statement {
        self.advance(); // consume 'fun'

        let name = match &self.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => panic!("Expected function name"),
        };

        if self.peek().token_type != TokenType::LeftParen {
            panic!("Expected '(' after function name");
        }
        self.advance(); // consume '('

        let mut params = Vec::new();
        let mut param_types = Vec::new();

        while self.peek().token_type != TokenType::RightParen && !self.is_at_end() {
            let param_name = match &self.advance().token_type {
                TokenType::Identifier(name) => name.clone(),
                _ => panic!("Expected parameter name"),
            };

            if self.peek().token_type != TokenType::Colon {
                panic!("Expected ':' after parameter name");
            }
            self.advance(); // consume ':'

            let param_type = self.parse_type();
            param_types.push(param_type.clone());

            params.push(Parameter {
                name: param_name,
                param_type,
            });

            if self.peek().token_type == TokenType::Comma {
                self.advance(); // consume ','
            }
        }

        if self.peek().token_type != TokenType::RightParen {
            panic!("Expected ')' after parameters");
        }
        self.advance(); // consume ')'

        let mut return_type = None;
        if self.peek().token_type == TokenType::Colon {
            self.advance(); // consume ':'
            return_type = Some(self.parse_type());
        }

        // Register function in symbol table
        if let Err(e) =
            self.symbol_table
                .declare_function(name.clone(), param_types, return_type.clone(), None)
        {
            panic!("Error declaring function '{}': {}", name, e);
        }

        // Enter function scope
        self.symbol_table
            .enter_scope(ScopeKind::Function { name: name.clone() });

        // Register parameters in the function scope
        for param in &params {
            if let Err(e) = self.symbol_table.declare_parameter(
                param.name.clone(),
                param.param_type.clone(),
                None,
            ) {
                panic!("Error declaring parameter '{}': {}", param.name, e);
            }
        }

        if self.peek().token_type != TokenType::LeftBrace {
            panic!("Expected '{{' after function signature");
        }
        self.advance(); // consume '{'

        let mut body = Vec::new();
        while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
            if self.peek().token_type == TokenType::Newline {
                self.advance();
                continue;
            }
            body.push(self.parse_statement());
        }

        if self.peek().token_type != TokenType::RightBrace {
            panic!("Expected '}}'");
        }
        self.advance(); // consume '}'

        // Exit function scope
        if let Err(e) = self.symbol_table.exit_scope() {
            panic!("Error exiting function scope: {}", e);
        }

        Statement::Function {
            name,
            params,
            return_type,
            body,
            exported,
        }
    }

    fn parse_return(&mut self) -> Statement {
        self.advance(); // consume 'return'

        let value = if self.peek().token_type == TokenType::Newline
            || self.peek().token_type == TokenType::RightBrace
            || self.is_at_end()
        {
            None
        } else {
            Some(self.parse_expression())
        };

        Statement::Return(value)
    }

    fn parse_import(&mut self) -> Statement {
        self.advance(); // consume 'import'

        // Check for different import patterns:
        // import module from "path"       - namespace import
        // import { item1, item2 } from "path"  - selective import

        if self.peek().token_type == TokenType::LeftBrace {
            // import { item1, item2 } from "path"
            self.advance(); // consume '{'
            let mut items = Vec::new();

            while self.peek().token_type != TokenType::RightBrace && !self.is_at_end() {
                if let TokenType::Identifier(name) = &self.advance().token_type {
                    items.push(name.clone());
                    if self.peek().token_type == TokenType::Comma {
                        self.advance(); // consume ','
                    }
                } else {
                    panic!("Expected identifier in import list");
                }
            }

            if self.peek().token_type != TokenType::RightBrace {
                panic!("Expected '}}' after import list");
            }
            self.advance(); // consume '}'

            if self.peek().token_type != TokenType::From {
                panic!("Expected 'from' after import list");
            }
            self.advance(); // consume 'from'

            let module_path = match &self.advance().token_type {
                TokenType::String(path) => path.clone(),
                _ => panic!("Expected string after 'from'"),
            };

            Statement::Import {
                module_name: None, // selective import
                module_path,
                items: Some(items),
            }
        } else {
            // import module from "path"
            let module_name = match &self.advance().token_type {
                TokenType::Identifier(name) => name.clone(),
                _ => panic!("Expected module name after 'import'"),
            };

            if self.peek().token_type != TokenType::From {
                panic!("Expected 'from' after module name");
            }
            self.advance(); // consume 'from'

            let module_path = match &self.advance().token_type {
                TokenType::String(path) => path.clone(),
                _ => panic!("Expected string after 'from'"),
            };

            Statement::Import {
                module_name: Some(module_name),
                module_path,
                items: None, // namespace import
            }
        }
    }

    fn parse_function_with_export_flag(&mut self, exported: bool) -> Statement {
        self.parse_function(exported)
    }

    fn parse_export(&mut self) -> Statement {
        self.advance(); // consume 'export'

        // Check if next token is 'fun' (export function)
        match &self.peek().token_type {
            TokenType::Fun => {
                // Parse the function with exported=true
                self.parse_function(true)
            }
            TokenType::Identifier(name) => {
                // export item (existing functionality)
                let item = name.clone();
                self.advance(); // consume the identifier
                Statement::Export { item }
            }
            _ => panic!("Expected 'fun' or identifier after 'export'"),
        }
    }

    fn parse_native_block(&mut self) -> Statement {
        self.advance(); // consume 'native'
        
        // Expect a string literal for the language (e.g., "C")
        let language = match &self.advance().token_type {
            TokenType::String(lang) => lang.clone(),
            _ => panic!("Expected string literal after 'native' (e.g., 'native \"C\"')"),
        };
        
        // Expect '{'
        match &self.advance().token_type {
            TokenType::LeftBrace => {},
            _ => panic!("Expected '{{' after native language"),
        }
        
        let mut functions = Vec::new();
        
        // Parse function declarations until '}'
        while !matches!(self.peek().token_type, TokenType::RightBrace) {
            if self.is_at_end() {
                panic!("Unclosed native block");
            }
            
            // Skip newlines
            if matches!(self.peek().token_type, TokenType::Newline) {
                self.advance();
                continue;
            }
            
            let exported = if matches!(self.peek().token_type, TokenType::Export) {
                self.advance(); // consume 'export'
                true
            } else {
                false
            };
            
            // Expect 'fun' (after optional export)
            match &self.peek().token_type {
                TokenType::Fun => {
                    self.advance(); // consume 'fun'
                },
                _ => panic!("Expected 'fun' in native block, found {:?}", self.peek().token_type),
            }
            
            // Parse function signature (name, params, return type)
            let name = match &self.advance().token_type {
                TokenType::Identifier(n) => n.clone(),
                _ => panic!("Expected function name"),
            };
            
            // Expect '('
            match &self.advance().token_type {
                TokenType::LeftParen => {},
                _ => panic!("Expected '(' after function name"),
            }
            
            let mut params = Vec::new();
            
            // Parse parameters
            while !matches!(self.peek().token_type, TokenType::RightParen) {
                if !params.is_empty() {
                    match &self.advance().token_type {
                        TokenType::Comma => {},
                        _ => panic!("Expected ',' between parameters"),
                    }
                }
                
                let param_name = match &self.advance().token_type {
                    TokenType::Identifier(n) => n.clone(),
                    _ => panic!("Expected parameter name"),
                };
                
                // Expect ':'
                match &self.advance().token_type {
                    TokenType::Colon => {},
                    _ => panic!("Expected ':' after parameter name"),
                }
                
                let param_type = self.parse_type();
                params.push(Parameter {
                    name: param_name,
                    param_type,
                });
            }
            
            self.advance(); // consume ')'
            
            // Parse optional return type
            let return_type = if matches!(self.peek().token_type, TokenType::Colon) {
                self.advance(); // consume ':'
                Some(self.parse_type())
            } else {
                None
            };
            
            functions.push(NativeFunction {
                name,
                params,
                return_type,
                exported,
            });
        }
        
        // Expect '}'
        match &self.advance().token_type {
            TokenType::RightBrace => {},
            _ => panic!("Expected '}}' to close native block"),
        }
        
        Statement::NativeBlock {
            language,
            functions,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_type_from_string(input: &str) -> Type {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_type()
    }

    fn parse_statement_from_string(input: &str) -> Statement {
        let mut lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse_statement()
    }

    #[test]
    fn test_basic_type_parsing() {
        assert!(matches!(parse_type_from_string("String"), Type::String));
        assert!(matches!(parse_type_from_string("Integer"), Type::Integer));
        assert!(matches!(parse_type_from_string("Bool"), Type::Bool));
    }

    #[test]
    fn test_custom_type_parsing() {
        match parse_type_from_string("MyType") {
            Type::Custom(name) => assert_eq!(name, "MyType"),
            _ => panic!("Expected Custom type"),
        }
    }

    #[test]
    fn test_generic_type_parsing() {
        // Test Array[Integer]
        match parse_type_from_string("Array[Integer]") {
            Type::Generic { name, type_params } => {
                assert_eq!(name, "Array");
                assert_eq!(type_params.len(), 1);
                assert!(matches!(type_params[0], Type::Integer));
            }
            _ => panic!("Expected Generic type"),
        }

        // Test Map[String, Integer]
        match parse_type_from_string("Map[String, Integer]") {
            Type::Generic { name, type_params } => {
                assert_eq!(name, "Map");
                assert_eq!(type_params.len(), 2);
                assert!(matches!(type_params[0], Type::String));
                assert!(matches!(type_params[1], Type::Integer));
            }
            _ => panic!("Expected Generic type"),
        }
    }

    #[test]
    fn test_nested_generic_type_parsing() {
        // Test Array[Array[Integer]]
        match parse_type_from_string("Array[Array[Integer]]") {
            Type::Generic { name, type_params } => {
                assert_eq!(name, "Array");
                assert_eq!(type_params.len(), 1);
                match &type_params[0] {
                    Type::Generic {
                        name: inner_name,
                        type_params: inner_params,
                    } => {
                        assert_eq!(inner_name, "Array");
                        assert_eq!(inner_params.len(), 1);
                        assert!(matches!(inner_params[0], Type::Integer));
                    }
                    _ => panic!("Expected nested Generic type"),
                }
            }
            _ => panic!("Expected Generic type"),
        }
    }

    #[test]
    fn test_pointer_type_parsing() {
        match parse_type_from_string("^Integer") {
            Type::Pointer(inner) => assert!(matches!(inner.as_ref(), Type::Integer)),
            _ => panic!("Expected Pointer type"),
        }

        // Test ^Array[String]
        match parse_type_from_string("^Array[String]") {
            Type::Pointer(inner) => match inner.as_ref() {
                Type::Generic { name, type_params } => {
                    assert_eq!(name, "Array");
                    assert_eq!(type_params.len(), 1);
                    assert!(matches!(type_params[0], Type::String));
                }
                _ => panic!("Expected Generic type inside pointer"),
            },
            _ => panic!("Expected Pointer type"),
        }
    }

    #[test]
    fn test_generic_type_definition_parsing() {
        let input = "type Array[T] = { data: ^T, length: Integer }";
        match parse_statement_from_string(input) {
            Statement::TypeDef {
                name,
                type_params,
                fields,
            } => {
                assert_eq!(name, "Array");
                assert_eq!(type_params.len(), 1);
                assert_eq!(type_params[0], "T");
                assert_eq!(fields.len(), 2);

                // Check first field: data: ^T
                assert_eq!(fields[0].name, "data");
                match &fields[0].field_type {
                    Type::Pointer(inner) => match inner.as_ref() {
                        Type::Custom(param_name) => assert_eq!(param_name, "T"),
                        _ => panic!("Expected Custom type (type parameter) in pointer"),
                    },
                    _ => panic!("Expected Pointer type for data field"),
                }

                // Check second field: length: Integer
                assert_eq!(fields[1].name, "length");
                assert!(matches!(fields[1].field_type, Type::Integer));
            }
            _ => panic!("Expected TypeDef statement"),
        }
    }

    #[test]
    fn test_multi_param_generic_definition() {
        let input = "type Map[K, V] = { keys: Array[K], values: Array[V] }";
        match parse_statement_from_string(input) {
            Statement::TypeDef {
                name,
                type_params,
                fields,
            } => {
                assert_eq!(name, "Map");
                assert_eq!(type_params.len(), 2);
                assert_eq!(type_params[0], "K");
                assert_eq!(type_params[1], "V");
                assert_eq!(fields.len(), 2);

                // Check keys field: Array[K]
                match &fields[0].field_type {
                    Type::Generic { name, type_params } => {
                        assert_eq!(name, "Array");
                        assert_eq!(type_params.len(), 1);
                        match &type_params[0] {
                            Type::Custom(param) => assert_eq!(param, "K"),
                            _ => panic!("Expected type parameter K"),
                        }
                    }
                    _ => panic!("Expected Generic type for keys field"),
                }

                // Check values field: Array[V]
                match &fields[1].field_type {
                    Type::Generic { name, type_params } => {
                        assert_eq!(name, "Array");
                        assert_eq!(type_params.len(), 1);
                        match &type_params[0] {
                            Type::Custom(param) => assert_eq!(param, "V"),
                            _ => panic!("Expected type parameter V"),
                        }
                    }
                    _ => panic!("Expected Generic type for values field"),
                }
            }
            _ => panic!("Expected TypeDef statement"),
        }
    }
}
