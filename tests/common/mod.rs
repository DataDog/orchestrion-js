use assert_cmd::prelude::*;
use orchestrion_js::*;
use std::include_str;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use swc::{
    config::{IsModule, SourceMapsConfig},
    try_with_handler, Compiler, HandlerOpts, PrintArgs,
};
use swc_core::common::{comments::Comments, errors::ColorConfig, FileName, FilePathMapping};
use swc_core::ecma::ast::EsVersion;
use swc_ecma_parser::{EsSyntax, Syntax};
use swc_ecma_visit::VisitMutWith;
use tempfile;

fn print_result(original: &str, modified: &str) {
    println!(
        "\n - == === Original === == - \n{}\n\n\n - == === Modified === == - \n{}\n\n",
        original, modified
    );
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

pub fn init_instrumentor(typ: &str) -> Instrumentor {
    let yaml = format!(include_str!("./instrumentations.yml"), typ);
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
        .assert()
        .success();
}

fn get_test_code(channel: &str, test_code: &str, mjs: bool) -> String {
    let imports = if mjs {
        "import { tracingChannel } from 'diagnostics_channel'; import assert from 'assert';"
    } else {
        "const { tracingChannel } = require('diagnostics_channel'); const assert = require('assert');"
    };
    let preamble = include_str!("./preamble.js");
    format!(
        r#"
{}
const channel = tracingChannel('{}');
{}
// Test code
{}
"#,
        imports, channel, preamble, test_code
    )
}

pub fn transpile_and_test(
    channel: &str,
    mjs: bool,
    instrumentations: Vec<&mut Instrumentation>,
    contents: &str,
    test_code: &str,
) {
    let result = transpile(contents, IsModule::Bool(mjs), instrumentations);
    let all_test_code = get_test_code(channel, test_code, mjs);
    run_with_node(&all_test_code, &result, mjs);
}
