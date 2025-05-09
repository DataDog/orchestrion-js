use crate::common::*;
use orchestrion_js::*;

#[test]
fn multiple_class_method_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new(
            vec![
                InstrumentationConfig::new(
                    "Undici_fetch1",
                    test_module_matcher(),
                    FunctionQuery::class_method("Undici", "fetch1", FunctionKind::Async),
                ),
                InstrumentationConfig::new(
                    "Undici_fetch2",
                    test_module_matcher(),
                    FunctionQuery::class_method("Undici", "fetch2", FunctionKind::Async),
                ),
            ],
            None,
        ),
    );
}
