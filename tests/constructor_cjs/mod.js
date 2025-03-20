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
