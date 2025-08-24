use crate::lexer::{Token, TokenType};
use crate::ast::{Statement, Expression, Type, Field, Program, Parameter, BinaryOperator, UnaryOperator, StructField};

pub type ParseResult<T> = std::result::Result<T, String>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
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
            _ => {
                // Could be assignment or expression
                // Look ahead to see if it's an assignment
                if matches!(self.peek().token_type, TokenType::Identifier(_)) {
                    let next_idx = self.current + 1;
                    if next_idx < self.tokens.len() && self.tokens[next_idx].token_type == TokenType::Equal {
                        // It's an assignment
                        let name = match &self.advance().token_type {
                            TokenType::Identifier(n) => n.clone(),
                            _ => unreachable!(),
                        };
                        self.advance(); // consume '='
                        let value = self.parse_expression();
                        Statement::Assignment { variable: name, value }
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

        Statement::ValDecl {
            name,
            type_annotation,
            value,
        }
    }

    fn parse_type_def(&mut self) -> Statement {
        self.advance(); // consume 'def'
        
        let name = match &self.advance().token_type {
            TokenType::Identifier(name) => name.clone(),
            _ => panic!("Expected identifier after 'def'"),
        };

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

        Statement::TypeDef { name, fields }
    }

    fn parse_type(&mut self) -> Type {
        // Check for pointer type prefix (^)
        if self.peek().token_type == TokenType::Caret {
            self.advance(); // consume '^'
            let pointee_type = self.parse_type();
            return Type::Pointer(Box::new(pointee_type));
        }
        
        match &self.advance().token_type {
            TokenType::Identifier(name) => match name.as_str() {
                "String" => Type::String,
                "Integer" => Type::Integer,
                "Bool" => Type::Bool,
                "Array" => {
                    // Parse Array<Type> syntax
                    if self.peek().token_type != TokenType::LeftBracket {
                        panic!("Expected '[' after 'Array'");
                    }
                    self.advance(); // consume '['
                    let element_type = self.parse_type();
                    if self.peek().token_type != TokenType::RightBracket {
                        panic!("Expected ']' after array element type");
                    }
                    self.advance(); // consume ']'
                    Type::Array(Box::new(element_type))
                }
                _ => Type::Custom(name.clone()),
            },
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

        while matches!(self.peek().token_type, 
            TokenType::EqualEqual | TokenType::NotEqual | 
            TokenType::Less | TokenType::LessEqual |
            TokenType::Greater | TokenType::GreaterEqual) {
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

        while matches!(self.peek().token_type, TokenType::Star | TokenType::Slash | TokenType::Percent) {
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
                
                // Check if this is a function call
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
                    
                    Expression::FunctionCall {
                        name: val,
                        args,
                    }
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
                    
                    Statement::ForCondition {
                        condition,
                        body,
                    }
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
        
        let value = if self.peek().token_type == TokenType::Newline || 
                       self.peek().token_type == TokenType::RightBrace ||
                       self.is_at_end() {
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