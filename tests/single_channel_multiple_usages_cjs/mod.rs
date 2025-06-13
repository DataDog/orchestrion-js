use crate::common::*;
use orchestrion_js::*;

#[test]
fn same_channel_multiple_usages_cjs() {
    transpile_and_test(
        file!(),
        false,
        Config::new(
            vec![
                InstrumentationConfig::new(
                    "method_call",
                    test_module_matcher(),
                    FunctionQuery::function_declaration("foo", FunctionKind::Async),
                ),
                InstrumentationConfig::new(
                    "method_call",
                    test_module_matcher(),
                    FunctionQuery::function_declaration("bar", FunctionKind::Async),
                ),
            ],
            None,
        ),
    );
}
