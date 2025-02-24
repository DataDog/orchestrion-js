const { fetch } = require('./instrumented.js');
const { assert, getContext } = require('../common/preamble.js');
const context = getContext('orchestrion:undici:Undici_fetch');
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
