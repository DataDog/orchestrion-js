import { fetch } from './instrumented.mjs';
import { assert, getContext } from '../common/preamble.js';
import { tracingChannel } from 'diagnostics_channel';
const context = getContext('orchestrion:undici:fetch_decl');
const result = await fetch('https://example.com');
assert.strictEqual(result, 42);
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 42,
  asyncEnd: 42
});
assert.ok(tracingChannel.polyfilled);
