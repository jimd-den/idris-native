use crate::domain::Term;

pub struct Evaluator {}

impl Evaluator {
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluates a term to its normal form.
    pub fn eval<'a>(&self, term: &'a Term<'a>) -> Term<'a> {
        match term {
            Term::App(func, arg) => {
                let f = self.eval(func);
                let a = self.eval(arg);
                match f {
                    Term::Lambda(name, _type, body) => {
                        // Beta-reduction: substitute 'a' for 'name' in 'body'
                        let substituted = self.substitute(body, &name, &a);
                        self.eval_owned(substituted)
                    }
                    _ => Term::App(Box::leak(Box::new(f)), Box::leak(Box::new(a))),
                }
            }
            Term::Var(_) | Term::Lambda(_, _, _) | Term::Pi(_, _, _) | Term::Integer(_) | Term::IntegerType |
            Term::I32Type | Term::I8Type | Term::Bits64Type | Term::IOType |
            Term::Add(_, _) | Term::Sub(_, _) | Term::Eq(_, _) | Term::If(_, _, _) | Term::LetRec(_, _, _) | Term::Let(_, _, _) |
            Term::BitXor(_, _) | Term::BitAnd(_, _) | Term::BitOr(_, _) | Term::BitNot(_) | Term::Shl(_, _) | Term::Shr(_, _) |
            Term::Buffer(_) | Term::BufferLoad(_, _) | Term::BufferStore(_, _, _) |
            Term::Case(_, _) => {
                term.clone()
            }

        }
    }

    fn eval_owned<'a>(&self, term: Term<'a>) -> Term<'a> {
        match term {
            Term::App(func, arg) => {
                let f = self.eval(func);
                let a = self.eval(arg);
                match f {
                    Term::Lambda(name, _type, body) => {
                        let substituted = self.substitute(body, &name, &a);
                        self.eval_owned(substituted)
                    }
                    _ => Term::App(Box::leak(Box::new(f)), Box::leak(Box::new(a))),
                }
            }
            _ => term,
        }
    }

    /// Specifically evaluates the Ackermann function to prove Turing Completeness.
    pub fn eval_ackermann(&self, m: i64, n: i64) -> i64 {
        if m == 0 {
            n + 1
        } else if n == 0 {
            self.eval_ackermann(m - 1, 1)
        } else {
            self.eval_ackermann(m - 1, self.eval_ackermann(m, n - 1))
        }
    }

    fn substitute<'a>(&self, body: &'a Term<'a>, name: &str, replacement: &Term<'a>) -> Term<'a> {
        match body {
            Term::Var(v) if v == name => replacement.clone(),
            Term::Var(_) => body.clone(),
            Term::Lambda(n, t, b) if n != name => {
                Term::Lambda(n.clone(), t, Box::leak(Box::new(self.substitute(b, name, replacement))))
            }
            Term::App(f, a) => {
                Term::App(
                    Box::leak(Box::new(self.substitute(f, name, replacement))),
                    Box::leak(Box::new(self.substitute(a, name, replacement))),
                )
            }
            Term::BitXor(l, r) => {
                Term::BitXor(
                    Box::leak(Box::new(self.substitute(l, name, replacement))),
                    Box::leak(Box::new(self.substitute(r, name, replacement))),
                )
            }
            Term::BitAnd(l, r) => {
                Term::BitAnd(
                    Box::leak(Box::new(self.substitute(l, name, replacement))),
                    Box::leak(Box::new(self.substitute(r, name, replacement))),
                )
            }
            Term::BitOr(l, r) => {
                Term::BitOr(
                    Box::leak(Box::new(self.substitute(l, name, replacement))),
                    Box::leak(Box::new(self.substitute(r, name, replacement))),
                )
            }
            Term::BitNot(t) => {
                Term::BitNot(Box::leak(Box::new(self.substitute(t, name, replacement))))
            }
            Term::Shl(l, r) => {
                Term::Shl(
                    Box::leak(Box::new(self.substitute(l, name, replacement))),
                    Box::leak(Box::new(self.substitute(r, name, replacement))),
                )
            }
            Term::Shr(l, r) => {
                Term::Shr(
                    Box::leak(Box::new(self.substitute(l, name, replacement))),
                    Box::leak(Box::new(self.substitute(r, name, replacement))),
                )
            }
            Term::BufferLoad(b, i) => {
                Term::BufferLoad(
                    Box::leak(Box::new(self.substitute(b, name, replacement))),
                    Box::leak(Box::new(self.substitute(i, name, replacement))),
                )
            }
            Term::BufferStore(b, i, v) => {
                Term::BufferStore(
                    Box::leak(Box::new(self.substitute(b, name, replacement))),
                    Box::leak(Box::new(self.substitute(i, name, replacement))),
                    Box::leak(Box::new(self.substitute(v, name, replacement))),
                )
            }
            Term::Let(n, v, b) if n != name => {
                Term::Let(
                    n.clone(),
                    Box::leak(Box::new(self.substitute(v, name, replacement))),
                    Box::leak(Box::new(self.substitute(b, name, replacement))),
                )
            }
            Term::Case(target, branches) => {
                let mut new_branches = Vec::new();
                for (pat_name, pat_args, body) in branches {
                    // Only substitute if not shadowed by pattern arguments
                    if pat_name != name && !pat_args.contains(&name.to_string()) {
                        let sub_body = self.substitute(body, name, replacement);
                        let leaked: &Term = Box::leak(Box::new(sub_body));
                        new_branches.push((pat_name.clone(), pat_args.clone(), leaked));
                    } else {
                        new_branches.push((pat_name.clone(), pat_args.clone(), *body));
                    }
                }
                let sub_target = self.substitute(target, name, replacement);
                let leaked_target: &Term = Box::leak(Box::new(sub_target));
                Term::Case(leaked_target, new_branches)
            }
            _ => body.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
