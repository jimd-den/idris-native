//! # LLVM IR Builder (Infrastructure)
//!
//! This module implements the `IRBuilder`, which is responsible for 
//! translating high-level `Term` nodes into LLVM IR instructions.

use crate::domain::Term;
use std::collections::{HashMap, HashSet};

use super::string_interner::StringInterner;
use super::type_registry::{TypeRegistry, ConstructorLayout};
use super::function_lifter::FunctionLifter;

/// ISO 8601: 2026-03-19T20:55:29Z
/// Pattern: Facade Pattern (Workable iʃɛ́)
/// Why: IRBuilder now acts as a facade, coordinating specialized components 
/// (StringInterner, TypeRegistry, FunctionLifter) while focusing solely on 
/// instruction and register management.
pub struct IRBuilder {
    pub instructions: Vec<String>,
    pub string_interner: StringInterner,
    pub type_registry: TypeRegistry,
    pub function_lifter: FunctionLifter,
    pub next_reg: usize,
    pub label_counter: usize,
    pub pat_counter: usize,
    pub bit_width: u32,
}

impl IRBuilder {
    /// S-02: Side-effect free construction.
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            string_interner: StringInterner::new(),
            type_registry: TypeRegistry::new(),
            function_lifter: FunctionLifter::new(),
            next_reg: 1,
            label_counter: 0,
            pat_counter: 0,
            bit_width: 64,
        }
    }

    /// Generates a unique placeholder name for pattern matching.
    pub fn new_placeholder(&mut self) -> String {
        self.pat_counter += 1;
        format!("_pat_{}", self.pat_counter)
    }

    /// Delegates sanitization to TypeRegistry.
    pub fn sanitize_id(&self, id: &str) -> String {
        self.type_registry.sanitize_id(id)
    }

    /// Delegates escaping to StringInterner.
    pub fn escape_string(&self, s: &str) -> String {
        self.string_interner.escape_string(s)
    }

    /// Sets the default bit width for integer operations.
    pub fn set_bit_width(&mut self, width: u32) {
        self.bit_width = width;
    }

    pub fn new_reg(&mut self) -> String {
        let mut s = String::from("%");
        s.push_str(&self.next_reg.to_string());
        self.next_reg += 1;
        s
    }

    pub fn new_label(&mut self, prefix: &str) -> String {
        let mut s = String::from(prefix);
        s.push_str("_");
        s.push_str(&self.label_counter.to_string());
        self.label_counter += 1;
        s
    }

    fn resolve_global_name(&self, name: &str) -> Option<String> {
        self.type_registry.resolve_global_name(name, &self.function_lifter.function_definitions)
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

    /// Lowers a function definition to an LLVM IR function.
    fn lower_def(
        &mut self,
        name: &str,
        args: &[String],
        body: &Term,
        env: &HashMap<String, String>,
        captures: &[String],
    ) -> String {
        let mut inner_env = env.clone();
        let ty = format!("i{}", self.bit_width);

        // Captured variables come first as extra parameters
        for cap in captures {
            let sanitized = self.sanitize_id(cap).replace("\"", "");
            inner_env.insert(cap.clone(), format!("%{}", sanitized));
        }

        // Then the declared parameters
        for arg in args {
            let sanitized = self.sanitize_id(arg).replace("\"", "");
            inner_env.insert(arg.clone(), format!("%{}", sanitized));
        }

        // Save current state and clear for inner lowering
        let outer_instructions = std::mem::take(&mut self.instructions);
        let outer_next_reg = self.next_reg;
        self.next_reg = 1; // Reset for inner function
        
        let res_reg = self.lower_term(body, &inner_env);
        
        let body_instructions = std::mem::take(&mut self.instructions);
        self.instructions = outer_instructions;
        self.next_reg = outer_next_reg;

        self.function_lifter.lower_def(
            name,
            args,
            &res_reg,
            body_instructions,
            env,
            captures,
            self.bit_width,
            &self.sanitize_id(name),
        )
    }

    /// Lowers a high-level term into LLVM IR.
    pub fn lower_term(&mut self, term: &Term, env: &HashMap<String, String>) -> String {
        let ty = format!("i{}", self.bit_width);

        if let Term::App(_, _) = term {
            let (head, args) = self.collect_app_chain(term);
            if args.len() > 1 {
                if let Term::Var(name) = head {
                    if let Some(global_name) = self.resolve_global_name(name) {
                        // Lambda lifting: prepend captured variables as extra args
                        let mut all_args = Vec::new();
                        if let Some(captures) = self.function_lifter.lifted_captures.get(name).cloned() {
                            for cap in &captures {
                                let cap_val = env.get(cap).cloned().unwrap_or_else(|| format!("%{}", cap));
                                all_args.push(cap_val);
                            }
                        }
                        let lowered: Vec<String> = args.iter().map(|arg| self.lower_term(arg, env)).collect();
                        all_args.extend(lowered);

                        let res = self.new_reg();
                        let call_args = all_args
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
                    return format!("@{}", name);
                }
                if name == "print" {
                    return String::from("@print");
                }
                if let Some(layout) = self.type_registry.type_env.get(name) {
                    if layout.field_count == 0 {
                        return layout.tag.to_string();
                    } else {
                        return format!("@{}", self.sanitize_id(name));
                    }
                }
                if let Some(global_name) = self.resolve_global_name(name) {
                    return global_name;
                }
                env.get(name).cloned().unwrap_or_else(|| {
                    if self.function_lifter.function_definitions.contains_key(&self.sanitize_id(name)) {
                        let reg = self.new_reg();
                        let call_line = format!(
                            "  {} = call i64 @{}()\n",
                            reg,
                            self.sanitize_id(name)
                        );
                        self.instructions.push(call_line);
                        return reg;
                    }
                    String::from("0")
                })
            }
            Term::Integer(n) => n.to_string(),
            Term::Float(bits) => format!("0x{:x}", bits),
            Term::String(s) => {
                let label = self.string_interner.intern(s);
                let res = self.new_reg();
                self.instructions.push(format!("  {} = ptrtoint [{} x i8]* @{} to i64\n", 
                    res, s.len() + 1, label));
                res
            }
            Term::Char(c) => (*c as u32).to_string(),
            Term::Buffer(_) => String::from("0"), // ISO 8601: 2026-03-19 - Handled as null for now
            
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
                let fn_name = self.function_lifter.new_fn_name();
                let sanitized_fn_name = self.sanitize_id(&fn_name);
                
                let mut inner_env = env.clone();
                inner_env.insert(name.clone(), format!("%{}", name));
                
                let outer_instructions = std::mem::take(&mut self.instructions);
                let outer_next_reg = self.next_reg;
                self.next_reg = 1; // Reset for inner function

                let res_reg = self.lower_term(body, &inner_env);
                
                let body_instructions = std::mem::take(&mut self.instructions);
                self.instructions = outer_instructions;
                self.next_reg = outer_next_reg;

                let mut fn_def = format!("define {} @{}({} %{}) {{\n", ty, sanitized_fn_name, ty, name);
                for instr in body_instructions {
                    fn_def.push_str(&instr);
                }
                fn_def.push_str(&format!("  ret {} {}\n}}\n", ty, res_reg));
                self.function_lifter.function_definitions.insert(sanitized_fn_name.clone(), fn_def);
                
                let res = self.new_reg();
                self.instructions.push(format!("  {} = ptrtoint {} ({})* @{} to i64\n", 
                    res, ty, ty, sanitized_fn_name));
                res
            }
            
            Term::App(f, a) => {
                if let Term::Var(fname) = *f {
                    if let Some(global_name) = self.resolve_global_name(fname) {
                        let av = self.lower_term(a, env);
                        let mut all_args = Vec::new();
                        if let Some(captures) = self.function_lifter.lifted_captures.get(fname).cloned() {
                            for cap in &captures {
                                let cap_val = env.get(cap).cloned().unwrap_or_else(|| format!("%{}", cap));
                                all_args.push(cap_val);
                            }
                        }
                        all_args.push(av);
                        let res = self.new_reg();
                        let call_args = all_args
                            .into_iter()
                            .map(|arg| format!("{} {}", ty, arg))
                            .collect::<Vec<_>>()
                            .join(", ");
                        self.instructions.push(format!("  {} = call {} {}({})\n", res, ty, global_name, call_args));
                        return res;
                    }
                }

                let fv = self.lower_term(f, env);
                let av = self.lower_term(a, env);
                
                if fv.starts_with("@") {
                    let res = self.new_reg();
                    self.instructions.push(format!("  {} = call {} {}({} {})\n", res, ty, fv, ty, av));
                    return res;
                }

                let fn_ptr = self.new_reg();
                self.instructions.push(format!("  {} = inttoptr {} {} to {} ({})*\n", 
                    fn_ptr, ty, fv, ty, ty));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = call {} {}({} {})\n", res, ty, fn_ptr, ty, av));
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
                let target_val = self.lower_term(target, env);
                if branches.is_empty() { return String::from("0"); }

                let mut is_adt_match = false;
                for (pat, _, _) in branches {
                    if self.type_registry.type_env.contains_key(pat) {
                        is_adt_match = true;
                        break;
                    }
                }

                let mut val = target_val.clone();
                let mut struct_ptr = String::new();
                let generic_struct = "{ i64, [0 x i64] }";

                if is_adt_match {
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

                for (pat, _, _) in branches {
                    let label = self.new_label("case_branch");
                    branch_labels.push(label.clone());
                    if pat == "_" {
                        default_label = label;
                    } else {
                        let case_val = if is_adt_match {
                            self.type_registry.type_env.get(pat).map(|l| l.tag.to_string()).unwrap_or_else(|| pat.clone())
                        } else {
                            pat.clone()
                        };
                        cases.push((case_val, label));
                    }
                }

                let mut switch_instr = format!("  switch {} {}, label %{} [\n", ty, val, default_label);
                for (pat, label) in &cases {
                    switch_instr.push_str(&format!("    {} {}, label %{}\n", ty, pat, label));
                }
                switch_instr.push_str("  ]\n");
                self.instructions.push(switch_instr);

                let mut phi_entries = Vec::new();
                for (i, (pat, args, body)) in branches.iter().enumerate() {
                    let label = &branch_labels[i];
                    self.instructions.push(format!("{}:\n", label));
                    
                    let mut branch_env = env.clone();
                    if is_adt_match && pat != "_" {
                        if let Some(layout) = self.type_registry.type_env.get(pat).cloned() {
                            for (j, arg_name) in args.iter().enumerate() {
                                if j < layout.field_count {
                                    let i64_ptr = self.new_reg();
                                    self.instructions.push(format!("  {} = bitcast {}* {} to i64*\n", i64_ptr, generic_struct, struct_ptr));
                                    let field_ptr = self.new_reg();
                                    self.instructions.push(format!("  {} = getelementptr i64, i64* {}, i32 {}\n", field_ptr, i64_ptr, j + 1));
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
            Term::Bind(_, action) => self.lower_term(action, env),
            
            Term::Bits64Type => String::from("i64"),
            Term::I32Type => String::from("i32"),
            Term::I8Type => String::from("i8"),

            Term::Pi(_, _, _, _) | Term::IntegerType | Term::FloatType | Term::StringType | Term::CharType |
            Term::IOType | Term::TypeType |
            Term::Universe(_) => String::from("0"),

            Term::Def(name, args, body) => {
                self.lower_def(name, args, body, env, &[])
            }
            
            Term::Module(_) | Term::Import(_) | Term::Interface(_, _, _) |
            Term::Implementation(_, _, _) | Term::Record(_, _) | Term::Mutual(_) => "void".to_string(),
            
            Term::Data(_name, _params, constructors) => {
                for (i, con) in constructors.iter().enumerate() {
                    self.type_registry.register_constructor(con.name.clone(), ConstructorLayout {
                        tag: i as u32,
                        field_count: con.fields.len(),
                    });

                    if con.fields.len() > 0 {
                        let mut arg_str = String::new();
                        for j in 0..con.fields.len() {
                            if !arg_str.is_empty() { arg_str.push_str(", "); }
                            arg_str.push_str(&format!("{} %f{}", ty, j));
                        }
                        
                        let sanitized_name = self.sanitize_id(&con.name);
                        let mut con_def = format!("define {} @{}({}) {{\n", ty, sanitized_name, arg_str);
                        let struct_ty = format!("{{ i64, [{0} x i64] }}", con.fields.len());
                        let struct_size = 8 * (1 + con.fields.len());
                        
                        con_def.push_str(&format!("  %mem = call i8* @malloc(i64 {})\n", struct_size));
                        con_def.push_str(&format!("  %ptr = bitcast i8* %mem to {}*\n", struct_ty));
                        con_def.push_str(&format!("  %tag_ptr = getelementptr {}, {}* %ptr, i32 0, i32 0\n", struct_ty, struct_ty));
                        con_def.push_str(&format!("  store i64 {}, i64* %tag_ptr\n", i));
                        
                        for j in 0..con.fields.len() {
                            con_def.push_str(&format!("  %f{0}_ptr = getelementptr {1}, {1}* %ptr, i32 0, i32 1, i32 {0}\n", j, struct_ty));
                            con_def.push_str(&format!("  store i64 %f{0}, i64* %f{0}_ptr\n", j));
                        }
                        
                        con_def.push_str(&format!("  %res = ptrtoint {}* %ptr to i64\n", struct_ty));
                        con_def.push_str("  ret i64 %res\n}\n");
                        self.function_lifter.function_definitions.insert(sanitized_name, con_def);
                    }
                }
                "void".to_string()
            }
            
            Term::Where(body, defs) => {
                for def in defs {
                    if let Term::Def(name, args, dbody) = def {
                        let mut free_vars = Vec::new();
                        let mut bound = HashSet::new();
                        for a in args { bound.insert(a.as_str()); }
                        FunctionLifter::collect_free_vars(dbody, &bound, &mut free_vars);
                        
                        let unique_frees: HashSet<String> = free_vars.into_iter().collect();
                        let captures: Vec<String> = unique_frees.into_iter().collect();
                        
                        self.lower_def(name, args, dbody, env, &captures);
                    }
                }
                self.lower_term(body, env)
            }
            
            Term::Mutual(stmts) => {
                for s in stmts { self.lower_term(s, env); }
                "void".to_string()
            }

            Term::BufferLoad(b, i) => {
                let bv = self.lower_term(b, env);
                let iv = self.lower_term(i, env);
                let ptr = self.new_reg();
                self.instructions.push(format!("  {} = inttoptr {} {} to i8*\n", ptr, ty, bv));
                let gep = self.new_reg();
                self.instructions.push(format!("  {} = getelementptr i8, i8* {}, i64 {}\n", gep, ptr, iv));
                let val = self.new_reg();
                self.instructions.push(format!("  {} = load i8, i8* {}\n", val, gep));
                let res = self.new_reg();
                self.instructions.push(format!("  {} = zext i8 {} to {}\n", res, val, ty));
                res
            }

            Term::BufferStore(b, i, v) => {
                let bv = self.lower_term(b, env);
                let iv = self.lower_term(i, env);
                let vv = self.lower_term(v, env);
                let ptr = self.new_reg();
                self.instructions.push(format!("  {} = inttoptr {} {} to i8*\n", ptr, ty, bv));
                let gep = self.new_reg();
                self.instructions.push(format!("  {} = getelementptr i8, i8* {}, i64 {}\n", gep, ptr, iv));
                let val_i8 = self.new_reg();
                self.instructions.push(format!("  {} = trunc {} {} to i8\n", val_i8, ty, vv));
                self.instructions.push(format!("  store i8 {}, i8* {}\n", val_i8, gep));
                iv
            }
        }
    }
}
