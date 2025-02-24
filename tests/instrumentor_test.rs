use std::path::PathBuf;

mod common;
use common::{init_instrumentor, transpile_and_test};

macro_rules! make_test {
    ($name:ident, $typ:literal, $mjs:literal) => {
        #[test]
        fn $name() {
            let mut instrumentor = init_instrumentor($typ);
            let instrumentations = instrumentor.get_matching_instrumentations(
                "undici",
                "0.0.1",
                &PathBuf::from("index.mjs"),
            );

            transpile_and_test(stringify!($name), $mjs, instrumentations);
        }
    };
}

make_test!(decl_mjs, "decl", true);

make_test!(decl_cjs, "decl", false);

make_test!(expr_mjs, "expr", true);

make_test!(expr_cjs, "expr", false);

make_test!(class_method_cjs, "expr", false);
