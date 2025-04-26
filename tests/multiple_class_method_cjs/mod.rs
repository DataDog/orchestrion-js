use crate::common::*;
use nodejs_semver::Range;
use orchestrion_js::*;
use std::path::PathBuf;

#[test]
fn multiple_class_method_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new(
            vec![
                InstrumentationConfig {
                    module_name: "undici".to_string(),
                    version_range: Range::parse(">=0.0.1").unwrap(),
                    file_path: PathBuf::from("index.mjs"),
                    function_query: FunctionQuery {
                        class: Some("Undici".to_string()),
                        name: "fetch1".to_string(),
                        typ: FunctionType::Method,
                        kind: FunctionKind::Async,
                        index: 0,
                    },
                    operator: InstrumentationOperator::Promise,
                    channel_name: "Undici_fetch1".to_string(),
                },
                InstrumentationConfig {
                    module_name: "undici".to_string(),
                    version_range: Range::parse(">=0.0.1").unwrap(),
                    file_path: PathBuf::from("index.mjs"),
                    function_query: FunctionQuery {
                        class: Some("Undici".to_string()),
                        name: "fetch2".to_string(),
                        typ: FunctionType::Method,
                        kind: FunctionKind::Async,
                        index: 0,
                    },
                    operator: InstrumentationOperator::Promise,
                    channel_name: "Undici_fetch2".to_string(),
                },
            ],
            "diagnostics_channel".to_string(),
        ),
    );
}
