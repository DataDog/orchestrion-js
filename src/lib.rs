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
    fn new(instrumentations: Vec<InstrumentationConfig>) -> Self {
        Self {
            instrumentations: instrumentations
                .into_iter()
                .map(Instrumentation::new)
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
            .filter(|instr| {
                instr.matches(module_name, version, file_path)
            })
            .collect()
    }
}

impl FromStr for Instrumentor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(InstrumentationConfig::from_yaml_data(s)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;
    use swc::{
        config::{IsModule, SourceMapsConfig},
        try_with_handler, Compiler, HandlerOpts, PrintArgs,
    };
    use swc_core::common::{comments::Comments, errors::ColorConfig, FileName, FilePathMapping};
    use swc_core::ecma::ast::EsVersion;
    use swc_ecma_parser::{EsSyntax, Syntax};
    use swc_ecma_visit::VisitMutWith;

    fn print_result(original: &str, modified: &str) {
        println!("\n - == === Original === == - \n{}\n\n", original);
        println!("\n - == === Modified === == - \n{}\n\n", modified);
    }

    fn transpile(
        contents: &str,
        is_module: IsModule,
        instrumentation: Vec<&mut Instrumentation>,
    ) -> String {
        let compiler = Compiler::new(Arc::new(swc_core::common::SourceMap::new(
            FilePathMapping::empty(),
        )));
        try_with_handler(
            compiler.cm.clone(),
            HandlerOpts {
                color: ColorConfig::Never,
                skip_filename: false,
            },
            |handler| {
                let source_file = compiler.cm.new_source_file(
                    Arc::new(FileName::Real(PathBuf::from("index.mjs"))),
                    contents.to_string(),
                );

                let program = compiler
                    .parse_js(
                        source_file.to_owned(),
                        handler,
                        EsVersion::latest(),
                        Syntax::Es(EsSyntax {
                            explicit_resource_management: true,
                            import_attributes: true,
                            decorators: true,
                            ..Default::default()
                        }),
                        is_module,
                        Some(&compiler.comments() as &dyn Comments),
                    )
                    .and_then(|mut program| {
                        for instr in instrumentation {
                            program.visit_mut_with(instr);
                        }
                        Ok(program)
                    })
                    .unwrap();
                let result = compiler
                    .print(
                        &program,
                        PrintArgs {
                            source_file_name: None,
                            source_map: SourceMapsConfig::Bool(false),
                            comments: None,
                            emit_source_map_columns: false,
                            ..Default::default()
                        },
                    )
                    .unwrap();
                Ok(result.code)
            },
        )
        .unwrap()
    }

    fn init_instrumentor(typ: &str) -> Instrumentor {
        let yaml = format!(r#"
version: 1
instrumentations:
  - module_name: undici
    version_range: ">=0.0.1"
    file_path: index.mjs
    function_query:
      name: fetch
      type: {}
      kind: async
      index: 0
    operator: traceAsync
    channel_name: fetch
"#, typ);
        yaml.parse().unwrap()
    }

//     #[test]
//     fn basic_mjs() {
//         let mut instrumentor = init_instrumentor("decl");
//         let contents = "export async function fetch(url) { return 42; }";
// 
//         let instrumentations = instrumentor.get_matching_instrumentations(
//             "undici",
//             "0.0.1",
//             &PathBuf::from("index.mjs"),
//         );
// 
//         let result = transpile(contents, IsModule::Bool(true), instrumentations);
//         print_result(contents, &result);
//     }
// 
//     #[test]
//     fn basic_cjs() {
//         let mut instrumentor = init_instrumentor("decl");
//         let contents = "async function fetch(url) { return 42; }\nmodule.exports = { fetch };";
// 
//         let instrumentations = instrumentor.get_matching_instrumentations(
//             "undici",
//             "0.0.1",
//             &PathBuf::from("index.mjs"),
//         );
// 
//         let result = transpile(contents, IsModule::Bool(true), instrumentations);
//         print_result(contents, &result);
//     }
// 
//     #[test]
//     fn expr_mjs() {
//         let mut instrumentor = init_instrumentor("expr");
//         let contents = "const fetch = async function (url) { return 42; }; export { fetch };";
// 
//         let instrumentations = instrumentor.get_matching_instrumentations(
//             "undici",
//             "0.0.1",
//             &PathBuf::from("index.mjs"),
//         );
// 
//         let result = transpile(contents, IsModule::Bool(true), instrumentations);
//         print_result(contents, &result);
//     }

    #[test]
    fn expr_cjs() {
        let mut instrumentor = init_instrumentor("expr");
        let contents = "exports.fetch = async function (url) { return 42; };";

        let instrumentations = instrumentor.get_matching_instrumentations(
            "undici",
            "0.0.1",
            &PathBuf::from("index.mjs"),
        );

        let result = transpile(contents, IsModule::Bool(true), instrumentations);
        print_result(contents, &result);
    }
}
