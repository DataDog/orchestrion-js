/**
 * Unless explicitly stated otherwise all files in this repository are licensed under the Apache-2.0 License.
 * This product includes software developed at Datadog (https://www.datadoghq.com/). Copyright 2025 Datadog, Inc.
 **/
class UndiciBase {
    constructor () {
    }
}

class Undici extends UndiciBase {
    constructor (val) {
        super();
        this.val = val;
    }
}

module.exports = Undici;
