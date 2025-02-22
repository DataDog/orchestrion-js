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
    use std::process::Command;
    use std::io::prelude::*;
    use assert_cmd::prelude::*;
    use tempfile;

    fn print_result(original: &str, modified: &str) {
        println!("\n - == === Original === == - \n{}\n\n\n - == === Modified === == - \n{}\n\n", original, modified);
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

                print_result(contents, &result.code);
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
    operator: tracePromise
    channel_name: fetch
  - module_name: undici
    version_range: ">=0.0.1"
    file_path: index.mjs
    function_query:
      name: fetch
      type: method
      kind: async
      index: 0
    operator: tracePromise
    channel_name: Undici_fetch
  - module_name: "@langchain/core"
    version_range: ">=0.1.0"
    file_path: dist/runnables/base.js
    function_query:
      name: batch
      type: method
      kind: async
      index: 0
    operator: tracePromise
    channel_name: runnablesequence_batch
"#, typ);
        yaml.parse().unwrap()
    }

    fn run_with_node(test_code: &str, instrumented_code: &str, mjs: bool) {
        let temp_dir = tempfile::tempdir().unwrap();
        let extension = if mjs { "mjs" } else { "js" };
        let test_file = temp_dir.path().join(format!("test_file.{}", extension));
        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(test_code.as_bytes()).unwrap();

        let instrumented_file = temp_dir.path().join(format!("instrumented.{}", extension));
        let mut file = std::fs::File::create(&instrumented_file).unwrap();
        file.write_all(instrumented_code.as_bytes()).unwrap();

        Command::new("node")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .arg(&test_file)
            .assert().success();
    }

    fn get_test_code(channel: &str, test_code: &str, mjs: bool) -> String {
        let imports = if mjs {
            "import { tracingChannel } from 'diagnostics_channel'; import assert from 'assert';"
        } else {
            "const { tracingChannel } = require('diagnostics_channel'); const assert = require('assert');"
        };
        format!(r#"
{}
const channel = tracingChannel('{}');
const context = {{}};
channel.subscribe({{
  start(message) {{
    message.context = context;
    context.start = true;
  }},
  end(message) {{
    message.context.end = true;
    // Handle end message
  }},
  asyncStart(message) {{
    message.context.asyncStart = message.result
    // Handle asyncStart message
  }},
  asyncEnd(message) {{
    message.context.asyncEnd = message.result;
  }}
}});
// Test code
{}
"#, imports, channel, test_code)
    }

    fn transpile_and_test(channel: &str, mjs: bool, instrumentations: Vec<&mut Instrumentation>, contents: &str, test_code: &str) {
        let result = transpile(contents, IsModule::Bool(mjs), instrumentations);
        let all_test_code = get_test_code(channel, test_code, mjs);
        run_with_node(&all_test_code, &result, mjs);
    }

    #[test]
    fn decl_mjs() {
        let mut instrumentor = init_instrumentor("decl");
        let instrumentations = instrumentor.get_matching_instrumentations(
            "undici",
            "0.0.1",
            &PathBuf::from("index.mjs"),
        );
 
        let contents = "export async function fetch(url) { return 42; }";
        let test_code = r#"
import { fetch } from './instrumented.mjs';
const result = await fetch('https://example.com');
assert.strictEqual(result, 42);
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 42,
  asyncEnd: 42
});
        "#;
        transpile_and_test("orchestrion:undici:fetch", true, instrumentations, contents, test_code);
    }
 
    #[test]
    fn decl_cjs() {
        let mut instrumentor = init_instrumentor("decl");
        let instrumentations = instrumentor.get_matching_instrumentations(
            "undici",
            "0.0.1",
            &PathBuf::from("index.mjs"),
        );
 
        let contents = "async function fetch(url) { return 42; }\nmodule.exports = { fetch };";
        let test_code = r#"
const { fetch } = require('./instrumented.js');
(async () => {
  const result = await fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
})();
        "#;
        transpile_and_test("orchestrion:undici:fetch", false, instrumentations, contents, test_code);
    }
 
    #[test]
    fn expr_mjs() {
        let mut instrumentor = init_instrumentor("expr");
 
        let instrumentations = instrumentor.get_matching_instrumentations(
            "undici",
            "0.0.1",
            &PathBuf::from("index.mjs"),
        );
 
        let contents = "const fetch = async function (url) { return 42; }; export { fetch };";
        let test_code = r#"
import { fetch } from './instrumented.mjs';
const result = await fetch('https://example.com');
assert.strictEqual(result, 42);
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 42,
  asyncEnd: 42
});
        "#;
        transpile_and_test("orchestrion:undici:fetch", true, instrumentations, contents, test_code);
    }

    #[test]
    fn expr_cjs() {
        let mut instrumentor = init_instrumentor("expr");

        let instrumentations = instrumentor.get_matching_instrumentations(
            "undici",
            "0.0.1",
            &PathBuf::from("index.mjs"),
        );

        let contents = "exports.fetch = async function (url) { return 42; };";
        let test_code = r#"
const { fetch } = require('./instrumented.js');
(async () => {
  const result = await fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
})();
        "#;
        transpile_and_test("orchestrion:undici:fetch", false, instrumentations, contents, test_code);
    }

    #[test]
    fn class_method_cjs() {
        let mut instrumentor = init_instrumentor("expr");

        let instrumentations = instrumentor.get_matching_instrumentations(
            "undici",
            "0.0.1",
            &PathBuf::from("index.mjs"),
        );

        let contents = r#"
class Undici {
    async fetch (lmao) {
        return 42;
    }
}

module.exports = Undici;
"#;
        let test_code = r#"
const Undici = require('./instrumented.js');
(async () => {
  const undici = new Undici;
  const result = await undici.fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
})();
        "#;
        transpile_and_test("orchestrion:undici:Undici_fetch", false, instrumentations, contents, test_code);
    }
}
