/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
class UndiciBase {
    async fetch (url) {
        return 42;
    }
}
class Undici extends UndiciBase {
    async fetch (url) {
        return super.fetch(url);
    }
}

module.exports = Undici;
