// `tracingChannel` is imported, and `channel` is defined, above here.
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
// Test code after here.
