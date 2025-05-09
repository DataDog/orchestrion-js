use crate::common::*;
use orchestrion_js::*;

#[test]
fn index_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new_single(InstrumentationConfig::new(
            "Undici_fetch",
            test_module_matcher(),
            FunctionQuery::ClassMethod {
                class_name: "Undici".to_string(),
                method_name: "fetch".to_string(),
                kind: FunctionKind::Async,
                index: 2,
            },
        )),
    );
}
