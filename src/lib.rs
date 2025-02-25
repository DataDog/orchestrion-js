//! # Orchestrion
//! Orchestrion is a library for instrumenting Node.js libraries at build or load time.
//! It provides [`VisitMut`] implementations for SWC's AST nodes, which can be used to insert
//! tracing code into matching functions. It's entirely configurable via a YAML string, and can be
//! used in SWC plugins, or anything else that mutates JavaScript ASTs using SWC.
//!
//! [`VisitMut`]: https://rustdoc.swc.rs/swc_core/ecma/visit/trait.VisitMut.html

use std::path::PathBuf;
use std::str::FromStr;

mod config;
use config::*;

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
}

impl Instrumentor {
    fn new(instrumentations: Vec<InstrumentationConfig>, dc_module: String) -> Self {
        Self {
            instrumentations: instrumentations
                .into_iter()
                .map(|inst| Instrumentation::new(inst, dc_module.clone()))
                .collect(),
        }
    }

    /// For a given module name, version, and file path within the module, return all
    /// `Instrumentation` instances that match.
    pub fn get_matching_instrumentations(
        &mut self,
        module_name: &str,
        version: &str,
        file_path: &PathBuf,
    ) -> Vec<&mut Instrumentation> {
        self.instrumentations
            .iter_mut()
            .filter(|instr| instr.matches(module_name, version, file_path))
            .collect()
    }
}

impl FromStr for Instrumentor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Config::from_yaml_data(s).map(|c| Self::new(c.instrumentations, c.dc_module))
    }
}
