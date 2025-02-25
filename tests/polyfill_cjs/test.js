const { fetch } = require('./instrumented.js');
const { assert, getContext } = require('../common/preamble.js');
const { tracingChannel } = require('diagnostics_channel');
const context = getContext('orchestrion:undici:fetch_decl');
(async () => {
  const result = await fetch('https://example.com');
  assert.strictEqual(result, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true,
    asyncStart: 42,
    asyncEnd: 42
  });
assert.ok(tracingChannel.polyfilled);
})();
