/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
class Undici {
    async fetch1 (url) {
        return 42;
    }

    async fetch2 (url) {
        return 43;
    }
}

module.exports = Undici;
