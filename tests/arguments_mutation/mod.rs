use crate::common::*;
use orchestrion_js::*;

#[test]
fn arguments_mutation() {
    transpile_and_test(
        file!(),
        false,
        Config::new(
            vec![
                InstrumentationConfig::new(
                    "fetch_simple",
                    test_module_matcher(),
                    FunctionQuery::function_declaration("fetch_simple", FunctionKind::Sync),
                ),
                InstrumentationConfig::new(
                    "fetch_complex",
                    test_module_matcher(),
                    FunctionQuery::function_declaration("fetch_complex", FunctionKind::Sync),
                ),
            ],
            None,
        ),
    );
}
