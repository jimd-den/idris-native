//! # LLVM IR Builder (Infrastructure)
//!
//! This module implements the `IRBuilder`, which is responsible for 
//! translating high-level `Term` nodes into LLVM IR instructions.

use crate::domain::Term;
use std::collections::HashMap;

/// The IRBuilder translates our `Term` AST directly into LLVM IR.
pub struct IRBuilder {
    pub instructions: Vec<String>,
    pub function_definitions: Vec<String>,
    next_reg: usize,
    label_counter: usize,
    fn_counter: usize,
    bit_width: u32,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            function_definitions: Vec::new(),
            next_reg: 1,
            label_counter: 0,
            fn_counter: 0,
            bit_width: 64,
        }
    }

    /// Sets the default bit width for integer operations.
    pub fn set_bit_width(&mut self, width: u32) {
        self.bit_width = width;
    }

    fn new_reg(&mut self) -> String {
        let mut s = String::from("%");
        s.push_str(&self.next_reg.to_string());
        self.next_reg += 1;
        s
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let mut s = String::from(prefix);
        s.push_str("_");
        s.push_str(&self.label_counter.to_string());
        self.label_counter += 1;
        s
    }

    fn new_fn_name(&mut self) -> String {
        let mut s = String::from("lambda_fn_");
        s.push_str(&self.fn_counter.to_string());
        self.fn_counter += 1;
        s
    }

    /// Lowers a high-level term into LLVM IR.
    pub fn lower_term(&mut self, term: &Term, env: &HashMap<String, String>) -> String {
        let ty = String::from("i") + &self.bit_width.to_string();

        match term {
            Term::Var(name) => {
                if name == "True" { return String::from("1"); }
                if name == "False" { return String::from("0"); }
                if name == "putStr" || name == "putStrLn" || name == "getLine" || name == "print_int" {
                    let mut s = String::from("@");
                    s.push_str(name);
                    return s;
                }
                env.get(name).cloned().unwrap_or_else(|| {
                    let mut s = String::from("%");
                    s.push_str(name);
                    s
                })
            }
            Term::Integer(n) => n.to_string(),
            Term::Float(bits) => {
                let mut s = String::from("0x");
                s.push_str(&format!("{:x}", bits));
                s
            }
            Term::String(_s) => {
                String::from("0")
            }
            Term::Char(c) => {
                (*c as u32).to_string()
            }
            
            Term::Add(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = add {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::Sub(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = sub {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::Mul(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = mul {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::Div(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = sdiv {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::Eq(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let cmp = self.new_reg();
                self.instructions.push(format!("  {} = icmp eq {} {}, {}\n", cmp, ty, lv, rv));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = zext i1 {} to {}\n", res, cmp, ty));
                res
            }
            Term::Lt(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let cmp = self.new_reg();
                self.instructions.push(format!("  {} = icmp slt {} {}, {}\n", cmp, ty, lv, rv));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = zext i1 {} to {}\n", res, cmp, ty));
                res
            }
            Term::Gt(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let cmp = self.new_reg();
                self.instructions.push(format!("  {} = icmp sgt {} {}, {}\n", cmp, ty, lv, rv));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = zext i1 {} to {}\n", res, cmp, ty));
                res
            }
            
            Term::BitAnd(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = and {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::BitOr(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = or {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::BitXor(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = xor {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::BitNot(t) => {
                let v = self.lower_term(t, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = xor {} {}, -1\n", res, ty, v));
                res
            }
            Term::Shl(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = shl {} {}, {}\n", res, ty, lv, rv));
                res
            }
            Term::Shr(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = lshr {} {}, {}\n", res, ty, lv, rv));
                res
            }
            
            Term::If(c, t, e) => {
                let cond_val = self.lower_term(c, env);
                let cond_bool = self.new_reg();
                self.instructions.push(format!("  {} = icmp ne {} {}, 0\n", cond_bool, ty, cond_val));
                
                let then_label = self.new_label("then");
                let else_label = self.new_label("else");
                let merge_label = self.new_label("if_merge");
                
                self.instructions.push(format!("  br i1 {}, label %{}, label %{}\n", cond_bool, then_label, else_label));
                
                self.instructions.push(format!("{}:\n", then_label));
                let then_val = self.lower_term(t, env);
                self.instructions.push(format!("  br label %{}\n", merge_label));
                
                self.instructions.push(format!("{}:\n", else_label));
                let else_val = self.lower_term(e, env);
                self.instructions.push(format!("  br label %{}\n", merge_label));
                
                self.instructions.push(format!("{}:\n", merge_label));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = phi {} [ {}, %{} ], [ {}, %{} ]\n", res, ty, then_val, then_label, else_val, else_label));
                res
            }
            
            Term::Lambda(name, _type, body) => {
                let fn_name = self.new_fn_name();
                let mut inner_builder = IRBuilder::new();
                inner_builder.bit_width = self.bit_width;
                let mut inner_env = env.clone();
                let mut var_name = String::from("%");
                var_name.push_str(name);
                inner_env.insert(name.clone(), var_name);
                
                let res_reg = inner_builder.lower_term(body, &inner_env);
                
                let mut fn_def = String::from("define ");
                fn_def.push_str(&ty); fn_def.push_str(" @"); fn_def.push_str(&fn_name);
                fn_def.push_str("("); fn_def.push_str(&ty); fn_def.push_str(" %");
                fn_def.push_str(name); fn_def.push_str(") {\n");
                for instr in inner_builder.instructions {
                    fn_def.push_str(&instr);
                }
                fn_def.push_str("  ret "); fn_def.push_str(&ty); fn_def.push_str(" ");
                fn_def.push_str(&res_reg); fn_def.push_str("\n}\n");
                self.function_definitions.push(fn_def);
                
                let res = self.new_reg();
                let mut ptrtoint = String::from("  ");
                ptrtoint.push_str(&res); ptrtoint.push_str(" = ptrtoint "); ptrtoint.push_str(&ty);
                ptrtoint.push_str(" ("); ptrtoint.push_str(&ty); ptrtoint.push_str(")* @");
                ptrtoint.push_str(&fn_name); ptrtoint.push_str(" to i64\n");
                self.instructions.push(ptrtoint);
                res
            }
            
            Term::App(f, a) => {
                let fv = self.lower_term(f, env);
                let av = self.lower_term(a, env);
                
                if fv.starts_with("@") {
                    let res = self.new_reg();
                    let mut instr = String::from("  ");
                    instr.push_str(&res); instr.push_str(" = call "); instr.push_str(&ty);
                    instr.push_str(" "); instr.push_str(&fv); instr.push_str("(");
                    instr.push_str(&ty); instr.push_str(" "); instr.push_str(&av); instr.push_str(")\n");
                    self.instructions.push(instr);
                    return res;
                }

                let fn_ptr = self.new_reg();
                let mut cast = String::from("  ");
                cast.push_str(&fn_ptr); cast.push_str(" = inttoptr "); cast.push_str(&ty);
                cast.push_str(" "); cast.push_str(&fv); cast.push_str(" to "); cast.push_str(&ty);
                cast.push_str(" ("); cast.push_str(&ty); cast.push_str(" )*\n");
                self.instructions.push(cast);
                let res = self.new_reg();
                let mut call = String::from("  ");
                call.push_str(&res); call.push_str(" = call "); call.push_str(&ty);
                call.push_str(" "); call.push_str(&fn_ptr); call.push_str("(");
                call.push_str(&ty); call.push_str(" "); call.push_str(&av); call.push_str(")\n");
                self.instructions.push(call);
                res
            }
            
            Term::Let(name, val, body) => {
                let v = self.lower_term(val, env);
                let mut new_env = env.clone();
                new_env.insert(name.clone(), v);
                self.lower_term(body, &new_env)
            }
            
            Term::LetRec(name, val, body) => {
                let mut rec_env = env.clone();
                rec_env.insert(name.clone(), String::from("0")); 
                let v = self.lower_term(val, &rec_env);
                let mut final_env = env.clone();
                final_env.insert(name.clone(), v);
                self.lower_term(body, &final_env)
            }
            
            Term::Case(target, branches) => {
                let val = self.lower_term(target, env);
                if branches.is_empty() { return String::from("0"); }

                let mut labels = Vec::new();
                let mut vals = Vec::new();

                for (pat, _, body) in branches {
                    if pat != "_" {
                        let cmp = self.new_reg();
                        self.instructions.push(format!("  {} = icmp eq {} {}, {}\n", cmp, ty, val, pat));
                        let match_label = self.new_label("case_match");
                        let next_label = self.new_label("case_next");
                        self.instructions.push(format!("  br i1 {}, label %{}, label %{}\n", cmp, match_label, next_label));
                        
                        self.instructions.push(format!("{}:\n", match_label));
                        let branch_val = self.lower_term(body, env);
                        self.instructions.push("  br label %case_merge\n".to_string());
                        
                        labels.push(match_label);
                        vals.push(branch_val);
                        self.instructions.push(format!("{}:\n", next_label));
                    } else {
                        let branch_val = self.lower_term(body, env);
                        self.instructions.push("  br label %case_merge\n".to_string());
                        labels.push("wildcard".to_string());
                        vals.push(branch_val);
                    }
                }

                self.instructions.push("case_merge:\n".to_string());
                let phi_res = self.new_reg();
                if vals.len() >= 2 {
                    self.instructions.push(format!("  {} = phi {} [ {}, %{} ], [ {}, %{} ]\n", phi_res, ty, vals[0], labels[0], vals[1], labels[1]));
                } else {
                    self.instructions.push(format!("  {} = phi {} [ {}, %{} ]\n", phi_res, ty, vals.get(0).unwrap_or(&String::from("0")), labels.get(0).unwrap_or(&String::from("somewhere"))));
                }
                phi_res
            }

            Term::Do(stmts) => {
                let mut current_env = env.clone();
                let mut last_res = "0".to_string();
                for stmt in stmts {
                    match stmt {
                        Term::Bind(name, action) => {
                            let res = self.lower_term(action, &current_env);
                            current_env.insert(name.clone(), res.clone());
                            last_res = res;
                        }
                        _ => {
                            last_res = self.lower_term(stmt, &current_env);
                        }
                    }
                }
                last_res
            }
            Term::Bind(name, action) => {
                self.lower_term(action, env)
            }
            
            Term::Pi(_, _, _, _) | Term::IntegerType | Term::FloatType | Term::StringType | Term::CharType |
            Term::I32Type | Term::I8Type | Term::Bits64Type | Term::IOType | Term::TypeType |
            Term::Universe(_) => {
                String::from("0")
            }

            Term::Def(name, args, body) => {
                let mut arg_str = String::new();
                let mut inner_env = env.clone();
                for arg in args {
                    if !arg_str.is_empty() { arg_str.push_str(", "); }
                    arg_str.push_str(&ty); arg_str.push_str(" %"); arg_str.push_str(arg);
                    let mut val_name = String::from("%");
                    val_name.push_str(arg);
                    inner_env.insert(arg.clone(), val_name);
                }
                
                let res_reg = self.lower_term(body, &inner_env);
                let mut fn_def = String::from("define ");
                fn_def.push_str(&ty); fn_def.push_str(" @"); fn_def.push_str(name);
                fn_def.push_str("("); fn_def.push_str(&arg_str); fn_def.push_str(") {\n");
                for instr in self.instructions.drain(..) {
                    fn_def.push_str(&instr);
                }
                fn_def.push_str("  ret "); fn_def.push_str(&ty); fn_def.push_str(" ");
                fn_def.push_str(&res_reg); fn_def.push_str("\n}\n");
                self.function_definitions.push(fn_def);
                String::from("void")
            }
            
            Term::Module(_) | Term::Import(_) | Term::Data(_, _, _) | Term::Interface(_, _, _) |
            Term::Implementation(_, _, _) | Term::Record(_, _) | Term::Mutual(_) => {
                "void".to_string()
            }
            
            Term::Where(t, defs) => {
                for def in defs {
                    self.lower_term(def, env);
                }
                self.lower_term(t, env)
            }

            Term::Buffer(size) => {
                let res = self.new_reg();
                self.instructions.push(format!("  {} = alloca [{} x {}]\n", res, size, ty));
                res
            }
            Term::BufferLoad(b, i) => {
                let bv = self.lower_term(b, env);
                let iv = self.lower_term(i, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = getelementptr [64 x {}], [64 x {}]* {}, i64 0, i64 {}\n", res, ty, ty, bv, iv));
                let load_res = self.new_reg();
                self.instructions.push(format!("  {} = load {}, {}* {}\n", load_res, ty, ty, res));
                load_res
            }
            Term::BufferStore(b, i, v) => {
                let bv = self.lower_term(b, env);
                let iv = self.lower_term(i, env);
                let vv = self.lower_term(v, env);
                let ptr = self.new_reg();
                self.instructions.push(format!("  {} = getelementptr [64 x {}], [64 x {}]* {}, i64 0, i64 {}\n", ptr, ty, ty, bv, iv));
                self.instructions.push(format!("  store {} {}, {}* {}\n", ty, vv, ty, ptr));
                "void".to_string()
            }
        }
    }
}
