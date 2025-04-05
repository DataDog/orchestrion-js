/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
module.exports = require('diagnostics_channel');
const tracingChannel = module.exports.tracingChannel;
tracingChannel.polyfilled = true;
module.exports.tracingChannel = tracingChannel;
