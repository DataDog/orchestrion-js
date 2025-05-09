use crate::common::*;
use orchestrion_js::*;

#[test]
fn object_method_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new_single(InstrumentationConfig::new(
            "Undici_fetch",
            test_module_matcher(),
            FunctionQuery::object_method("fetch", FunctionKind::Async),
        )),
    );
}
