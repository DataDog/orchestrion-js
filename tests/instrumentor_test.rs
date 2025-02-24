use std::path::PathBuf;

mod common;
use common::{init_instrumentor, transpile_and_test};

#[test]
fn decl_mjs() {
    let mut instrumentor = init_instrumentor("decl");
    let instrumentations =
        instrumentor.get_matching_instrumentations("undici", "0.0.1", &PathBuf::from("index.mjs"));

    let contents = "export async function fetch(url) { return 42; }";
    let test_code = r#"
import { fetch } from './instrumented.mjs';
const result = await fetch('https://example.com');
assert.strictEqual(result, 42);
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 42,
  asyncEnd: 42
});
    "#;
    transpile_and_test(
        "orchestrion:undici:fetch",
        true,
        instrumentations,
        contents,
        test_code,
    );
}

#[test]
fn decl_cjs() {
    let mut instrumentor = init_instrumentor("decl");
    let instrumentations =
        instrumentor.get_matching_instrumentations("undici", "0.0.1", &PathBuf::from("index.mjs"));

    let contents = "async function fetch(url) { return 42; }\nmodule.exports = { fetch };";
    let test_code = r#"
const { fetch } = require('./instrumented.js');
(async () => {
  const result = await fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
})();
        "#;
    transpile_and_test(
        "orchestrion:undici:fetch",
        false,
        instrumentations,
        contents,
        test_code,
    );
}

#[test]
fn expr_mjs() {
    let mut instrumentor = init_instrumentor("expr");

    let instrumentations =
        instrumentor.get_matching_instrumentations("undici", "0.0.1", &PathBuf::from("index.mjs"));

    let contents = "const fetch = async function (url) { return 42; }; export { fetch };";
    let test_code = r#"
import { fetch } from './instrumented.mjs';
const result = await fetch('https://example.com');
assert.strictEqual(result, 42);
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 42,
  asyncEnd: 42
});
    "#;
    transpile_and_test(
        "orchestrion:undici:fetch",
        true,
        instrumentations,
        contents,
        test_code,
    );
}

#[test]
fn expr_cjs() {
    let mut instrumentor = init_instrumentor("expr");

    let instrumentations =
        instrumentor.get_matching_instrumentations("undici", "0.0.1", &PathBuf::from("index.mjs"));

    let contents = "exports.fetch = async function (url) { return 42; };";
    let test_code = r#"
const { fetch } = require('./instrumented.js');
(async () => {
  const result = await fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
})();
    "#;
    transpile_and_test(
        "orchestrion:undici:fetch",
        false,
        instrumentations,
        contents,
        test_code,
    );
}

#[test]
fn class_method_cjs() {
    let mut instrumentor = init_instrumentor("expr");

    let instrumentations =
        instrumentor.get_matching_instrumentations("undici", "0.0.1", &PathBuf::from("index.mjs"));

    let contents = r#"
class Undici {
    async fetch (lmao) {
        return 42;
    }
}

module.exports = Undici;
"#;
    let test_code = r#"
const Undici = require('./instrumented.js');
(async () => {
  const undici = new Undici;
  const result = await undici.fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
})();
    "#;
    transpile_and_test(
        "orchestrion:undici:Undici_fetch",
        false,
        instrumentations,
        contents,
        test_code,
    );
}
