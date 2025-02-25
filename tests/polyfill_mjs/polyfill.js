module.exports = require('diagnostics_channel');
const tracingChannel = module.exports.tracingChannel;
tracingChannel.polyfilled = true;
module.exports.tracingChannel = tracingChannel;
