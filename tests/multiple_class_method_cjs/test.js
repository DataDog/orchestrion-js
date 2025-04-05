/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
const Undici = require('./instrumented.js');
const { assert, getContext } = require('../common/preamble.js');
const context1 = getContext('orchestrion:undici:Undici_fetch1');
const context2 = getContext('orchestrion:undici:Undici_fetch2');

(async () => {
  const undici = new Undici;
  const result1 = await undici.fetch1('https://example.com');
  assert.strictEqual(result1, 42);
  assert.deepStrictEqual(context1, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
  const result2 = await undici.fetch2('https://example.com');
  assert.strictEqual(result2, 43);
  assert.deepStrictEqual(context2, {
    start: true,
    end: true,
    asyncStart: 43,
    asyncEnd: 43
  });
})();
