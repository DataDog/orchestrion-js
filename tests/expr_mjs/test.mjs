import { fetch } from './instrumented.mjs';
import { assert, getContext } from '../common/preamble.js';
const context = getContext('orchestrion:undici:fetch_expr');
const result = await fetch('https://example.com');
assert.strictEqual(result, 42);
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 42,
  asyncEnd: 42
});
