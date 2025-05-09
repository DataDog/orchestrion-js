use crate::common::*;
use orchestrion_js::*;

#[test]
fn expr_mjs() {
    transpile_and_test(
        file!(),
        true,
        Config::new_single(InstrumentationConfig::new(
            "fetch_expr",
            test_module_matcher(),
            FunctionQuery::function_expression("fetch", FunctionKind::Async),
        )),
    );
}
