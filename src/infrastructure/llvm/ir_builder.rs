//! # LLVM IR Builder (Infrastructure)
//!
//! This module implements the `IRBuilder`, which is responsible for 
//! translating high-level `Term` nodes into LLVM IR instructions.

use crate::domain::Term;
use std::collections::{HashMap, HashSet};

/// Represents the layout of a constructor in memory.
#[derive(Debug, Clone)]
pub struct ConstructorLayout {
    pub tag: u32,
    pub field_count: usize,
}

/// The IRBuilder translates our `Term` AST directly into LLVM IR.
pub struct IRBuilder {
    pub instructions: Vec<String>,
    pub function_definitions: std::collections::HashMap<String, String>,
    pub string_literals: std::collections::HashMap<String, String>,
    pub type_env: std::collections::HashMap<String, ConstructorLayout>,
    pub known_functions: HashSet<String>,
    next_reg: usize,
    label_counter: usize,
    fn_counter: usize,
    pat_counter: usize,
    bit_width: u32,
}

impl IRBuilder {
    pub fn new() -> Self {
        let mut type_env = std::collections::HashMap::new();
        // Built-in List constructors
        type_env.insert("Nil".to_string(), ConstructorLayout { tag: 0, field_count: 0 });
        type_env.insert("::".to_string(), ConstructorLayout { tag: 1, field_count: 2 });
        // Built-in Nat constructors
        type_env.insert("Z".to_string(), ConstructorLayout { tag: 0, field_count: 0 });
        type_env.insert("S".to_string(), ConstructorLayout { tag: 1, field_count: 1 });
        
        Self {
            instructions: Vec::new(),
            function_definitions: std::collections::HashMap::new(),
            string_literals: std::collections::HashMap::new(),
            type_env,
            known_functions: HashSet::new(),
            next_reg: 1,
            label_counter: 0,
            fn_counter: 0,
            pat_counter: 0,
            bit_width: 64,
        }
    }

    /// Generates a unique placeholder name for pattern matching.
    pub fn new_placeholder(&mut self) -> String {
        self.pat_counter += 1;
        format!("_pat_{}", self.pat_counter)
    }

    /// Sanitizes an Idris identifier for LLVM, escaping special characters
    /// and wrapping it in quotes to prevent collisions.
    pub fn sanitize_id(&self, id: &str) -> String {
        let mut sanitized = String::new();
        let mut input = id;
        
        // Handle holes
        if id.starts_with('?') {
            sanitized.push_str("_hole_");
            input = &id[1..];
        }

        for c in input.chars() {
            match c {
                '.' | '-' | ' ' | '(' | ')' | '[' | ']' | ',' | '?' => sanitized.push('_'),
                _ => sanitized.push(c),
            }
        }

        format!("\"{}\"", sanitized)
    }

    /// Escapes a string for LLVM IR string literals (hexadecimal escaping).
    pub fn escape_string(&self, s: &str) -> String {
        let mut escaped = String::new();
        for b in s.as_bytes() {
            match b {
                b'\"' => escaped.push_str("\\22"),
                b'\\' => escaped.push_str("\\5C"),
                b if *b < 32 || *b > 126 => {
                    escaped.push_str(&format!("\\{:02X}", b));
                }
                _ => escaped.push(*b as char),
            }
        }
        escaped
    }

    /// Sets the default bit width for integer operations.
    pub fn set_bit_width(&mut self, width: u32) {
        self.bit_width = width;
    }

    fn new_string_label(&mut self) -> String {
        let label = format!("str_{}", self.string_literals.len());
        label
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

    fn resolve_global_name(&self, name: &str) -> Option<String> {
        let direct_match = self.known_functions.contains(name) || self.function_definitions.contains_key(&self.sanitize_id(name));
        if direct_match {
            return Some(format!("@{}", self.sanitize_id(name)));
        }

        if let Some((_, short_name)) = name.rsplit_once('.') {
            let short_match = self.known_functions.contains(short_name)
                || self.function_definitions.contains_key(&self.sanitize_id(short_name));
            if short_match {
                return Some(format!("@{}", self.sanitize_id(short_name)));
            }
        }

        None
    }

    fn collect_app_chain<'a>(&self, term: &'a Term<'a>) -> (&'a Term<'a>, Vec<&'a Term<'a>>) {
        let mut args = Vec::new();
        let mut current = term;

        while let Term::App(func, arg) = current {
            args.push(*arg);
            current = func;
        }

        args.reverse();
        (current, args)
    }

