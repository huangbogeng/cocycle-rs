# Documentation

Cocycle's documentation describes the current library. Dated measurements live
under `benches/`; the roadmap contains proposed work, not supported features.

| Start here | Purpose |
| --- | --- |
| [User guide](guide.md) | Compute and interpret diagrams |
| [API source](../src/lib.rs) | Public contracts; render with `cargo doc --no-deps --open` |
| [Mathematics](mathematics.md) | Definitions, derivations, and implementation invariants |
| [Architecture](architecture.md) | Production, reference, and tooling boundaries |
| [Testing](testing.md) | Independent oracles, properties, and verification commands |
| [References](references.md) | Primary mathematical and algorithm sources |
| [Roadmap](roadmap.md) | Priorities and admission criteria for future features |
| [Contributing](../CONTRIBUTING.md) | Code, documentation, review, and release rules |
| [Benchmarks](../benches/README.md) | Measurement protocol and dated evidence |

All project documentation is in English. Keep API details in rustdoc, derivations
in the mathematical specification, and completed experiment logs out of the roadmap.
