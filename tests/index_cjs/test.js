/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
const undicis = require('./instrumented.js');
const { assert, getContext } = require('../common/preamble.js');
const context = getContext('orchestrion:undici:Undici_fetch');

async function testOne(Undici, num, expectedCtx) {
  const undici = new Undici;
  const result = await undici.fetch('https://example.com');
  assert.strictEqual(result, num);
  assert.deepStrictEqual(context, expectedCtx);
  delete context.start;
  delete context.end;
  delete context.asyncStart;
  delete context.asyncEnd;
}

(async () => {
  await testOne(undicis.Undici0, 0, {});
  await testOne(undicis.Undici1, 1, {});
  await testOne(undicis.Undici2, 2, {
    start: true,
    end: true,
    asyncStart: 2,
    asyncEnd: 2
  });
  await testOne(undicis.Undici3, 3, {});
  await testOne(undicis.Undici4, 4, {});
})();
