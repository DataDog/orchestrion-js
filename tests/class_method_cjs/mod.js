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
