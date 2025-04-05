/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
const { tracingChannel } = require('diagnostics_channel');
const assert = require('assert');
function getContext (channelName) {
  const channel = tracingChannel(channelName);
  const context = {};
  channel.subscribe({
    start(message) {
      message.context = context;
      context.start = true;
    },
    end(message) {
      message.context.end = true;
      // Handle end message
    },
    asyncStart(message) {
      message.context.asyncStart = message.result
      // Handle asyncStart message
    },
    asyncEnd(message) {
      message.context.asyncEnd = message.result;
    }
  });
  return context;
}
module.exports = { assert, getContext };
