//! # Orchestrion
//! Orchestrion is a library for instrumenting Node.js libraries at build or load time.
//! It provides [`VisitMut`] implementations for SWC's AST nodes, which can be used to insert
//! tracing code into matching functions. It's entirely configurable via a YAML string, and can be
//! used in SWC plugins, or anything else that mutates JavaScript ASTs using SWC.
//!
//! [`VisitMut`]: https://rustdoc.swc.rs/swc_core/ecma/visit/trait.VisitMut.html

mod macros;
use std::path::PathBuf;
use std::str::FromStr;

use swc_core::ecma::{
    ast::{
        AssignExpr, ClassDecl, ClassMethod, Constructor, FnDecl, MethodProp, Module, Script, Str,
        VarDecl,
    },
    visit::{VisitMut, VisitMutWith},
};
use swc_core::quote;

mod error;
use error::OrchestrionError;

mod config;
use config::Config;

mod instrumentation;
pub use instrumentation::*;

mod function_query;

/// This struct is responsible for managing all instrumentations. It's created from a YAML string
/// via the [`FromStr`] trait. See tests for examples, but by-and-large this just means you can
/// call `.parse()` on a YAML string to get an `Instrumentor` instance, if it's valid.
///
/// [`FromStr`]: https://doc.rust-lang.org/std/str/trait.FromStr.html
pub struct Instrumentor {
    instrumentations: Vec<Instrumentation>,
    diagnostic_channel_module: String,
}

impl Instrumentor {
    fn new(config: Config) -> Self {
        Self {
            instrumentations: config
                .instrumentations
                .into_iter()
                .map(Instrumentation::new)
                .collect(),
            diagnostic_channel_module: config.diagnostic_channel_module,
        }
    }

    /// For a given module name, version, and file path within the module, return all
    /// `Instrumentation` instances that match.
    pub fn get_matching_instrumentations<'a>(
        &'a mut self,
        module_name: &'a str,
        version: &'a str,
        file_path: &'a PathBuf,
    ) -> InstrumentationVisitor<'a> {
        let instrumentations = self
            .instrumentations
            .iter_mut()
            .filter(|instr| instr.matches(module_name, version, file_path));
        InstrumentationVisitor::new(instrumentations, self.diagnostic_channel_module.as_ref())
    }
}

impl FromStr for Instrumentor {
    type Err = OrchestrionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Config::from_yaml_data(s).map(Self::new)
    }
}

pub struct InstrumentationVisitor<'a> {
    instrumentations: Vec<&'a mut Instrumentation>,
    dc_module: &'a str,
}

impl<'a> InstrumentationVisitor<'a> {
    fn new<I>(instrumentations: I, dc_module: &'a str) -> Self
    where
        I: Iterator<Item = &'a mut Instrumentation> + 'a,
    {
        Self {
            instrumentations: instrumentations.collect(),
            dc_module,
        }
    }
}

macro_rules! visit_with_all {
    ($self:expr, $method:ident, $item:expr) => {
        let mut recurse = false;
        for instr in &mut $self.instrumentations {
            let needs_recurse = instr.$method($item);
            recurse = recurse || needs_recurse;
        }
        if recurse {
            $item.visit_mut_children_with($self);
        }
    };
}

macro_rules! visit_with_all_fn {
    ($method:ident, $item_struct:ty) => {
        fn $method(&mut self, item: &mut $item_struct) {
            visit_with_all!(self, $method, item);
        }
    };
}

impl VisitMut for InstrumentationVisitor<'_> {
    fn visit_mut_module(&mut self, item: &mut Module) {
        let mut line = quote!(
            "import { tracingChannel as tr_ch_apm_tracingChannel } from 'dc';" as ModuleItem,
        );
        if let Some(module_decl) = line.as_mut_module_decl() {
            if let Some(import) = module_decl.as_mut_import() {
                import.src = Box::new(Str::from(self.dc_module));
                item.body.insert(0, line);
            }
        }
        visit_with_all!(self, visit_mut_module, item);
        for instr in &mut self.instrumentations {
            instr.reset();
        }
    }

    fn visit_mut_script(&mut self, item: &mut Script) {
        let import = quote!(
            "const { tracingChannel: tr_ch_apm_tracingChannel } = require($dc);" as Stmt,
            dc: Expr = self.dc_module.into(),
        );
        item.body.insert(get_script_start_index(item), import);
        visit_with_all!(self, visit_mut_script, item);
        for instr in &mut self.instrumentations {
            instr.reset();
        }
    }

    visit_with_all_fn!(visit_mut_fn_decl, FnDecl);
    visit_with_all_fn!(visit_mut_var_decl, VarDecl);
    visit_with_all_fn!(visit_mut_method_prop, MethodProp);
    visit_with_all_fn!(visit_mut_assign_expr, AssignExpr);
    visit_with_all_fn!(visit_mut_class_decl, ClassDecl);
    visit_with_all_fn!(visit_mut_class_method, ClassMethod);
    visit_with_all_fn!(visit_mut_constructor, Constructor);
}
