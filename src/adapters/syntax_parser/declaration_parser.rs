//! # Declaration Parser (Adapter)
//!
//! This module implements top-level declaration parsing for the Idris 2
//! compiler. It handles all declaration forms including:
//!
//! - `module Name`
//! - `import Name`
//! - `data Name params = Con1 | Con2` (simple ADTs)
//! - `data Name : Type where ...` (GADT-style)
//! - `interface Name params where ...`
//! - `implementation InterfaceName TargetType where ...`
//! - `[Name] InterfaceName TargetType where ...` (named implementations)
//! - `record Name where ...`
//! - `mutual ...`
//! - `parameters (...) ...`
//! - Function signatures: `name : Type`
//! - Function definitions: `name args = body`
//! - Visibility modifiers: `public export`, `export`, `total`, `partial`
//!
//! # Design Pattern: Composition via Extension Impl
//! This extends `Parser` analogously to `expression_parser` and
//! `type_parser`. The three sub-parser files together replace the
//! monolithic god class while sharing `Parser`'s cursor and arena state.

use crate::domain::{Term, term::Constructor};
use crate::adapters::syntax_parser::scanner::Token;
use crate::common::errors::{CompilerError, ParseError};

use super::parser::Parser;

impl<'a, 'arena> Parser<'a, 'arena> {
    /// Dispatches on the leading token to parse the appropriate declaration form.
    ///
    /// # Business Logic
    /// When leading with an Identifier, we must disambiguate between a
    /// type signature (`name : Type`) and a function definition (`name args = body`).
    /// We also handle visibility modifiers (`public`, `export`, `total`, etc.)
    /// and named implementations (`[Name] Interface Type where`).
    pub fn parse_declaration(&mut self) -> Result<Term<'a>, CompilerError> {
        self.skip_newlines();
        match self.peek() {
            Token::Module => self.parse_module(),
            Token::Import => self.parse_import(),
            Token::Data => self.parse_data(),
            Token::Interface => self.parse_interface(),
            Token::Implementation => self.parse_implementation(),
            Token::Record => self.parse_record(),
            Token::Mutual => self.parse_mutual(),
            Token::LBracket => {
                // Named implementation: [Name] Interface Type where
                self.parse_named_impl_or_skip()
            }
            Token::Identifier(_) => {
                // Check for visibility/totality modifiers first
                let peek_name = match self.peek() {
                    Token::Identifier(n) => Some(n.clone()),
                    _ => None,
                };
                if let Some(name) = peek_name {
                    match name.as_str() {
                        "public" | "export" | "total" | "partial" | "covering" => {
                            return self.parse_with_modifier();
                        }
                        "parameters" => {
                            return self.parse_parameters_block();
                        }
                        _ => {}
                    }
                }

                // Peek ahead to see if it's a signature or a definition.
                let mut idx = 1;
                while let Some(t) = self.cursor.peek_at(idx) {
                    if t.node == Token::Newline {
                        idx += 1;
                        continue;
                    }
                    if t.node == Token::Colon {
                        return self.parse_signature_decl();
                    }
                    break;
                }
                self.parse_definition_decl()
            }
            Token::Integer(_) => {
                // Numeric literal in declaration position (e.g., `5 = 5` in Proofs.idr
                // is a dependent type expression). Try as a signature/expression.
                self.parse_definition_decl()
            }
            _ => {
                // Permissive: skip unknown tokens at declaration level
                // to allow parsing to continue past unsupported syntax.
                let token = self.advance();
                // If we hit a known sub-expression start, try to recover
                match token.node {
                    Token::LParen => {
                        // Likely an operator definition: (++) : Type
                        self.skip_balanced_parens_decl();
                        self.skip_newlines();
                        if self.peek() == &Token::Colon {
                            return self.parse_signature_after_name("_op_".to_string());
                        }
                        // Try as next declaration
                        if self.peek() != &Token::EOF {
                            return self.parse_declaration();
                        }
                        Err(CompilerError::Parse(ParseError {
                            span: token.span,
                            token: token.node,
                            expected: Some("Declaration".to_string()),
                            message: "Expected top-level declaration".to_string(),
                        }))
                    }
                    _ => {
                        // Skip to next newline and try again
                        while self.peek() != &Token::Newline && self.peek() != &Token::EOF {
                            self.advance();
                        }
                        self.skip_newlines();
                        if self.peek() != &Token::EOF {
                            self.parse_declaration()
                        } else {
                            Ok(Term::Module("_skipped".to_string()))
                        }
                    }
                }
            }
        }
    }

    /// Helper: skip balanced parens at declaration level
    fn skip_balanced_parens_decl(&mut self) {
        let mut depth = 1;
        while self.peek() != &Token::EOF && depth > 0 {
            match self.peek() {
                Token::LParen => { depth += 1; self.advance(); }
                Token::RParen => { depth -= 1; self.advance(); }
                _ => { self.advance(); }
            }
        }
    }

    /// Parse a signature when name has already been determined
    fn parse_signature_after_name(&mut self, _name: String) -> Result<Term<'a>, CompilerError> {
        self.consume(Token::Colon, "Expected : in signature")?;
        let sig = self.parse_pi()?;
        Ok(sig.clone())
    }

    /// Parses `module Name`.
    fn parse_module(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // module
        let name_token = self.advance();
        match name_token.node {
            Token::Identifier(n) => Ok(Term::Module(n)),
            _ => Err(CompilerError::Parse(ParseError {
                span: name_token.span,
                token: name_token.node,
                expected: Some("Module Name".to_string()),
                message: "Expected module name".to_string(),
            })),
        }
    }

    /// Parses `import Name`.
    fn parse_import(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // import
        let name_token = self.advance();
        match name_token.node {
            Token::Identifier(n) => Ok(Term::Import(n)),
            _ => Err(CompilerError::Parse(ParseError {
                span: name_token.span,
                token: name_token.node,
                expected: Some("Import Name".to_string()),
                message: "Expected import name".to_string(),
            })),
        }
    }

    /// Parses data type declarations in both forms:
    /// - Simple ADT: `data Name params = Con1 | Con2`
    /// - GADT-style: `data Name : IndexType -> Type where Con1 : ...; Con2 : ...`
    fn parse_data(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // data
        let name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected data type name"),
        };
        let mut params = Vec::new();
        self.skip_newlines();

        // Check for GADT-style: `data Name : ... where`
        if self.peek() == &Token::Colon {
            self.advance(); // skip :
            // Skip the kind signature until `where`
            while self.peek() != &Token::Where && self.peek() != &Token::EOF {
                self.advance();
            }
            if self.peek() == &Token::Where {
                self.advance(); // skip 'where'
            }
            // Parse GADT constructors: `Con : Type` separated by newlines
            let mut constructors = Vec::new();
            self.skip_newlines();
            while self.peek() != &Token::EOF && !self.is_non_constructor_decl() {
                self.skip_newlines();
                if self.peek() == &Token::EOF || self.is_non_constructor_decl() { break; }
                // Constructor name (skip LParen for operator constructors like `(::)`)
                let c_name = if self.peek() == &Token::LParen {
                    self.advance(); // (
                    let mut op_name = String::new();
                    while self.peek() != &Token::RParen && self.peek() != &Token::EOF {
                        let t = self.advance();
                        op_name.push_str(&format!("{:?}", t.node));
                    }
                    if self.peek() == &Token::RParen { self.advance(); }
                    op_name
                } else {
                    match self.advance().node {
                        Token::Identifier(n) => n,
                        _ => break,
                    }
                };
                // Skip the constructor's type signature (everything until next newline at indent 0)
                if self.peek() == &Token::Colon {
                    self.advance(); // skip :
                    // Skip type until next newline-at-start or EOF
                    while self.peek() != &Token::EOF {
                        if self.peek() == &Token::Newline {
                            self.advance();
                            // Check if next token is at top level (not indented)
                            if self.is_top_level_token() { break; }
                            continue;
                        }
                        self.advance();
                    }
                }
                let fields = Vec::new();
                constructors.push(Constructor { name: c_name, fields });
                self.skip_newlines();
            }
            return Ok(Term::Data(name, params, constructors));
        }

        // Simple ADT: `data Name params = Con1 | Con2`
        while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
            match self.advance().node {
                Token::Identifier(p) => params.push(p),
                _ => break,
            }
        }
        self.consume(Token::Assign, "Expected = in data definition")?;
        let mut constructors = Vec::new();
        while self.peek() != &Token::EOF {
            self.skip_newlines();
            let c_name = match self.advance().node {
                Token::Identifier(n) => n,
                _ => break,
            };
            // Skip constructor fields/types until | or newline
            while self.peek() != &Token::Pipe && self.peek() != &Token::Newline && self.peek() != &Token::EOF {
                self.advance();
            }
            let fields = Vec::new();
            constructors.push(Constructor { name: c_name, fields });
            self.skip_newlines();
            if self.peek() == &Token::Pipe { self.advance(); } else { break; }
        }
        Ok(Term::Data(name, params, constructors))
    }

    /// Checks if the current token is a top-level position (likely start of new construct)
    fn is_top_level_token(&self) -> bool {
        match self.peek() {
            Token::Identifier(_) | Token::LParen | Token::Data |
            Token::Module | Token::Import | Token::Interface |
            Token::Implementation | Token::Record | Token::Mutual |
            Token::LBracket => true,
            _ => false,
        }
    }

    /// Checks if current token starts a non-constructor declaration
    /// (i.e., not a GADT constructor — used to stop GADT constructor parsing)
    fn is_non_constructor_decl(&self) -> bool {
        match self.peek() {
            Token::Data | Token::Module | Token::Import | Token::Interface |
            Token::Implementation | Token::Record | Token::Mutual |
            Token::LBracket => true,
            Token::Identifier(_) => {
                let is_modifier = if let Token::Identifier(n) = self.peek() {
                    matches!(n.as_str(),
                        "public" | "export" | "total" | "partial" | "covering" |
                        "parameters")
                } else { false };
                is_modifier
            }
            _ => false,
        }
    }

    /// Parses `interface Name params where`.
    fn parse_interface(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // interface
        let name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected interface name"),
        };
        let mut params = Vec::new();
        while self.peek() != &Token::Where && self.peek() != &Token::EOF {
            match self.advance().node {
                Token::Identifier(p) => params.push(p),
                _ => break,
            }
        }
        if self.peek() == &Token::Where {
            self.advance();
        }
        // Skip interface body — consume until next top-level declaration
        self.skip_indented_block();
        Ok(Term::Interface(name, params, Vec::new()))
    }

    /// Parses `implementation InterfaceName TargetType where`.
    fn parse_implementation(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // implementation
        let iface = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected interface name"),
        };
        let mut target = String::new();
        // Consume implementation head until `where` or newline
        while self.peek() != &Token::Where && self.peek() != &Token::Newline && self.peek() != &Token::EOF {
            match self.advance().node {
                Token::Identifier(n) => target = n,
                _ => {}
            }
        }
        if self.peek() == &Token::Where {
            self.advance();
            self.skip_indented_block();
        }
        Ok(Term::Implementation(iface, target, Vec::new()))
    }

    /// Parses named implementations: `[Name] Interface Type where ...`
    fn parse_named_impl_or_skip(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // [
        // Get the implementation name
        let _impl_name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => "_named".to_string(),
        };
        self.consume(Token::RBracket, "Expected ] after named implementation")?;
        self.skip_newlines();

        // The rest is like a regular interface implementation
        let iface = match self.advance().node {
            Token::Identifier(n) => n,
            _ => "_iface".to_string(),
        };
        let mut target = String::new();
        while self.peek() != &Token::Where && self.peek() != &Token::Newline && self.peek() != &Token::EOF {
            match self.advance().node {
                Token::Identifier(n) => target = n,
                _ => {}
            }
        }
        if self.peek() == &Token::Where {
            self.advance();
            self.skip_indented_block();
        }
        Ok(Term::Implementation(iface, target, Vec::new()))
    }

    /// Parses `record Name where ...`.
    fn parse_record(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // record
        let name = match self.advance().node {
            Token::Identifier(n) => n,
            _ => panic!("Expected record name"),
        };
        // Skip optional params
        while self.peek() != &Token::Where && self.peek() != &Token::EOF {
            self.advance();
        }
        if self.peek() == &Token::Where {
            self.advance();
        }
        self.skip_indented_block();
        Ok(Term::Record(name, Vec::new()))
    }

    /// Parses `mutual ...`.
    fn parse_mutual(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // mutual
        Ok(Term::Mutual(Vec::new()))
    }

    /// Parses declarations with visibility/totality modifiers.
    /// e.g., `public export`, `export`, `total`, `partial`, `covering`
    fn parse_with_modifier(&mut self) -> Result<Term<'a>, CompilerError> {
        let _modifier = self.advance(); // consume the modifier
        self.skip_newlines();
        // If next is also a modifier, consume it too (e.g., "public export")
        let modifier_check = match self.peek() {
            Token::Identifier(n) => Some(n.clone()),
            _ => None,
        };
        if let Some(mod_name) = modifier_check {
            match mod_name.as_str() {
                "export" | "total" | "partial" | "covering" => {
                    self.advance();
                    self.skip_newlines();
                }
                _ => {}
            }
        }
        // Now parse the actual declaration
        if self.peek() != &Token::EOF {
            self.parse_declaration()
        } else {
            Ok(Term::Module("_modifier_eof".to_string()))
        }
    }

    /// Parses a `parameters` block.
    fn parse_parameters_block(&mut self) -> Result<Term<'a>, CompilerError> {
        self.advance(); // parameters
        // Skip the parameter list (...)
        if self.peek() == &Token::LParen {
            self.advance();
            let mut depth = 1;
            while self.peek() != &Token::EOF && depth > 0 {
                match self.peek() {
                    Token::LParen => { depth += 1; self.advance(); }
                    Token::RParen => { depth -= 1; self.advance(); }
                    _ => { self.advance(); }
                }
            }
        }
        // Parse the block body
        self.skip_newlines();
        let mut decls = Vec::new();
        while self.peek() != &Token::EOF {
            self.skip_newlines();
            if self.peek() == &Token::EOF { break; }
            // Stop if we hit something that looks like an un-indented declaration
            // For now, parse remaining declarations as part of the block
            match self.parse_declaration() {
                Ok(d) => decls.push(d),
                Err(_) => break,
            }
        }
        // Return the first declaration or a placeholder
        if let Some(first) = decls.into_iter().next() {
            Ok(first)
        } else {
            Ok(Term::Module("_params_block".to_string()))
        }
    }

    /// Parses a signature used as a standalone declaration.
    /// Wraps the signature type in the declaration stream.
    fn parse_signature_decl(&mut self) -> Result<Term<'a>, CompilerError> {
        let (_name, sig) = self.parse_signature()?;
        Ok(sig.clone())
    }

    /// Parses a definition used as a standalone declaration.
    /// Wraps the result in `Term::Def(name, args, body)` for the AST.
    fn parse_definition_decl(&mut self) -> Result<Term<'a>, CompilerError> {
        let name_token = self.advance();
        let def_col = name_token.span.col;
        let name = match name_token.node {
            Token::Identifier(n) => n,
            Token::Integer(i) => i.to_string(),
            Token::LParen => {
                // Operator definition like (++) xs ys = ...
                let mut op_name = String::from("(");
                while self.peek() != &Token::RParen && self.peek() != &Token::EOF {
                    let t = self.advance();
                    op_name.push_str(&format!("{:?}", t.node));
                }
                if self.peek() == &Token::RParen { self.advance(); }
                op_name.push(')');
                op_name
            }
            _ => {
                // Skip to next line and try to recover
                while self.peek() != &Token::Newline && self.peek() != &Token::EOF {
                    self.advance();
                }
                return Ok(Term::Module("_skip_def".to_string()));
            }
        };
        let mut args = Vec::new();
        // Collect argument patterns until `=`
        while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
            match self.peek() {
                Token::Newline => { self.advance(); continue; }
                Token::Identifier(_) | Token::Integer(_) | Token::LParen |
                Token::LBracket | Token::Underscore => {
                    let arg_token = self.advance();
                    match arg_token.node {
                        Token::Identifier(arg) => args.push(arg),
                        Token::Integer(i) => args.push(i.to_string()),
                        Token::LParen => {
                            // Pattern like (x :: xs) or (S k) — skip balanced parens
                            let mut depth = 1;
                            while self.peek() != &Token::EOF && depth > 0 {
                                match self.peek() {
                                    Token::LParen => { depth += 1; self.advance(); }
                                    Token::RParen => { depth -= 1; self.advance(); }
                                    _ => { self.advance(); }
                                }
                            }
                            args.push("_pat".to_string());
                        }
                        Token::LBracket => {
                            // Pattern like [] — skip balanced brackets
                            let mut depth = 1;
                            while self.peek() != &Token::EOF && depth > 0 {
                                match self.peek() {
                                    Token::LBracket => { depth += 1; self.advance(); }
                                    Token::RBracket => { depth -= 1; self.advance(); }
                                    _ => { self.advance(); }
                                }
                            }
                            args.push("_list_pat".to_string());
                        }
                        _ => args.push("_".to_string()),
                    }
                }
                Token::At => {
                    // Named pattern: @{name}
                    self.advance();
                    if self.peek() == &Token::LBrace {
                        self.advance();
                        while self.peek() != &Token::RBrace && self.peek() != &Token::EOF {
                            self.advance();
                        }
                        if self.peek() == &Token::RBrace { self.advance(); }
                    }
                    args.push("_at".to_string());
                }
                Token::Where => {
                    // Hit `where` before `=` — this is a definition with where clause
                    break;
                }
                Token::Pipe => {
                    // Hit `|` — this is a `with` view pattern
                    // Skip to `=`
                    while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
                        self.advance();
                    }
                    break;
                }
                _ => {
                    // Unknown token in argument position — skip it
                    self.advance();
                }
            }
        }

        if self.peek() == &Token::Where {
            // Definition followed by where clause but no body
            self.advance();
            self.skip_indented_block();
            let placeholder = unsafe { &*self.arena.alloc(Term::Var("_where".to_string())) };
            return Ok(Term::Def(name, args, placeholder));
        }

        if self.peek() == &Token::Assign {
            self.consume(Token::Assign, "Expected = in definition")?;
            let mut body = self.parse_expr()?;

            // Check for `where` clause after the body
            self.skip_newlines();
            if self.peek() == &Token::Where {
                self.advance();
                let mut local_decls = Vec::new();
                self.skip_newlines();

                // Parse where-clause local definitions.
                //
                // Indentation-based termination: where-clause declarations
                // are indented past the parent definition's column (`def_col`).
                // When we see an identifier at column <= def_col after
                // newlines, it's a new top-level declaration and we stop.
                while self.peek() != &Token::EOF && self.peek() != &Token::RParen {
                    if self.peek() == &Token::Newline {
                        self.skip_newlines();
                        continue;
                    }

                    // Check if next token is back at top-level indentation
                    if let Some(t) = self.cursor.peek_at(0) {
                        if t.span.col <= def_col {
                            break;
                        }
                    }

                    if let Ok(decl) = self.parse_declaration() {
                        local_decls.push(decl);
                    } else {
                        break;
                    }
                    self.skip_newlines();
                }
                if !local_decls.is_empty() {
                    body = unsafe { &*self.arena.alloc(Term::Where(body, local_decls)) };
                }
            }

            Ok(Term::Def(name, args, body))
        }
 else {
            // No `=` found — skip to next line
            while self.peek() != &Token::Newline && self.peek() != &Token::EOF {
                self.advance();
            }
            let placeholder = unsafe { &*self.arena.alloc(Term::Var("_no_eq".to_string())) };
            Ok(Term::Def(name, args, placeholder))
        }
    }

    /// Parses a function definition: `name args = body`.
    ///
    /// Returns the body term, the function name, and the argument names.
    pub fn parse_def(&mut self) -> Result<(&'a Term<'a>, String, Vec<String>), CompilerError> {
        let name_token = self.advance();
        let name = match name_token.node {
            Token::Identifier(n) => n,
            _ => panic!("Expected identifier in definition"),
        };
        let mut args = Vec::new();
        while self.peek() != &Token::Assign && self.peek() != &Token::EOF {
            let arg_token = self.advance();
            match arg_token.node {
                Token::Identifier(arg) => args.push(arg),
                _ => panic!("Expected argument name"),
            }
        }
        self.consume(Token::Assign, "Expected = in definition")?;
        let body = self.parse_expr()?;
        Ok((body, name, args))
    }

    /// Checks whether the current token starts a new declaration.
    ///
    /// # Business Logic
    /// This is used by the `do`-notation parser to know when to stop
    /// consuming statements.
    pub fn is_decl_start(&self) -> bool {
        match self.peek() {
            Token::Module | Token::Import | Token::Data | Token::Interface |
            Token::Implementation | Token::Record | Token::Mutual => true,
            Token::Identifier(_) => {
                let is_mod = if let Token::Identifier(n) = self.peek() {
                    matches!(n.as_str(),
                        "public" | "export" | "total" | "partial" | "covering" |
                        "parameters")
                } else { false };
                if is_mod { return true; }
                let mut idx = 1;
                while let Some(t) = self.cursor.peek_at(idx) {
                    if t.node == Token::Newline {
                        idx += 1;
                        continue;
                    }
                    return matches!(t.node, Token::Colon | Token::Assign);
                }
                false
            }
            _ => false,
        }
    }

    /// Skips an indented block — consumes tokens until we return to
    /// a top-level declaration or hit EOF.
    ///
    /// # Business Logic
    /// Many Idris 2 constructs (interface bodies, where clauses, record
    /// fields) use indentation-based nesting. Since our parser doesn't
    /// track indentation levels, we use a heuristic: skip newlines and
    /// non-declaration tokens until we see something that looks top-level.
    fn skip_indented_block(&mut self) {
        self.skip_newlines();
        while self.peek() != &Token::EOF {
            // A blank line (two consecutive newlines) often signals end of block
            if self.peek() == &Token::Newline {
                self.advance();
                self.skip_newlines();
                // If next token starts a declaration, we're done
                if self.is_top_level_token() || self.peek() == &Token::EOF {
                    break;
                }
                continue;
            }
            // If this looks like a top-level declaration, stop
            if self.is_non_constructor_decl() {
                break;
            }
            self.advance();
        }
    }
}
