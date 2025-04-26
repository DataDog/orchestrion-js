use crate::common::*;
use nodejs_semver::Range;
use orchestrion_js::*;
use std::path::PathBuf;

#[test]
fn decl_mjs() {
    transpile_and_test(
        file!(),
        true,
        Config::new_single_with_default_dc_module(InstrumentationConfig {
            module_name: "undici".to_string(),
            version_range: Range::parse(">=0.0.1").unwrap(),
            file_path: PathBuf::from("index.mjs"),
            function_query: FunctionQuery {
                class: None,
                name: "fetch".to_string(),
                typ: FunctionType::FunctionDeclaration,
                kind: FunctionKind::Async,
                index: 0,
            },
            operator: InstrumentationOperator::Promise,
            channel_name: "fetch_decl".to_string(),
        }),
    );
}
