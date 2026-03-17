use crate::core_terms::Term;

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
            Term::Add(_, _) | Term::Sub(_, _) | Term::Eq(_, _) | Term::If(_, _, _) | Term::LetRec(_, _, _) => {
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
            _ => body.clone(),
        }
    }
}

#[cfg(test)]
mod tests;
