use crate::common::*;
use orchestrion_js::*;

#[test]
fn decl_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new_single(InstrumentationConfig::new(
            "fetch_decl",
            test_module_matcher(),
            FunctionQuery::function_declaration("fetch", FunctionKind::Async),
        )),
    );
}
