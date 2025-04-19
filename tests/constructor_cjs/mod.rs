use crate::common::*;
use nodejs_semver::Range;
use orchestrion_js::*;
use std::path::PathBuf;

#[test]
fn constructor_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new_single_with_default_dc_module(InstrumentationConfig {
            module_name: "undici".to_string(),
            version_range: Range::parse(">=0.0.1").unwrap(),
            file_path: PathBuf::from("tests/constructor_cjs/index.mjs"),
            function_query: FunctionQuery {
                class: Some("Undici".to_string()),
                name: "constructor".to_string(),
                typ: FunctionType::Method,
                kind: FunctionKind::Sync,
                index: 0,
            },
            operator: InstrumentationOperator::Sync,
            channel_name: "Undici_constructor".to_string(),
        }),
    );
}
