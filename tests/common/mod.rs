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

pub fn init_instrumentor() -> Instrumentor {
    let yaml = include_str!("./instrumentations.yml");
    yaml.parse().unwrap()
}

fn get_dir(test_name: &str) -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("tests");
    dir.push(test_name);
    dir
}

pub fn transpile_and_test(test_name: &str, mjs: bool, instrumentations: Vec<&mut Instrumentation>) {
    let dir = get_dir(test_name);
    let extension = if mjs { "mjs" } else { "js" };
    let instrumentable = dir.join(format!("mod.{}", extension));
    let mut file = std::fs::File::open(&instrumentable).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let result = transpile(&contents, IsModule::Bool(mjs), instrumentations);

    let instrumented_file = dir.join(format!("instrumented.{}", extension));
    let mut file = std::fs::File::create(&instrumented_file).unwrap();
    file.write_all(result.as_bytes()).unwrap();

    let test_file = dir.join(format!("test.{}", extension));
    Command::new("node")
        .current_dir(dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .arg(&test_file)
        .assert()
        .success();
}
