# Native H0/H1 validation: draft source e1493a10ae91

[Benchmarks](../README.md) / [Reports](README.md)

Status: local validation of the native harness and its baseline inputs. These
runs use Rust/C++ executables, not GUDHI or Ripser Python bindings. The complete
large-input scaling suite and hosted CI were not run for this record.

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Correctness/smoke validation with initial resource snapshots |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `e1493a10ae91a776a50c80c77395baa9b9fa0c7ed8d7892f30e56e6a23e1198f` |
| Protocol | [cocycle-native-v1](../protocol.md) |
| Provenance | [Verification](../results/native-2026-09-20/verification.json) and [preserved source](../results/native-2026-09-20/source.tar.gz) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.

The source archive contains 42 measured files and reproduces the recorded
fingerprint. It differs from both `707ddaa12122d5d9ac99fa042d416cfeeaf26375` and
`00e4541124dde5b14984cf488eb369932941b58a` in `src/persistence/rips/mod.rs`,
`tools/build_native.py` and `tools/benchmark_inputs.py`. No checked local commit
matches the archive. See the [migration audit](archive/migration.md).

## Implementations and environment

- Cocycle: release Rust public API, Rust 1.92.0.
- GUDHI C++: commit `cba915e3ab8e1f5b1fe26eb44b407285f7af4e78`, direct Rips/CAM and
  one-pass flag-edge-collapse/CAM paths.
- Upstream Ripser C++: commit `01add51ff64aaf40889483260cc5c3b7d0f2a1e7`, default f32
  arithmetic, with only barcode print sites adapted to collect numeric results.
- GCC 11.4.0, C++17, `-O3 -DNDEBUG`, Boost 1.74 headers; no CGAL or Python TDA runtime.
- Linux x86_64, affinity pinned to CPU 0, fresh process for every call, no warmup,
  60-second whole-worker limit and 2 GiB address-space limit per sample.

All backends received the same float32-exact dissimilarities stored as f64.
Cocycle and GUDHI kept f64 kernels. See the [protocol](../protocol.md) for timing,
ownership, process RSS interpretation, and exact multiset validation.

## Results

| Run | Cases | Completed native calls | Validated diagram comparisons | Outcome |
| --- | --- | --- | --- | --- |
| Standard baseline, three samples per performance case | 32 | 220 | 112 | All comparable diagrams passed; four declared exclusions |
| Small scaling suite, two samples per performance case | 32 | 180 | 114 | All comparable diagrams passed; four declared exclusions |

Each run has 124 completed backend/case records and four `unsupported` records.
The exclusions are upstream Ripser's n < 2 adapter domain: empty and singleton
inputs requested through H0 and H1. Cocycle and both GUDHI paths still match the
hand-derived results for those cases. No diagram or timing was fabricated for
Ripser. There were no timeouts, process failures, protocol failures, or mismatches.

The baseline includes H1 inputs through 128 vertices and H0 through 1024 vertices.
The scaling smoke run exercises uniform, circle, normalized 8D cube samples,
nonmetric, grid, and full/truncated bipartite inputs through 36 vertices. These
are validation and initial baseline measurements; they do not establish a
large-input speed or memory ranking. Compare fresh matched experiments before
making such a claim.

## Retained evidence

- [Verification summary](../results/native-2026-09-20/verification.json): counts,
  source identity, exclusions and auxiliary checks.
- Baseline: [environment](../results/native-2026-09-20/baseline/environment.json),
  [raw results](../results/native-2026-09-20/baseline/results.json),
  [CSV](../results/native-2026-09-20/baseline/summary.csv),
  [build metadata](../results/native-2026-09-20/baseline/build/build.json).
- Scaling smoke: [environment](../results/native-2026-09-20/scaling-smoke/environment.json),
  [raw results](../results/native-2026-09-20/scaling-smoke/results.json),
  [CSV](../results/native-2026-09-20/scaling-smoke/summary.csv),
  [build metadata](../results/native-2026-09-20/scaling-smoke/build/build.json).
- [Local source snapshot](../results/native-2026-09-20/source.tar.gz): the exact
  hashed Rust sources, manifests, native adapters and controller sources used by
  both retained runs. External source revisions remain pinned in build metadata.

Fixture bytes, manifests, build logs and the transformed Ripser source accompany
each run. Generated executables and Cargo build caches are excluded from retained
artifacts; their hashes and compilation commands remain in build metadata.
Absolute build paths identify the original execution directories, not the later
retention location. Existing 206 result files were unchanged; 30 historical
fixtures still match the extracted shared generator byte-for-byte.

## Additional checks

Tool tests passed (28), including invalid worker output, exact multiplicity,
coverage, failed process launches, timeouts, and withholding unverified summaries.
Rust debug, release and MSRV runs each passed 60 tests and five doctests; two
profiling tests remained intentionally ignored. Three Markdown examples,
formatting, strict Clippy/rustdoc, and MSRV all-target checks passed. The crate
package was verified to retain the Rust benchmark while excluding native C++
workers, Python tools and raw experiments.

Reproduction uses the [native setup commands](../native/README.md) with new output
directories. Historical wrapper timings remain in separately labeled reports.
