//! # Orchestrion
//! Orchestrion is a library for instrumenting Node.js libraries at build or load time.
//! It provides [`VisitMut`] implementations for SWC's AST nodes, which can be used to insert
//! tracing code into matching functions. It's entirely configurable via a YAML string, and can be
//! used in SWC plugins, or anything else that mutates JavaScript ASTs using SWC.
//!
//! [`VisitMut`]: https://rustdoc.swc.rs/swc_core/ecma/visit/trait.VisitMut.html

use std::path::PathBuf;
use std::str::FromStr;

use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
};
use swc_core::common::{Span, SyntaxContext};
use swc_core::quote;

mod config;
use config::*;

mod instrumentation;
pub use instrumentation::*;

mod function_query;

macro_rules! ident {
    ($name:expr) => {
        Ident::new($name.into(), Span::default(), SyntaxContext::empty())
    };
}

/// This struct is responsible for managing all instrumentations. It's created from a YAML string
/// via the [`FromStr`] trait. See tests for examples, but by-and-large this just means you can
/// call `.parse()` on a YAML string to get an `Instrumentor` instance, if it's valid.
///
/// [`FromStr`]: https://doc.rust-lang.org/std/str/trait.FromStr.html
pub struct Instrumentor {
    instrumentations: Vec<Instrumentation>,
    dc_module: String,
}

impl Instrumentor {
    fn new(config: Config) -> Self {
        Self {
            instrumentations: config.instrumentations
                .into_iter()
                .map(Instrumentation::new)
                .collect(),
            dc_module: config.dc_module,
        }
    }

    /// For a given module name, version, and file path within the module, return all
    /// `Instrumentation` instances that match.
    pub fn get_matching_instrumentations(
        &mut self,
        module_name: &str,
        version: &str,
        file_path: &PathBuf,
    ) -> InstrumentationVisitor {
        let instrumentations = self.instrumentations
            .iter_mut()
            .filter(|instr| instr.matches(module_name, version, file_path))
            .collect();
        InstrumentationVisitor::new(instrumentations, self.dc_module.clone())
    }
}

impl FromStr for Instrumentor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Config::from_yaml_data(s).map(Self::new)
    }
}

pub struct InstrumentationVisitor<'a> {
    instrumentations: Vec<&'a mut Instrumentation>,
    dc_module: String,
}

impl<'a> InstrumentationVisitor<'a> {
    fn new(instrumentations: Vec<&'a mut Instrumentation>, dc_module: String) -> Self {
        Self { instrumentations, dc_module }
    }
}

macro_rules! visit_with_all {
    ($self:expr, $method:ident, $item:expr) => {
        let mut recurse = false;
        for instr in &mut $self.instrumentations {
            recurse = recurse || instr.$method($item);
        }
        if recurse {
            $item.visit_mut_children_with($self);
        }
    };
}

impl VisitMut for InstrumentationVisitor<'_> {
    fn visit_mut_module(&mut self, item: &mut Module) {
        let import = ImportDecl {
            span: Span::default(),
            specifiers: vec![ImportSpecifier::Named(ImportNamedSpecifier {
                is_type_only: false,
                span: Span::default(),
                local: ident!("tr_ch_apm_tracingChannel"),
                imported: Some(ModuleExportName::Ident(ident!("tracingChannel"))),
            })],
            src: Box::new(Str::from(self.dc_module.clone())),
            type_only: false,
            with: None,
            phase: Default::default(),
        };
        item.body.insert(0, ModuleItem::ModuleDecl(import.into()));
        visit_with_all!(self, visit_mut_module, item);
    }

    fn visit_mut_script(&mut self, item: &mut Script) {
        item.body.insert(
            get_script_start_index(item),
            quote!(
                "const { tracingChannel: tr_ch_apm_tracingChannel } = require($dc);" as Stmt,
                dc: Expr = self.dc_module.clone().into(),
            ),
        );
        visit_with_all!(self, visit_mut_script, item);
    }

    fn visit_mut_fn_decl(&mut self, item: &mut FnDecl) {
        visit_with_all!(self, visit_mut_fn_decl, item);
    }

    fn visit_mut_var_decl(&mut self, item: &mut VarDecl) {
        let mut recurse = false;
        for instr in &mut self.instrumentations {
            recurse = recurse || instr.visit_mut_var_decl(item);
        }
        if recurse {
            item.visit_mut_children_with(self);
        }
    }

    fn visit_mut_class_method(&mut self, item: &mut ClassMethod) {
        let mut recurse = false;
        for instr in &mut self.instrumentations {
            recurse = recurse || instr.visit_mut_class_method(item);
        }
        if recurse {
            item.visit_mut_children_with(self);
        }
    }

    fn visit_mut_method_prop(&mut self, item: &mut MethodProp) {
        let mut recurse = false;
        for instr in &mut self.instrumentations {
            recurse = recurse || instr.visit_mut_method_prop(item);
        }
        if recurse {
            item.visit_mut_children_with(self);
        }
    }

    fn visit_mut_assign_expr(&mut self, item: &mut AssignExpr) {
        let mut recurse = false;
        for instr in &mut self.instrumentations {
            recurse = recurse || instr.visit_mut_assign_expr(item);
        }
        if recurse {
            item.visit_mut_children_with(self);
        }
    }
}
