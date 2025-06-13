/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
const assert = require('assert');

function fetch_simple (url, cb) {
  assert.strictEqual(this.this, 'this');
  assert.strictEqual(url, 'https://example.com');
  assert.strictEqual(cb.length, 2);
  const result = cb.apply(this, ['arg1', 'arg2']);
  assert.strictEqual(result, 'result');
  return 'return';
}

function fetch_complex ({ url, tuple: [a = 'a', b = 'b'] }, cb, optional = 'default', ...rest) {
  assert.strictEqual(this.this, 'this');
  assert.strictEqual(url, 'https://example.com');
  assert.strictEqual(a, 'a');
  assert.strictEqual(b, 'b');
  assert.strictEqual(cb.length, 2);
  assert.strictEqual(optional, 'default');
  assert.deepStrictEqual(rest, []);
  const result = cb.apply(this, ['arg1', 'arg2']);
  assert.strictEqual(result, 'result');
  return 'return';
}


module.exports = { fetch_simple, fetch_complex };
