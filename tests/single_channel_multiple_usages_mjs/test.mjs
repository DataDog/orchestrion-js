/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
import { foo, bar } from './instrumented.mjs';
import { assert, getContext } from '../common/preamble.js';
const context = getContext('orchestrion:undici:method_call');
const result = await foo();
assert.strictEqual(result, 'foo');
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 'foo',
  asyncEnd: 'foo'
});

const result2 = await bar();
assert.strictEqual(result2, 'bar');
assert.deepStrictEqual(context, {
  start: true,
  end: true,
  asyncStart: 'bar',
  asyncEnd: 'bar'
});