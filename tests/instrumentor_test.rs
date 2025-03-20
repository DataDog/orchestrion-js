use std::path::PathBuf;

mod common;
use common::{init_instrumentor, transpile_and_test};

macro_rules! make_test {
    ($name:ident, $mjs:literal) => {
        #[test]
        fn $name() {
            let file_path = PathBuf::from("index.mjs");
            let mut instrumentor = init_instrumentor(stringify!($name));
            let mut instrumentations =
                instrumentor.get_matching_instrumentations("undici", "0.0.1", &file_path);

            transpile_and_test(stringify!($name), $mjs, &mut instrumentations);
        }
    };
}

make_test!(decl_mjs, true);

make_test!(decl_cjs, false);

make_test!(expr_mjs, true);

make_test!(expr_cjs, false);

make_test!(class_method_cjs, false);

make_test!(object_method_cjs, false);

make_test!(constructor_cjs, false);

make_test!(constructor_mjs, true);

make_test!(polyfill_mjs, true);

make_test!(polyfill_cjs, false);
