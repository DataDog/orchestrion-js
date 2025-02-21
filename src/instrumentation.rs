use std::path::PathBuf;
use crate::config::InstrumentationConfig;
use swc_core::common::{Span, SyntaxContext};
use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
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

    fn new_fn(&self, body: Option<BlockStmt>) -> ArrowExpr {
        ArrowExpr {
            params: vec![],
            body: Box::new(body.unwrap().into()),
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

    fn insert_tracing(&self, body: &mut Option<BlockStmt>) {
        let ch_ident = ident!(format!("tr_ch_apm${}", self.config.channel_name));
        let trace_ident = ident!(format!(
            "tr_ch_apm${}.{}",
            self.config.channel_name.clone(),
            self.config.operator.clone().as_str()
        ));
        let traced_fn = self.new_fn(body.clone());
        body.as_mut().unwrap().stmts = vec![
            quote!("const traced = $traced;" as Stmt, traced: Expr = traced_fn.into()),
            quote!(
                "if (!$ch.hasSubscribers) return traced();" as Stmt,
                ch = ch_ident
            ),
            quote!(
                "return $trace(traced, { arguments} );" as Stmt,
                trace = trace_ident
            ),
        ];
    }

    fn trace_expr_or_count(&mut self, func_expr: &mut FnExpr, name: &Atom) {
        if self.config.function_query.matches_expr(func_expr, self.count, &name.to_string()) {
            self.insert_tracing(&mut func_expr.function.body);
        } else {
            self.count += 1;
        }

    }

    pub fn matches(&self, module_name: &str, version: &str, file_path: &PathBuf) -> bool {
        self.config.matches(module_name, version, file_path)
    }
}

impl VisitMut for Instrumentation {
    fn visit_mut_module(&mut self, node: &mut Module) {
        let import = ImportDecl {
            span: Span::default(),
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                is_type_only: false,
                span: Span::default(),
                local: ident!("tr_ch_apm_tracingChannel"),
                imported: Some(ModuleExportName::Ident(ident!("tracingChannel"))),
            })],
            src: Box::new(Str::from("diagnostics_channel")),
            type_only: false,
            with: None,
            phase: Default::default(),
        };
        node.body.insert(0, ModuleItem::ModuleDecl(import.into()));
        node.body
            .insert(1, ModuleItem::Stmt(self.create_tracing_channel()));
        node.visit_mut_children_with(self);
    }

    fn visit_mut_script(&mut self, node: &mut Script) {
        node.body.insert(0, quote!(
            "const { tracingChannel: tr_ch_apm_tracingChannel } = require('diagnostics_channel');" as Stmt,
        ));
        node.body.insert(1, self.create_tracing_channel());
        node.visit_mut_children_with(self);
    }

    fn visit_mut_fn_decl(&mut self, node: &mut FnDecl) {
        if self.config.function_query.matches_decl(node, self.count) {
            self.insert_tracing(&mut node.function.body);
        } else {
            self.count += 1;
        }
    }

    fn visit_mut_var_decl(&mut self, node: &mut VarDecl) {
        for decl in &mut node.decls {
            if let Some(init) = &mut decl.init {
                if let Some(func_expr) = init.as_mut_fn_expr() {
                    if let Pat::Ident(name) = &decl.name {
                        self.trace_expr_or_count(func_expr, &name.id.sym);
                    }
                }
            }
        }
    }

    fn visit_mut_assign_expr(&mut self, node: &mut AssignExpr) {
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
        if let Some(func_expr) = node.right.as_mut_fn_expr() {
            if let AssignTarget::Simple(node) = &mut node.left {
                match &node {
                    SimpleAssignTarget::Ident(name) => {
                        self.trace_expr_or_count(func_expr, &name.id.sym);
                    },
                    SimpleAssignTarget::Member(member) => {
                        match &member.prop {
                            MemberProp::Ident(ident) => {
                                self.trace_expr_or_count(func_expr, &ident.sym);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    // TODO(bengl) Support class methods
}
