# Contributing to Cocycle

Cocycle is a small, general-purpose Rust library. Contributions should improve a
concrete mathematical capability, correctness, usability, or measured performance.
Application workflows and language bindings are outside this crate.

## Development environment

Use Rust 1.91 or later, rustfmt, and Clippy. The core has no external runtime or
test dependencies. Python's standard library runs documentation checks and tool
tests. Native comparisons need a C++17 compiler, Boost headers, and pinned GUDHI
and Ripser sources; see the [native setup](benches/native/README.md). Python TDA
packages are only needed for [historical wrapper reproduction](tools/legacy-benchmarks.md).
Start with `cargo test --locked` and the [architecture](docs/development/architecture.md).

Use the [issue tracker](https://github.com/huangbogeng/cocycle-rs/issues) for
reproducible bugs and substantial API or algorithm proposals. Small, focused fixes
can be reviewed directly through pull requests. Do not add placeholder implementations for future work.

## Code and API rules

Follow the [code conventions](docs/development/conventions.md) for file boundaries,
naming, reuse, public contracts, errors, numeric behavior, language, and style.
Keep changes focused. New dependencies need a concrete benefit and license/MSRV
review; Python tools and C++ comparison workers are outside the Rust runtime.

Public API includes documented numeric, ordering, and error semantics, not only
Rust signatures. Before publication, describe intentional breaking changes in the
changelog. After publication, preserve compatibility within a 0.x minor line and
use a new minor version for incompatible changes. MSRV changes must be explicit.

## Verification

Run these from the repository root for Rust changes. CI distributes them across
quality, test, and MSRV jobs:

```sh
cargo fmt --all -- --check
rustfmt --edition 2024 --check tools/diagram_dump.rs tools/benchmark_driver.rs benches/native/cocycle.rs tools/reference/rips_cocycle.rs tools/reference/sparse_cocycle.rs benches/pipeline/cocycle.rs
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
cargo test --locked --release --all-features
cargo run --locked --example square
cargo run --locked --example rips_graph
cargo run --locked --example flag_persistence
cargo run --locked --example rips_sphere
cargo run --locked --example rips_representatives
cargo run --locked --example sparse_rips
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps
cargo +1.91.0 test --locked --all-features
cargo +1.91.0 check --locked --all-targets --all-features
python3 tools/check_source.py
python3 tools/check_docs.py
python3 -m unittest discover -s tools -p 'test_*.py'
cargo build --locked
rustdoc --edition 2024 --test README.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
rustdoc --edition 2024 --test docs/guides/rips.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
rustdoc --edition 2024 --test docs/guides/rips-construction.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
rustdoc --edition 2024 --test docs/guides/rips-representatives.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
rustdoc --edition 2024 --test docs/guides/sparse-rips.md --extern cocycle=target/debug/libcocycle.rlib -L dependency=target/debug/deps
```

Select additional checks by the changed contract:

| Change | Required evidence |
| --- | --- |
| Documentation only | Source and Markdown checks; run affected Rust examples/doctests |
| Rust implementation or public API | Commands above; update contract tests and relevant rustdoc |
| Mathematical algorithm | Independent expectation/property and relevant native comparisons, in addition to Rust checks |
| Python checks or controllers | Source and Markdown checks, all `test_*.py`; exercise the changed command on a small case |
| Native adapters, builder, or benchmark protocol | Tool tests, standalone Rust formatting when affected, and the native smoke command below |
| Performance | Comparable before/after measurements under the applicable native suite and reporting rules; keep unfavorable results |
| File layout or packaging | Relevant checks above, package file-list review, package build and packaged example |

The source and documentation checks need only Python's standard library. For
mathematical or algorithm changes, update the [specification](docs/reference/mathematics.md), add an
independent expected result or property, and run the relevant external comparisons
in [tools/README.md](tools/README.md). Performance changes need the same fixtures,
precision, and timing boundaries before and after; keep unfavorable results.
Ordinary tests must not assert machine-dependent timing thresholds.

Benchmark changes must follow the [reporting rules](benches/reporting.md) and the
affected execution contract: [H0/H1 native](benches/protocol.md) or
[Rips pipeline](benches/pipeline/README.md). Run the affected native smoke suite
when changing workers, controllers or measurement semantics. GUDHI and Ripser comparisons use C++
executables; Python TDA wrappers belong only to historical reproduction. Tool
changes also need `python3 -m unittest discover -s tools -p 'test_*.py'`.

After the native setup, use a new output directory for each run:

```sh
python3 tools/benchmark_native.py --quick --samples 1 --output target/native-smoke
```

Supply `--boost-include` when Boost headers are outside system include paths.
For exact graph/matrix changes also run
`python3 tools/compare_rips.py --output target/rips-reference` with a fresh output
directory and the same optional Boost setting. For sparse approximation, also run
`python3 tools/compare_sparse_rips.py --output target/sparse-rips-reference`
with a fresh output directory. For workflow timing/resource changes, also run
`python3 tools/benchmark_rips_pipeline.py --quick --samples 1 --output target/rips-pipeline-smoke`.
Treat unsupported reference inputs as documented exclusions, not successful
cross-library comparisons. See the native guide for platform requirements.

Documentation-only changes need source/documentation checks and any affected examples;
they do not require rerunning large performance experiments. See
[testing](docs/development/testing.md) for coverage and CI responsibilities.

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

The [documentation index](docs/README.md#organization-and-maintenance) defines
where usage, reference, development, design, and research pages belong. It also
links each authoritative document. API behavior belongs in rustdoc next to the
item; coding rules belong in the code conventions; verification commands and
contribution procedures belong in this file. Link to the owner rather than
duplicating its source tree, commands, or detailed rules.

When adding or moving a document:

1. Update the index, relative links, heading fragments, and literal paths in
   commands or tooling. Clearly label proposals and dated source observations.
2. Run `python3 tools/check_docs.py`. It recursively checks `docs/` and selected
   repository Markdown for inline/reference local targets, headings, and CJK
   text. It does not fetch external URLs or establish English prose quality.
   Run affected Rust examples using the commands above.
3. Keep CI's guide doctest path aligned with the guide. If changing the checker,
   run `python3 -m unittest discover -s tools -p 'test_check_docs.py'`.
4. If changing directories or package inclusion, inspect
   `cargo package --locked --allow-dirty --list` and ensure nested docs remain in
   the package. Raw benchmark artifacts remain outside the crate payload.

Performance reports bind to the measured commit and associated PR, with a full
SHA, source fingerprint and protocol. Dates are execution metadata. The report
commit and measured commit are distinct; uncommitted runs remain drafts. Do not
rewrite its raw data after a refactor or treat old timing as a new measurement.
The [reporting rules](benches/reporting.md) own evidence classification,
comparability, sampling and artifact retention; use the
[report template](benches/report-template.md) for new experiments.

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
