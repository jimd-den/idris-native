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
        } else if c == '(' || c == ')' || c == '=' || c == '+' || c == '-' {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            if c == '=' {
                chars.next();
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    tokens.push("==".to_string());
                } else {
                    tokens.push("=".to_string());
                }
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
            }
        }

        let mut lhs = self.parse_app();
        
        while let Some(t) = self.peek() {
            if t == "==" || t == "+" || t == "-" {
                let op = self.advance().unwrap();
                let rhs = self.parse_app();
                let term = match op.as_str() {
                    "==" => Term::Eq(lhs, rhs),
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

    fn parse_app(&mut self) -> &'a Term<'a> {
        let mut expr = self.parse_primary();
        while let Some(t) = self.peek() {
            if t != "==" && t != "+" && t != "-" && t != "then" && t != "else" && t != ")" {
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
            if self.advance().unwrap() != ")" { panic!("Expected )"); }
            expr
        } else if let Ok(val) = t.parse::<i64>() {
            let term = Term::Integer(val);
            unsafe { &*self.arena.alloc(term) }
        } else {
            let term = Term::Var(t);
            unsafe { &*self.arena.alloc(term) }
        }
    }
}