    /// Lowers a high-level term into LLVM IR.
    pub fn lower_term(&mut self, term: &Term, env: &HashMap<String, String>) -> String {
        let ty = String::from("i") + &self.bit_width.to_string();

        if let Term::App(_, _) = term {
            let (head, args) = self.collect_app_chain(term);
            if args.len() > 1 {
                if let Term::Var(name) = head {
                    if let Some(global_name) = self.resolve_global_name(name) {
                        let lowered_args: Vec<String> = args.iter().map(|arg| self.lower_term(arg, env)).collect();
                        let res = self.new_reg();
                        let call_args = lowered_args
                            .into_iter()
                            .map(|arg| format!("{} {}", ty, arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        self.instructions.push(format!("  {} = call {} {}({})\n", res, ty, global_name, call_args));
                        return res;
                    }
                }
            }
        }

        match term {
            Term::Var(name) => {
                if name == "True" { return String::from("1"); }
                if name == "False" { return String::from("0"); }
                if name == "putStr" || name == "putStrLn" || name == "getLine" || name == "print_int" {
                    let mut s = String::from("@");
                    s.push_str(name);
                    return s;
                }
                if name == "print" {
                    return String::from("@print");
                }
                if let Some(layout) = self.type_env.get(name) {
                    if layout.field_count == 0 {
                        return layout.tag.to_string();
                    } else {
                        // For constructors with fields, we need a global reference
                        // so that Term::App can call it if it's treated as a function.
                        let mut s = String::from("@");
                        s.push_str(&self.sanitize_id(name));
                        return s;
                    }
                }
                if let Some(global_name) = self.resolve_global_name(name) {
                    return global_name;
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
            Term::String(s) => {
                let label = if let Some(l) = self.string_literals.get(s) {
                    l.clone()
                } else {
                    let l = self.new_string_label();
                    self.string_literals.insert(s.clone(), l.clone());
                    l
                };
                let res = self.new_reg();
                let mut instr = String::from("  ");
                instr.push_str(&res); instr.push_str(" = ptrtoint [");
                instr.push_str(&(s.len() + 1).to_string());
                instr.push_str(" x i8]* @"); instr.push_str(&label);
                instr.push_str(" to i64\n");
                self.instructions.push(instr);
                res
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
            Term::Append(l, r) => {
                let lv = self.lower_term(l, env);
                let rv = self.lower_term(r, env);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = call i64 @concat(i64 {}, i64 {})\n", res, lv, rv));
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
                self.function_definitions.insert(fn_name.clone(), fn_def);
                
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
                let mut target_val = self.lower_term(target, env);
                if branches.is_empty() { return String::from("0"); }

                // Check if any branch is a constructor match
                let mut is_adt_match = false;
                for (pat, _, _) in branches {
                    if self.type_env.contains_key(pat) {
                        is_adt_match = true;
                        break;
                    }
                }

                let mut val = target_val.clone();
                let mut struct_ptr = String::new();
                let generic_struct = "{ i64, [0 x i64] }";

                if is_adt_match {
                    // target_val is currently a i64 (pointer cast to int)
                    // We need to: 
                    // 1. cast back to pointer
                    // 2. load tag from index 0
                    let ptr = self.new_reg();
                    self.instructions.push(format!("  {} = inttoptr {} {} to {}*\n", ptr, ty, target_val, generic_struct));
                    struct_ptr = ptr;
                    let tag_ptr = self.new_reg();
                    self.instructions.push(format!("  {} = getelementptr {}, {}* {}, i32 0, i32 0\n", tag_ptr, generic_struct, generic_struct, struct_ptr));
                    let tag_val = self.new_reg();
                    self.instructions.push(format!("  {} = load i64, i64* {}\n", tag_val, tag_ptr));
                    val = tag_val;
                }

                let merge_label = self.new_label("case_merge");
                let mut default_label = merge_label.clone();
                let mut branch_labels = Vec::new();
                let mut cases = Vec::new();

                // 1. Identify default and generate labels
                for (pat, _, _) in branches {
                    let label = self.new_label("case_branch");
                    branch_labels.push(label.clone());
                    if pat == "_" {
                        default_label = label;
                    } else {
                        // If ADT match, pat is the constructor name, use its tag
                        let case_val = if is_adt_match {
                            self.type_env.get(pat).map(|l| l.tag.to_string()).unwrap_or_else(|| pat.clone())
                        } else {
                            pat.clone()
                        };
                        cases.push((case_val, label));
                    }
                }

                // 2. Emit switch instruction
                let mut switch_instr = format!("  switch {} {}, label %{} [\n", ty, val, default_label);
                for (pat, label) in &cases {
                    switch_instr.push_str(&format!("    {} {}, label %{}\n", ty, pat, label));
                }
                switch_instr.push_str("  ]\n");
                self.instructions.push(switch_instr);

                // 3. Lower each branch
                let mut phi_entries = Vec::new();
                for (i, (pat, args, body)) in branches.iter().enumerate() {
                    let label = &branch_labels[i];
                    self.instructions.push(format!("{}:\n", label));
                    
                    let mut branch_env = env.clone();
                    if is_adt_match && pat != "_" {
                        // Extract fields and bind to args
                        if let Some(layout) = self.type_env.get(pat).cloned() {
                            // LLVM GEP on generic {i64, [0 x i64]} works if we use the right indices
                            for (j, arg_name) in args.iter().enumerate() {
                                if j < layout.field_count {
                                    let field_ptr = self.new_reg();
                                    self.instructions.push(format!("  {} = getelementptr {}, {}* {}, i32 0, i32 1, i32 {}\n", 
                                        field_ptr, generic_struct, generic_struct, struct_ptr, j));
                                    let field_val = self.new_reg();
                                    self.instructions.push(format!("  {} = load i64, i64* {}\n", field_val, field_ptr));
                                    branch_env.insert(arg_name.clone(), field_val);
                                }
                            }
                        }
                    }
                    
                    let branch_val = self.lower_term(body, &branch_env);
                    phi_entries.push((branch_val, label.clone()));
                    self.instructions.push(format!("  br label %{}\n", merge_label));
                }

                // 4. Merge
                self.instructions.push(format!("{}:\n", merge_label));
                let res = self.new_reg();
                let mut phi_instr = format!("  {} = phi {} ", res, ty);
                for (i, (val, label)) in phi_entries.iter().enumerate() {
                    if i > 0 { phi_instr.push_str(", "); }
                    phi_instr.push_str(&format!("[ {}, %{} ]", val, label));
                }
                phi_instr.push('\n');
                self.instructions.push(phi_instr);
                res
            }

            Term::Do(stmts) => {
                let mut current_env = env.clone();
                let mut last_res = "0".to_string();
                for stmt in stmts {
                    match stmt {
                        Term::Bind(name, action) => {
                            let mut res = self.lower_term(action, &current_env);
                            if res.starts_with("@") {
                                let r = self.new_reg();
                                self.instructions.push(format!("  {} = call {} {}()\n", r, ty, res));
                                res = r;
                            }
                            current_env.insert(name.clone(), res.clone());
                            last_res = res;
                        }
                        _ => {
                            let mut res = self.lower_term(stmt, &current_env);
                            if res.starts_with("@") {
                                let r = self.new_reg();
                                self.instructions.push(format!("  {} = call {} {}()\n", r, ty, res));
                                res = r;
                            }
                            last_res = res;
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
                    let sanitized_arg = self.sanitize_id(arg);
                    arg_str.push_str(&ty); arg_str.push_str(" %"); arg_str.push_str(&sanitized_arg.replace("\"", ""));
                    let mut val_name = String::from("%");
                    val_name.push_str(&sanitized_arg.replace("\"", ""));
                    inner_env.insert(arg.clone(), val_name);
                }
                
                let res_reg = self.lower_term(body, &inner_env);
                let mut fn_def = String::from("define ");
                fn_def.push_str(&ty); fn_def.push_str(" @"); fn_def.push_str(&self.sanitize_id(name));
                fn_def.push_str("("); fn_def.push_str(&arg_str); fn_def.push_str(") {\n");
                for instr in self.instructions.drain(..) {
                    fn_def.push_str(&instr);
                }
                fn_def.push_str("  ret "); fn_def.push_str(&ty); fn_def.push_str(" ");
                fn_def.push_str(&res_reg); fn_def.push_str("\n}\n");
                
                self.function_definitions.entry(self.sanitize_id(name)).or_insert(fn_def);
                String::from("void")
            }
            
            Term::Module(_) | Term::Import(_) | Term::Interface(_, _, _) |
            Term::Implementation(_, _, _) | Term::Record(_, _) | Term::Mutual(_) => {
                "void".to_string()
            }
            
            Term::Data(_name, _params, constructors) => {
                for (i, con) in constructors.iter().enumerate() {
                    self.type_env.insert(con.name.clone(), ConstructorLayout {
                        tag: i as u32,
                        field_count: con.fields.len(),
                    });

                    if con.fields.len() > 0 {
                        // Generate a global constructor function
                        let mut arg_str = String::new();
                        for j in 0..con.fields.len() {
                            if !arg_str.is_empty() { arg_str.push_str(", "); }
                            arg_str.push_str(&format!("{} %f{}", ty, j));
                        }
                        
                        let mut con_def = format!("define {} @{}({}) {{\n", ty, self.sanitize_id(&con.name), arg_str);
                        let struct_ty = format!("{{ i64, [{0} x i64] }}", con.fields.len());
                        
                        // Use a temporary IR builder for the constructor body to avoid polluting current instructions
                        let ptr = "%1"; // First register in new function
                        con_def.push_str(&format!("  {} = alloca {}\n", ptr, struct_ty));
                        
                        let tag_ptr = "%2";
                        con_def.push_str(&format!("  {} = getelementptr {}, {}* {}, i32 0, i32 0\n", tag_ptr, struct_ty, struct_ty, ptr));
                        con_def.push_str(&format!("  store i64 {}, i64* {}\n", i, tag_ptr));
                        
                        let mut next_reg = 3;
                        for j in 0..con.fields.len() {
                            let field_ptr = format!("%{}", next_reg);
                            con_def.push_str(&format!("  {} = getelementptr {}, {}* {}, i32 0, i32 1, i32 {}\n", field_ptr, struct_ty, struct_ty, ptr, j));
                            con_def.push_str(&format!("  store i64 %f{}, i64* {}\n", j, field_ptr));
                            next_reg += 1;
                        }
                        
                        let res = format!("%{}", next_reg);
                        con_def.push_str(&format!("  {} = ptrtoint {}* {} to i64\n", res, struct_ty, ptr));
                        con_def.push_str(&format!("  ret i64 {}\n}}\n", res));
                        
                        self.function_definitions.insert(self.sanitize_id(&con.name), con_def);
                    }
                }
                "void".to_string()
            }
            
            Term::Where(t, defs) => {
                // Lower local definitions first to register them in function_definitions HashMap
                for def in defs {
                    match def {
                        Term::Def(_, _, _) | Term::Data(_, _, _) => {
                            self.lower_term(def, env);
                        }
                        Term::Where(_, nested_defs) => {
                            // Recursively register definitions in nested where blocks
                            self.lower_term(def, env);
                        }
                        _ => { self.lower_term(def, env); }
                    }
                }
                // Finally lower the body
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
