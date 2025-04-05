# Orchestion-JS

Orchestrion is a library for instrumenting Node.js libraries at build or load time.
It provides [`VisitMut`] implementations for SWC's AST nodes, which can be used to insert tracing code into matching functions.
It's entirely configurable via a YAML string, and can be used in SWC plugins, or anything else that mutates JavaScript ASTs using SWC.

## Contributing

See CONTRIBUTING.md

## License

See LICENSE

[`VisitMut`]: https://rustdoc.swc.rs/swc_core/ecma/visit/trait.VisitMut.html
