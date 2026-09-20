# Repository cleanup verification: source 977573dab74e

[Benchmarks](../../../README.md) / [Reports](../../README.md)

Historical maintenance verification; includes Python-wrapper comparisons,
not a native C++ benchmark.

This record covers a source-layout and documentation refactor after the
[same-size benchmark](../source-be652a04dba1-python-threeway.md). It does not claim new performance
measurements or a published release.

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived maintenance verification; no performance claim |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `977573dab74e42742974e2b6ef652fbb692f6284bf266a38d961dea5275ba6fb` |
| Protocol | Historical local checks and wrapper smoke suites |
| Provenance | [Check summary](../../../results/project-cleanup-2026-09-17.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.

## Changes under review

- Move explicit simplices, filtration, sparse columns, and boundary reduction into
  the `cfg(test)`-only `persistence/reference/` module.
- Separate validated options from entry-point/result assembly, preserving the
  `cocycle::persistence::RipsOptions` public path.
- Separate ignored diagnostics from correctness tests and update their Python
  callers to the new private test paths.
- Replace development milestones and mixed validation logs with English user,
  architecture, mathematical, testing, roadmap, and contributor documentation.
- Preserve raw benchmark artifacts and translate retained reports. Historical
  measurements still identify their original source hashes.
- Enforce public rustdoc, add a local documentation check to CI, and narrow the
  crate package to library sources, tests, examples, Rust benchmark, and core docs.

## Verification

Local checks completed on Linux x86_64 with Rust 1.98.0 and MSRV Rust 1.91.0:

| Check | Result |
| --- | --- |
| Debug, release, and MSRV tests | 60 Rust tests and five doctests each; two profiling tests intentionally ignored |
| README and guide examples | Three additional Markdown doctests passed |
| Formatting, Clippy, rustdoc | Passed, with warnings denied for Clippy and rustdoc |
| MSRV all-target check | Passed |
| Documentation | 19 English Markdown files checked for local links; positive/negative checker probes passed |
| Python tool regression tests | 15 passed, including strict ResourceWarning handling |
| Independent Ripser comparison | 512 diagram comparisons passed |
| GUDHI quick suite | 28 fixtures, 56 diagram comparisons passed |
| Shared-precision quick suite | 27 fixtures, 79 comparisons passed; two known Ripser empty-input exclusions retained |
| Scaling smoke suite | 14 fixtures, 56 completed workers, 46 comparisons including four analytic checks; no limits or failures |
| Profiling entry points | Six ablation stages and two workload cases passed diagram validation |
| Crate package | Verified locally; packaged square example produced the expected diagram and Betti curve |

The [check summary](../../../results/project-cleanup-2026-09-17.json) retains source identity,
commands, and external-suite metadata. Smoke outputs were generated under
`target/cleanup-*`; their timings are not promoted as performance measurements.
For Rust checks use [CONTRIBUTING.md](../../../../CONTRIBUTING.md#verification). External
reproduction commands, using new output paths:

```sh
python tools/compare_ripser.py
python -W error::ResourceWarning -m unittest discover -s tools -p 'test_*.py'
python tools/benchmark_gudhi.py --quick --samples 1 --output target/cleanup-gudhi
python tools/benchmark_gudhi.py --include-ripser --quick --samples 1 --output target/cleanup-ripser
python tools/benchmark_scaling.py --quick --cpu 0 --samples 1 --output target/cleanup-scaling
python tools/profile_rips.py target/cleanup-scaling/fixtures/uniform_h1_16.bin --cpu 0 --output target/cleanup-ablation.json
python tools/profile_scaling.py --benchmark target/cleanup-scaling --cases uniform_h1_16 bipartite_h1_32_cutoff --output target/cleanup-work.json
```

Raw historical benchmark records remain evidence for their original source hashes.
At the time of these checks, no hosted CI or publication had been performed.
This historical statement does not describe current repository or CI status;
consult the current contribution and release rules.
