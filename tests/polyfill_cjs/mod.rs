use crate::common::*;
use orchestrion_js::*;

#[test]
fn polyfill_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new(
            vec![InstrumentationConfig::new(
                "fetch_decl",
                test_module_matcher(),
                FunctionQuery::function_declaration("fetch", FunctionKind::Async),
            )],
            Some("./polyfill.js".to_string()),
        ),
    );
}
