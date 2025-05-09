use crate::common::*;
use orchestrion_js::*;

#[test]
fn class_method_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new_single(InstrumentationConfig::new(
            "Undici_fetch",
            test_module_matcher(),
            FunctionQuery::class_method("Undici", "fetch", FunctionKind::Async),
        )),
    );
}
