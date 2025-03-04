use crate::config::InstrumentationConfig;
use std::path::PathBuf;
use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::{
    ast::{
        ArrowExpr, AssignExpr, AssignTarget, BlockStmt, ClassMethod, Expr, FnDecl, FnExpr, Ident,
        Lit, MemberProp, MethodProp, Module, ModuleItem, Pat, PropName, Script, SimpleAssignTarget,
        Stmt, Str, VarDecl,
    },
    atoms::Atom,
};
use swc_core::quote;

macro_rules! ident {
    ($name:expr) => {
        Ident::new($name.into(), Span::default(), SyntaxContext::empty())
    };
}

/// An [`Instrumentation`] instance represents a single instrumentation configuration, and implements
/// SWC's [`VisitMut`] trait to insert tracing code into matching functions. You can use this
/// wherever you would use a [`VisitMut`] instance, such as within an SWC plugin, for example.
///
/// [`Instrumentation`]: Instrumentation
/// [`VisitMut`]: https://rustdoc.swc.rs/swc_core/ecma/visit/trait.VisitMut.html
pub struct Instrumentation {
    config: InstrumentationConfig,
    count: usize,
}

impl Instrumentation {
    pub(crate) fn new(config: InstrumentationConfig) -> Self {
        Self { config, count: 0 }
    }

    fn new_fn(&self, body: BlockStmt) -> ArrowExpr {
        ArrowExpr {
            params: vec![],
            body: Box::new(body.into()),
            is_async: self.config.function_query.kind.is_async(),
            is_generator: self.config.function_query.kind.is_generator(),
            type_params: None,
            return_type: None,
            span: Span::default(),
            ctxt: SyntaxContext::empty(),
        }
    }

    fn create_tracing_channel(&self) -> Stmt {
        let ch_str = ident!(format!("tr_ch_apm${}", self.config.channel_name));
        let channel_string = Expr::Lit(Lit::Str(Str {
            span: Span::default(),
            value: format!(
                "orchestrion:{}:{}",
                self.config.module_name, self.config.channel_name
            )
            .into(),
            raw: None,
        }));
        let define_channel = quote!(
            "const $ch = tr_ch_apm_tracingChannel($channel_str);" as Stmt,
            ch = ch_str,
            channel_str: Expr = channel_string,
        );
        define_channel
    }

    fn insert_tracing(&self, body: &mut BlockStmt) {
        let ch_ident = ident!(format!("tr_ch_apm${}", self.config.channel_name));
        let trace_ident = ident!(format!(
            "tr_ch_apm${}.{}",
            self.config.channel_name,
            self.config.operator.as_str()
        ));
        let traced_fn = self.new_fn(body.clone());
        body.stmts = vec![
            quote!("const traced = $traced;" as Stmt, traced: Expr = traced_fn.into()),
            quote!(
                "if (!$ch.hasSubscribers) return traced();" as Stmt,
                ch = ch_ident
            ),
            quote!(
                "return $trace(traced, { arguments, self: this } );" as Stmt,
                trace = trace_ident
            ),
        ];
    }

    fn trace_expr_or_count(&mut self, func_expr: &mut FnExpr, name: &Atom) -> bool {
        if self
            .config
            .function_query
            .matches_expr(func_expr, self.count, name.as_ref())
            && func_expr.function.body.is_some()
        {
            if let Some(body) = func_expr.function.body.as_mut() {
                self.insert_tracing(body)
            }
            true
        } else {
            self.count += 1;
            false
        }
    }

    #[must_use]
    pub fn matches(&self, module_name: &str, version: &str, file_path: &PathBuf) -> bool {
        self.config.matches(module_name, version, file_path)
    }

    // The rest of these functions are from `VisitMut`, except they return a boolean to indicate
    // whether recusrsing through the tree is necessary, rather than calling
    // `visit_mut_children_with`.

    pub fn visit_mut_module(&mut self, node: &mut Module) -> bool {
        node.body
            .insert(1, ModuleItem::Stmt(self.create_tracing_channel()));
        true
    }

    pub fn visit_mut_script(&mut self, node: &mut Script) -> bool {
        let start_index = get_script_start_index(node);
        node.body
            .insert(start_index + 1, self.create_tracing_channel());
        true
    }

    pub fn visit_mut_fn_decl(&mut self, node: &mut FnDecl) -> bool {
        if self.config.function_query.matches_decl(node, self.count) && node.function.body.is_some()
        {
            if let Some(body) = node.function.body.as_mut() {
                self.insert_tracing(body)
            }
        } else {
            self.count += 1;
        }
        false
    }

    pub fn visit_mut_var_decl(&mut self, node: &mut VarDecl) -> bool {
        let mut traced = false;
        for decl in &mut node.decls {
            if let Some(init) = &mut decl.init {
                if let Some(func_expr) = init.as_mut_fn_expr() {
                    if let Pat::Ident(name) = &decl.name {
                        traced = self.trace_expr_or_count(func_expr, &name.id.sym);
                    }
                }
            }
        }
        !traced
    }

    pub fn visit_mut_class_method(&mut self, node: &mut ClassMethod) -> bool {
        let name = match &node.key {
            PropName::Ident(ident) => ident.sym.clone(),
            _ => return false,
        };
        if self
            .config
            .function_query
            .matches_class_method(node, self.count, name.as_ref())
            && node.function.body.is_some()
        {
            if let Some(body) = node.function.body.as_mut() {
                self.insert_tracing(body)
            }
        } else {
            self.count += 1;
        }
        false
    }

    pub fn visit_mut_method_prop(&mut self, node: &mut MethodProp) -> bool {
        let name = match &node.key {
            PropName::Ident(ident) => ident.sym.clone(),
            _ => return false,
        };
        if self
            .config
            .function_query
            .matches_method_prop(node, self.count, name.as_ref())
            && node.function.body.is_some()
        {
            if let Some(body) = node.function.body.as_mut() {
                self.insert_tracing(body)
            }
        } else {
            self.count += 1;
        }
        false
    }

    pub fn visit_mut_assign_expr(&mut self, node: &mut AssignExpr) -> bool {
        // TODO(bengl) This is by far the hardest bit. We're trying to infer a name for this
        // function expresion using the surrounding code, but it's not always possible, and even
        // where it is, there are so many ways to give a function expression a "name", that the
        // code paths here can get pretty hairy. Right now this is only covering some basic cases.
        // The following cases are missing:
        // - Destructuring assignment
        // - Assignment to private fields
        // - Doing anything with `super`
        // What's covered is:
        // - Simple assignment to an already-declared variable
        // - Simple assignment to a property of an object
        let mut traced = false;
        if let Some(func_expr) = node.right.as_mut_fn_expr() {
            if let AssignTarget::Simple(node) = &mut node.left {
                match &node {
                    SimpleAssignTarget::Ident(name) => {
                        traced = self.trace_expr_or_count(func_expr, &name.id.sym);
                    }
                    SimpleAssignTarget::Member(member) => {
                        if let MemberProp::Ident(ident) = &member.prop {
                            traced = self.trace_expr_or_count(func_expr, &ident.sym);
                        }
                    }
                    _ => {}
                }
            }
        }
        !traced
    }
}

/// If the script starts with a "use strict" directive, we need to skip it when inserting there
#[must_use]
pub fn get_script_start_index(script: &Script) -> usize {
    if let Some(Stmt::Expr(expr)) = script.body.first() {
        if let Some(Lit::Str(str_lit)) = expr.expr.as_lit() {
            if str_lit.value == "use strict" {
                return 1;
            }
        }
    }
    0
}
