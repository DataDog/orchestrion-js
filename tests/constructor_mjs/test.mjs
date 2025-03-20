import { Undici } from './instrumented.mjs';
import { assert, getContext } from '../common/preamble.js';
const context = getContext('orchestrion:undici:Undici_constructor');

(() => {
  const undici = new Undici(42);
  assert.deepEqual(undici.val, 42);
  assert.deepStrictEqual(context, {
    start: true,
    end: true
  });
})();