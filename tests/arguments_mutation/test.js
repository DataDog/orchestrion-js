/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
const { fetch_simple, fetch_complex } = require('./instrumented.js');
const assert = require('assert');
const { tracingChannel } = require('diagnostics_channel');

const handler = {
  start (message) {
    const originalCb = message.arguments[1];
    const wrappedCb = function (a, b) {
      assert.strictEqual(this.this, 'this');
      assert.strictEqual(a, 'arg1');
      assert.strictEqual(b, 'arg2');
      arguments[1] = 'arg2_mutated';
      return originalCb.apply(this, arguments);
    }

    message.arguments[1] = wrappedCb;
  }
};

tracingChannel('orchestrion:undici:fetch_simple').subscribe(handler);
tracingChannel('orchestrion:undici:fetch_complex').subscribe(handler);

assert.strictEqual(fetch_simple.length, 2);
assert.strictEqual(fetch_complex.length, 2);

const cb = function (a, b) {
  assert.strictEqual(this.this, 'this');
  assert.strictEqual(a, 'arg1');
  assert.strictEqual(b, 'arg2_mutated');
  return 'result';
};

assert.strictEqual(fetch_simple.apply({ this: 'this' }, ['https://example.com', cb]), 'return');
assert.strictEqual(fetch_complex.apply({ this: 'this' }, [{ url: 'https://example.com', tuple: [] }, cb]), 'return');
