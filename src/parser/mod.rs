pub mod ast;

use crate::lexer::token::{Token, TokenType};
use ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Eof)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for t in types {
            if self.check(t) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<&Token, String> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            let tok = self.peek();
            Err(format!("{} at line {}, column {}", message, tok.line, tok.column))
        }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(Program { statements })
    }

    fn declaration(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[TokenType::Func]) {
            self.function_declaration()
        } else if self.match_token(&[TokenType::Class]) {
            self.class_declaration()
        } else {
            self.statement()
        }
    }

    fn function_declaration(&mut self) -> Result<Stmt, String> {
        let name = if let TokenType::Identifier(id) = &self.peek().token_type {
            let n = id.clone();
            self.advance();
            n
        } else {
            return Err(format!("Expected function name at line {}", self.peek().line));
        };

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        
        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if let TokenType::Identifier(id) = &self.peek().token_type {
                    params.push(id.clone());
                    self.advance();
                } else {
                    return Err(format!("Expected parameter name at line {}", self.peek().line));
                }

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        // Optional return type annotation
        let return_type = if self.match_token(&[TokenType::Colon]) {
            if let TokenType::Identifier(type_name) = &self.peek().token_type {
                let t = Some(type_name.clone());
                self.advance();
                t
            } else {
                return Err(format!("Expected type name after ':' at line {}", self.peek().line));
            }
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;
        
        let mut body = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after function body")?;

        Ok(Stmt::FuncDecl {
            name,
            params,
            return_type,
            body,
        })
    }

    fn class_declaration(&mut self) -> Result<Stmt, String> {
        let name = if let TokenType::Identifier(id) = &self.peek().token_type {
            let n = id.clone();
            self.advance();
            n
        } else {
            return Err(format!("Expected class name at line {}", self.peek().line));
        };

        // Check for inheritance
        let extends = if self.match_token(&[TokenType::Extends]) {
            if let TokenType::Identifier(parent_name) = &self.peek().token_type {
                let p = Some(parent_name.clone());
                self.advance();
                p
            } else {
                return Err(format!("Expected parent class name at line {}", self.peek().line));
            }
        } else {
            None
        };

        self.consume(TokenType::LeftBrace, "Expected '{' before class body")?;

        let mut methods = Vec::new();
        let mut properties = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            if self.match_token(&[TokenType::Func]) {
                // Parse method
                let method_name = if let TokenType::Identifier(id) = &self.peek().token_type {
                    let n = id.clone();
                    self.advance();
                    n
                } else {
                    return Err(format!("Expected method name at line {}", self.peek().line));
                };

                self.consume(TokenType::LeftParen, "Expected '(' after method name")?;
                
                let mut params = Vec::new();
                if !self.check(&TokenType::RightParen) {
                    loop {
                        if let TokenType::Identifier(id) = &self.peek().token_type {
                            params.push(id.clone());
                            self.advance();
                        } else {
                            return Err(format!("Expected parameter name at line {}", self.peek().line));
                        }

                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }

                self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

                // Optional return type
                let return_type = if self.match_token(&[TokenType::Colon]) {
                    if let TokenType::Identifier(type_name) = &self.peek().token_type {
                        let t = Some(type_name.clone());
                        self.advance();
                        t
                    } else {
                        None
                    }
                } else {
                    None
                };

                self.consume(TokenType::LeftBrace, "Expected '{' before method body")?;
                
                let mut body = Vec::new();
                while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                    body.push(self.declaration()?);
                }

                self.consume(TokenType::RightBrace, "Expected '}' after method body")?;

                methods.push((method_name, params, return_type, body));
            } else {
                // Parse property
                if let TokenType::Identifier(prop_name) = &self.peek().token_type {
                    let p = prop_name.clone();
                    self.advance();
                    
                    if self.match_token(&[TokenType::Assign]) {
                        let expr = self.expression()?;
                        properties.push((p, expr));
                    } else {
                        properties.push((p, Expr::Literal(Literal::Null)));
                    }

                    if self.match_token(&[TokenType::Semicolon]) {
                        // Optional semicolon
                    }
                } else {
                    return Err(format!("Expected property name at line {}", self.peek().line));
                }
            }
        }

        self.consume(TokenType::RightBrace, "Expected '}' after class body")?;

        Ok(Stmt::ClassDecl {
            name,
            extends,
            methods,
            properties,
        })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block_statement()?))
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = if !self.check(&TokenType::RightBrace) {
            Some(self.expression()?)
        } else {
            None
        };
        Ok(Stmt::Return(value))
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expected ')' after condition")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'for'")?;

        // Check if this is a foreach loop (for variable in iterable)
        if let TokenType::Identifier(var_name) = &self.peek().token_type {
            let temp_pos = self.current;
            let var_name = var_name.clone();
            self.advance();
            
            if self.match_token(&[TokenType::In]) {
                // This is a foreach loop
                let iterable = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after foreach")?;
                let body = Box::new(self.statement()?);
                return Ok(Stmt::ForEach {
                    variable: var_name,
                    iterable,
                    body,
                });
            } else {
                // Reset and parse as regular for loop
                self.current = temp_pos;
            }
        }

        // Parse as traditional for loop: for (init; condition; increment)
        let init = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(Box::new(self.statement()?))
        };
        self.consume(TokenType::Semicolon, "Expected ';' after for loop initializer")?;

        let condition = if self.check(&TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "Expected ';' after for loop condition")?;

        let increment = if self.check(&TokenType::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::RightParen, "Expected ')' after for clauses")?;

        let body = Box::new(self.statement()?);

        Ok(Stmt::For {
            init,
            condition,
            increment,
            body,
        })
    }

    fn block_statement(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, String> {
        let expr = self.expression()?;
        
        // Check if this is a variable declaration (assignment)
        if let Expr::Assign { name, value } = expr {
            Ok(Stmt::VarDecl {
                name,
                value: *value,
            })
        } else {
            Ok(Stmt::Expr(expr))
        }
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Assign]) {
            let value = Box::new(self.assignment()?);
            match expr {
                Expr::Variable(name) => {
                    return Ok(Expr::Assign { name, value });
                }
                Expr::PropertyAccess { object, property } => {
                    // Property assignment: obj.prop = value
                    return Ok(Expr::PropertyAssign {
                        object,
                        property,
                        value,
                    });
                }
                _ => {
                    return Err("Invalid assignment target".to_string());
                }
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let operator = BinaryOp::Or;
            let right = Box::new(self.and()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let operator = BinaryOp::And;
            let right = Box::new(self.equality()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::EqualEqual, TokenType::NotEqual]) {
            let operator = match &self.previous().token_type {
                TokenType::EqualEqual => BinaryOp::Equal,
                TokenType::NotEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = Box::new(self.comparison()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = match &self.previous().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = Box::new(self.term()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Plus, TokenType::Minus]) {
            let operator = match &self.previous().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = Box::new(self.factor()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Star, TokenType::Slash]) {
            let operator = match &self.previous().token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                _ => unreachable!(),
            };
            let right = Box::new(self.unary()?);
            expr = Expr::BinaryOp {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = match &self.previous().token_type {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Negate,
                _ => unreachable!(),
            };
            let right = Box::new(self.unary()?);
            return Ok(Expr::UnaryOp { operator, right });
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenType::Dot]) {
                if let TokenType::Identifier(name) = &self.peek().token_type {
                    let member_name = name.clone();
                    self.advance();
                    
                    // Check if it's a method call or property access
                    if self.match_token(&[TokenType::LeftParen]) {
                        // Method call
                        let mut args = Vec::new();
                        
                        if !self.check(&TokenType::RightParen) {
                            loop {
                                args.push(self.expression()?);
                                if !self.match_token(&[TokenType::Comma]) {
                                    break;
                                }
                            }
                        }
                        
                        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            method: member_name,
                            args,
                        };
                    } else {
                        // Property access
                        expr = Expr::PropertyAccess {
                            object: Box::new(expr),
                            property: member_name,
                        };
                    }
                } else {
                    return Err(format!("Expected property or method name after '.' at line {}", self.peek().line));
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut args = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                args.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;

        if let Expr::Variable(name) = callee {
            Ok(Expr::FunctionCall { name, args })
        } else {
            Err("Invalid function call".to_string())
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match &self.peek().token_type {
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Boolean(false)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            TokenType::Number(n) => {
                let num = n.parse::<f64>().map_err(|_| "Invalid number")?;
                self.advance();
                Ok(Expr::Literal(Literal::Number(num)))
            }
            TokenType::String(s) => {
                let str = s.clone();
                self.advance();
                Ok(Expr::Literal(Literal::String(str)))
            }
            TokenType::New => {
                self.advance();
                if let TokenType::Identifier(class_name) = &self.peek().token_type {
                    let name = class_name.clone();
                    self.advance();
                    
                    self.consume(TokenType::LeftParen, "Expected '(' after class name")?;
                    
                    let mut args = Vec::new();
                    if !self.check(&TokenType::RightParen) {
                        loop {
                            args.push(self.expression()?);
                            if !self.match_token(&[TokenType::Comma]) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
                    
                    Ok(Expr::New {
                        class_name: name,
                        args,
                    })
                } else {
                    Err(format!("Expected class name after 'new' at line {}", self.peek().line))
                }
            }
            TokenType::Identifier(id) => {
                let name = id.clone();
                self.advance();
                Ok(Expr::Variable(name))
            }
            TokenType::LeftParen => {
                self.advance();
                
                // Check for lambda: (params) => body
                if let TokenType::Identifier(_) = &self.peek().token_type {
                    let start_pos = self.current;
                    let mut params = Vec::new();
                    
                    // Try to parse as lambda parameters
                    loop {
                        if let TokenType::Identifier(id) = &self.peek().token_type {
                            params.push(id.clone());
                            self.advance();
                        } else {
                            break;
                        }
                        
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                    
                    if self.check(&TokenType::RightParen) {
                        self.advance();
                        if self.match_token(&[TokenType::Arrow]) {
                            // It's a lambda!
                            let body = Box::new(self.expression()?);
                            return Ok(Expr::Lambda { params, body });
                        }
                    }
                    
                    // Not a lambda, backtrack
                    self.current = start_pos - 1; // -1 because we already consumed LeftParen
                }
                
                // Regular grouped expression
                let expr = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            TokenType::LeftBracket => {
                self.advance();
                let mut elements = Vec::new();
                
                if !self.check(&TokenType::RightBracket) {
                    loop {
                        elements.push(self.expression()?);
                        if !self.match_token(&[TokenType::Comma]) {
                            break;
                        }
                    }
                }
                
                self.consume(TokenType::RightBracket, "Expected ']' after array elements")?;
                Ok(Expr::Array(elements))
            }
            TokenType::Match => {
                self.advance();
                self.consume(TokenType::LeftParen, "Expected '(' after 'match'")?;
                let expr = Box::new(self.expression()?);
                self.consume(TokenType::RightParen, "Expected ')' after match expression")?;
                self.consume(TokenType::LeftBrace, "Expected '{' before match cases")?;
                
                let mut cases = Vec::new();
                
                while self.match_token(&[TokenType::Case]) {
                    let pattern = self.match_pattern()?;
                    self.consume(TokenType::Arrow, "Expected '=>' after case pattern")?;
                    let body = self.expression()?;
                    cases.push(MatchCase { pattern, body });
                }
                
                self.consume(TokenType::RightBrace, "Expected '}' after match cases")?;
                Ok(Expr::Match { expr, cases })
            }
            _ => {
                let tok = self.peek();
                Err(format!("Unexpected token {:?} at line {}, column {}", tok.token_type, tok.line, tok.column))
            }
        }
    }

    fn match_pattern(&mut self) -> Result<Pattern, String> {
        match &self.peek().token_type {
            TokenType::String(s) => {
                let str = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(str)))
            }
            TokenType::Number(n) => {
                let num = n.parse::<f64>().map_err(|_| "Invalid number")?;
                self.advance();
                Ok(Pattern::Literal(Literal::Number(num)))
            }
            TokenType::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(true)))
            }
            TokenType::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(false)))
            }
            TokenType::Identifier(id) if id == "_" => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenType::Identifier(id) => {
                let name = id.clone();
                self.advance();
                Ok(Pattern::Identifier(name))
            }
            _ => Err(format!("Invalid pattern at line {}", self.peek().line)),
        }
    }
}
