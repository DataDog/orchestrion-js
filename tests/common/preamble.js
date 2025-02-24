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
