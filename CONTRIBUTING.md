# Contributing to Cocycle

Cocycle is a small, general-purpose Rust library. Contributions should improve a
concrete mathematical capability, correctness, usability, or measured performance.
Application workflows and language bindings are outside this crate.

## Development environment

Use Rust 1.91 or later, rustfmt, and Clippy. The core has no external runtime or
test dependencies. Python is optional and only needed for external comparisons.
Start with `cargo test --locked` and the [architecture](docs/architecture.md).

Use the [issue tracker](https://github.com/huangbogeng/cocycle-rs/issues) for
reproducible bugs and substantial API or algorithm proposals. Small, focused fixes
can be reviewed directly through pull requests. Do not add placeholder implementations for future work.

## Code and API rules

- Keep the public surface under `geometry`, `persistence`, `diagram`, and
  `descriptors`. Prefer concrete types until multiple real implementations require
  an abstraction; do not expose internal storage or algorithm switches.
- Validate input at construction. Preserve interval multiplicity, cutoff coverage,
  and the distinction between finite, essential, and censored endpoints.
- Explain algorithm invariants and non-obvious ordering decisions in comments.
  Public items need rustdoc, error behavior, and examples where useful.
- Unsafe code is forbidden. Check size arithmetic and use fallible reservations
  where supported. Do not promise that every allocation failure is recoverable.
- Keep the explicit reference implementation independent and test-only. An
  optimized implementation must not generate its own expected test results.
- Keep changes focused. New dependencies need a concrete benefit and license/MSRV
  review; Python tools must not become Rust runtime dependencies.
- Write documentation, comments, issue templates, and change descriptions in
  English. Use `cargo fmt`; keep prose about current behavior separate from dated
  experiment reports.

Public API includes documented numeric, ordering, and error semantics, not only
Rust signatures. Before publication, describe intentional breaking changes in the
changelog. After publication, preserve compatibility within a 0.x minor line and
use a new minor version for incompatible changes. MSRV changes must be explicit.

## Verification

Run these for Rust changes:

```sh
cargo fmt --all -- --check
rustfmt --edition 2024 --check tools/diagram_dump.rs tools/benchmark_driver.rs
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --release --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo +1.91.0 test --locked --all-features
cargo +1.91.0 check --locked --all-targets --all-features
python3 tools/check_docs.py
cargo build --locked
rustdoc --edition 2024 --test README.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
rustdoc --edition 2024 --test docs/guide.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
```

The documentation check needs only Python's standard library. For mathematical
or algorithm changes, update the [specification](docs/mathematics.md), add an
independent expected result or property, and run the relevant external comparisons
in [tools/README.md](tools/README.md). Performance changes need the same fixtures,
precision, and timing boundaries before and after; keep unfavorable results.
Ordinary tests must not assert machine-dependent timing thresholds.

Documentation-only changes need the documentation check and any affected examples;
they do not require rerunning large performance experiments. See
[testing](docs/testing.md) for coverage and CI responsibilities.

## Pull requests

Describe the problem, resulting behavior, mathematical basis if relevant, and
checks actually run. Identify incomplete checks and limitations. Update the
unreleased changelog for user-visible changes. Avoid unrelated formatting or
speculative abstractions. The pull request template is a guide, not a requirement
to add irrelevant sections.

Review focuses on correctness, public contracts, independent evidence, and whether
the change fits the library's scope. Discuss technical choices respectfully and
make feedback specific and actionable.

## Documentation ownership

| Information | Authoritative location |
| --- | --- |
| Function/type behavior | Rustdoc next to the public item |
| Task-oriented usage | `docs/guide.md` |
| Mathematical conventions and proofs | `docs/mathematics.md` |
| Module boundaries | `docs/architecture.md` |
| Test obligations | `docs/testing.md` |
| Planned work | `docs/roadmap.md` |
| Measured performance | Dated reports and artifacts under `benches/` |
| User-visible changes | `CHANGELOG.md` |

A dated report describes the source hash and protocol at measurement time. Do not
rewrite its raw data after a refactor or treat old timing as a new measurement.
The [benchmark guide](benches/README.md) explains artifact retention.

## Release procedure

Releases are an explicit maintainer action. A successful local build is not a
release. Before the first publication, verify registry-name availability and
establish the publishing identity. The source repository is
[huangbogeng/cocycle-rs](https://github.com/huangbogeng/cocycle-rs); repository
bootstrap does not publish a crate or reserve its name.

For every release:

1. Finalize the version and changelog, review public API and MSRV changes.
2. Run the required CI jobs for the exact release commit; inspect their results.
3. Run `cargo package --locked`, inspect the package file list, and execute its
   `square` example. Exclude raw experiments and temporary files from the crate.
4. Publish only after those checks pass, then verify installation from crates.io.
5. Record the release tag and notes for the published source.

`cargo package --allow-dirty` is acceptable for local review of uncommitted work;
it is not the release procedure. Do not fabricate repository URLs or success
badges before a hosted repository and CI results exist.

Contributions are made under the project's [MIT License](LICENSE). Attribute
external code and check its license before incorporating it; citing an algorithm
paper does not license copying an implementation.
