/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
use assert_cmd::prelude::*;
use nodejs_semver::Version;
use orchestrion_js::*;
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

fn print_result(original: &str, modified: &str) {
    println!(
        "\n - == === Original === == - \n{}\n\n\n - == === Modified === == - \n{}\n\n",
        original, modified
    );
}

fn transpile(
    contents: &str,
    is_module: IsModule,
    instrumentation: &mut InstrumentationVisitor,
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
                .map(|mut program| {
                    program.visit_mut_with(instrumentation);
                    program
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

static TEST_MODULE_NAME: &str = "undici";
static TEST_MODULE_PATH: &str = "index.mjs";

pub fn transpile_and_test(test_file: &str, mjs: bool, config: Config) {
    let test_file = PathBuf::from(test_file);
    let test_dir = test_file.parent().expect("Couldn't find test directory");

    let file_path = PathBuf::from("index.mjs");
    let mut instrumentor = Instrumentor::new(config);
    let dep = Dependency {
        name: TEST_MODULE_NAME.to_string(),
        version: Version::parse("0.0.1").unwrap(),
        relative_path: file_path,
    };
    let abs = PathBuf::from("");
    let mut instrumentations = instrumentor.get_matching_instrumentations(&abs, Some(&dep));

    let extension = if mjs { "mjs" } else { "js" };
    let instrumentable = test_dir.join(format!("mod.{}", extension));
    let mut file = std::fs::File::open(&instrumentable).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let result = transpile(&contents, IsModule::Bool(mjs), &mut instrumentations);

    let instrumented_file = test_dir.join(format!("instrumented.{}", extension));
    let mut file = std::fs::File::create(&instrumented_file).unwrap();
    file.write_all(result.as_bytes()).unwrap();

    let test_file = format!("test.{}", extension);
    Command::new("node")
        .current_dir(test_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .arg(&test_file)
        .assert()
        .success();
}

pub fn test_module_matcher() -> CodeMatcher {
    CodeMatcher::dependency(TEST_MODULE_NAME, ">=0.0.1", TEST_MODULE_PATH).unwrap()
}
