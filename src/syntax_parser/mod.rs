use crate::core_terms::{Term, arena::Arena};

pub fn lex(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else if c == '-' {
            chars.next();
            if let Some(&'-') = chars.peek() {
                // Comment: skip until end of line
                while let Some(&c) = chars.peek() {
                    if c == '\n' { break; }
                    chars.next();
                }
            } else if let Some(&'>') = chars.peek() {
                chars.next();
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push("->".to_string());
            } else {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push("-".to_string());
            }
        } else if c == '(' || c == ')' || c == '=' || c == '+' || c == '`' || c == ':' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            if c == ':' {
                tokens.push(":".to_string());
                chars.next();
            } else if c == '=' {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push("==".to_string());
                } else if let Some(&'>') = chars.peek() {
                    chars.next();
                    tokens.push("=>".to_string());
                } else {
                    tokens.push("=".to_string());
                }
            } else if c == '`' {
                tokens.push("`".to_string());
                chars.next();
            } else {
                tokens.push(c.to_string());
                chars.next();
            }
        } else {
            current.push(c);
            chars.next();
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub struct Parser<'a, 'arena> {
    tokens: Vec<String>,
    pos: usize,
    arena: &'arena mut Arena<Term<'a>>,
}

impl<'a, 'arena> Parser<'a, 'arena> {
    pub fn new(tokens: Vec<String>, arena: &'arena mut Arena<Term<'a>>) -> Self {
        Self { tokens, pos: 0, arena }
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<String> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    pub fn parse_def(&mut self) -> (&'a Term<'a>, String, Vec<String>) {
        // e.g. "ack m n = ..."
        let name = self.advance().unwrap();
        let mut args = Vec::new();
        while let Some(t) = self.peek() {
            if t == "=" {
                self.advance(); // consume '='
                break;
            }
            args.push(self.advance().unwrap());
        }
        let body = self.parse_expr();
        (body, name, args)
    }

    pub fn parse_signature(&mut self) -> (String, &'a Term<'a>) {
        // e.g. "ack : Integer -> Integer"
        let name = self.advance().unwrap();
        if self.advance().unwrap() != ":" { panic!("Expected : in signature"); }
        let sig = self.parse_pi();
        (name, sig)
    }

    fn parse_pi(&mut self) -> &'a Term<'a> {
        if let Some(t) = self.peek() {
            if t == "(" {
                // Check for multiplicity binder like (1 x : Type)
                if let Some(next) = self.tokens.get(self.pos + 1) {
                    if next == "0" || next == "1" {
                        self.advance(); // (
                        let _q = self.advance().unwrap(); // multiplicity
                        let name = self.advance().unwrap();
                        self.advance(); // :
                        let ty = self.parse_expr();
                        self.advance(); // )
                        if self.peek().map(|s| s.as_str()) == Some("->") {
                            self.advance(); // ->
                            let body = self.parse_pi();
                            return unsafe { &*self.arena.alloc(Term::Pi(name, ty, body)) };
                        }
                    }
                }
            }
        }

        let lhs = self.parse_expr();
        if let Some(t) = self.peek() {
            if t == "->" {
                self.advance();
                let rhs = self.parse_pi();
                // Anonymous Pi (lhs -> rhs)
                let term = Term::Pi("_".to_string(), lhs, rhs);
                return unsafe { &*self.arena.alloc(term) };
            }
        }
        lhs
    }

    /// Parses a simple program consisting of a signature and a definition.
    pub fn parse_program(&mut self) -> (String, &'a Term<'a>, &'a Term<'a>, Vec<String>) {
        let (name_sig, sig) = self.parse_signature();
        let (body, name_def, args) = self.parse_def();
        if name_sig != name_def {
            panic!("Name mismatch: {} vs {}", name_sig, name_def);
        }
        (name_sig, sig, body, args)
    }

    fn parse_expr(&mut self) -> &'a Term<'a> {
        if let Some(t) = self.peek() {
            if t == "if" {
                self.advance(); // consume 'if'
                let cond = self.parse_expr();
                if self.advance().unwrap() != "then" { panic!("Expected then"); }
                let then_br = self.parse_expr();
                if self.advance().unwrap() != "else" { panic!("Expected else"); }
                let else_br = self.parse_expr();
                let term = Term::If(cond, then_br, else_br);
                return unsafe { &*self.arena.alloc(term) };
            } else if t == "let" {
                self.advance(); // consume 'let'
                let name = self.advance().unwrap();
                if self.advance().unwrap() != "=" { panic!("Expected = after let name"); }
                let val = self.parse_expr();
                if self.advance().unwrap() != "in" { panic!("Expected in after let value"); }
                let body = self.parse_expr();
                let term = Term::Let(name, val, body);
                return unsafe { &*self.arena.alloc(term) };
            } else if t == "case" {
                self.advance(); // consume 'case'
                let target = self.parse_expr();
                if self.advance().unwrap() != "of" { panic!("Expected of after case target"); }
                
                let mut branches = Vec::new();
                while let Some(t) = self.peek() {
                    if t == ")" || t == "in" || t == "then" || t == "else" { break; }
                    
                    let pat_name = self.advance().unwrap();
                    let mut pat_args = Vec::new();
                    // Simple pattern parsing: assume single name or name + args until =>
                    while let Some(next) = self.peek() {
                        if next == "=>" || next == "|" || next == ")" || next == "in" { break; }
                        pat_args.push(self.advance().unwrap());
                    }
                    
                    if self.advance().unwrap() != "=>" { panic!("Expected => in case branch"); }
                    let branch_body = self.parse_expr();
                    branches.push((pat_name, pat_args, branch_body));
                    
                    if self.peek().map(|s| s.as_str()) == Some("|") {
                        self.advance(); // consume '|'
                    } else {
                        break;
                    }
                }
                let term = Term::Case(target, branches);
                return unsafe { &*self.arena.alloc(term) };
            }
        }
        self.parse_comparison()
    }

    pub fn parse_adt(&mut self) -> crate::core_terms::AdtDefinition<'a> {
        // data Name params = Constr args | Constr args
        if self.advance().unwrap() != "data" { panic!("Expected data"); }
        let name = self.advance().unwrap();
        let mut params = Vec::new();
        while let Some(t) = self.peek() {
            if t == "=" { break; }
            params.push(self.advance().unwrap());
        }
        self.advance(); // consume '='
        
        let mut constructors = Vec::new();
        while let Some(t) = self.peek() {
            let c_name = self.advance().unwrap();
            let mut c_args = Vec::new();
            while let Some(next) = self.peek() {
                if next == "|" || next == ")" { break; }
                // For simplicity, parse args as primary terms (types)
                c_args.push(self.parse_primary());
            }
            constructors.push((c_name, c_args));
            
            if self.peek().map(|s| s.as_str()) == Some("|") {
                self.advance(); // consume '|'
            } else {
                break;
            }
        }
        
        crate::core_terms::AdtDefinition {
            name,
            params,
            constructors,
        }
    }

    fn parse_comparison(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_or();
        while let Some(t) = self.peek() {
            if t == "==" {
                self.advance();
                let rhs = self.parse_bitwise_or();
                let term = Term::Eq(lhs, rhs);
                lhs = unsafe { &*self.arena.alloc(term) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_bitwise_or(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_xor();
        while let Some(t) = self.peek() {
            if t == ".|." {
                self.advance();
                let rhs = self.parse_bitwise_xor();
                let term = Term::BitOr(lhs, rhs);
                lhs = unsafe { &*self.arena.alloc(term) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_bitwise_xor(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_bitwise_and();
        while let Some(t) = self.peek() {
            if t == "`" {
                // Check for `xor`
                if self.tokens.get(self.pos + 1).map(|s| s.as_str()) == Some("xor") && 
                   self.tokens.get(self.pos + 2).map(|s| s.as_str()) == Some("`") {
                    self.advance(); // `
                    self.advance(); // xor
                    self.advance(); // `
                    let rhs = self.parse_bitwise_and();
                    let term = Term::BitXor(lhs, rhs);
                    lhs = unsafe { &*self.arena.alloc(term) };
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_bitwise_and(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_shift();
        while let Some(t) = self.peek() {
            if t == ".&." {
                self.advance();
                let rhs = self.parse_shift();
                let term = Term::BitAnd(lhs, rhs);
                lhs = unsafe { &*self.arena.alloc(term) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_shift(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_arithmetic();
        while let Some(t) = self.peek() {
            if t == "`" {
                // Check for `shiftL` or `shiftR`
                let op_name = self.tokens.get(self.pos + 1).map(|s| s.as_str());
                if (op_name == Some("shiftL") || op_name == Some("shiftR")) &&
                   self.tokens.get(self.pos + 2).map(|s| s.as_str()) == Some("`") {
                    self.advance(); // `
                    let op = self.advance().unwrap();
                    self.advance(); // `
                    let rhs = self.parse_arithmetic();
                    let term = if op == "shiftL" {
                        Term::Shl(lhs, rhs)
                    } else {
                        Term::Shr(lhs, rhs)
                    };
                    lhs = unsafe { &*self.arena.alloc(term) };
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_arithmetic(&mut self) -> &'a Term<'a> {
        let mut lhs = self.parse_unary();
        while let Some(t) = self.peek() {
            if t == "+" || t == "-" {
                let op = self.advance().unwrap();
                let rhs = self.parse_unary();
                let term = match op.as_str() {
                    "+" => Term::Add(lhs, rhs),
                    "-" => Term::Sub(lhs, rhs),
                    _ => unreachable!(),
                };
                lhs = unsafe { &*self.arena.alloc(term) };
            } else {
                break;
            }
        }
        lhs
    }

    fn parse_unary(&mut self) -> &'a Term<'a> {
        if let Some(t) = self.peek() {
            if t == "complement" {
                self.advance();
                let body = self.parse_unary();
                let term = Term::BitNot(body);
                return unsafe { &*self.arena.alloc(term) };
            }
        }
        self.parse_app()
    }

    fn parse_app(&mut self) -> &'a Term<'a> {
        let mut expr = self.parse_primary();
        while let Some(t) = self.peek() {
            // Function application has higher precedence than infix operators
            if t != "==" && t != "+" && t != "-" && t != "then" && t != "else" && t != ")" &&
               t != ".|." && t != ".&." && t != "in" && t != "`" && t != "->" && t != "of" && t != "=>" {
                let arg = self.parse_primary();
                let term = Term::App(expr, arg);
                expr = unsafe { &*self.arena.alloc(term) };
            } else {
                break;
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> &'a Term<'a> {
        let t = self.advance().unwrap();
        if t == "(" {
            let expr = self.parse_expr();
            if self.advance().unwrap() != ")" { panic!("Expected ) "); }
            expr
        } else if t == "i32" {
            let term = Term::I32Type;
            unsafe { &*self.arena.alloc(term) }
        } else if t == "i8" {
            let term = Term::I8Type;
            unsafe { &*self.arena.alloc(term) }
        } else if t == "Integer" {
            let term = Term::IntegerType;
            unsafe { &*self.arena.alloc(term) }
        } else if t == "buffer" {
            let size_token = self.advance().unwrap();
            let size = size_token.parse::<usize>().expect("Expected buffer size");
            let term = Term::Buffer(size);
            unsafe { &*self.arena.alloc(term) }
        } else if t == "getBits64" {
            let buffer = self.parse_primary();
            let index = self.parse_primary();
            let term = Term::BufferLoad(buffer, index);
            unsafe { &*self.arena.alloc(term) }
        } else if t == "setBits64" {
            let buffer = self.parse_primary();
            let index = self.parse_primary();
            let value = self.parse_primary();
            let term = Term::BufferStore(buffer, index, value);
            unsafe { &*self.arena.alloc(term) }
        } else if let Ok(val) = t.parse::<i64>() {
            let term = Term::Integer(val);
            unsafe { &*self.arena.alloc(term) }
        } else {
            let term = Term::Var(t);
            unsafe { &*self.arena.alloc(term) }
        }
    }
}

#[cfg(test)]
mod tests {
    pub mod sha256_syntax_tests;
}
