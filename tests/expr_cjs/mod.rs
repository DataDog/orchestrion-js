use crate::common::*;
use orchestrion_js::*;

#[test]
fn expr_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new_single(InstrumentationConfig::new(
            test_module_matcher(),
            FunctionQuery::function_expression("fetch", FunctionKind::Async),
            "fetch_expr",
        )),
    );
}
