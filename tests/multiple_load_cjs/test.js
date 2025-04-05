/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
const Undici = require('./instrumented.js');
const { assert, getContext } = require('../common/preamble.js');
const context = getContext('orchestrion:undici:Undici_fetch');
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
