# Cocycle, GUDHI and Ripser: Rips comparison

[Benchmarks](../README.md) / [Reports](README.md)

Cocycle provides a Rust Rips workflow from borrowed inputs and graph construction
to persistence diagrams, explicit complexes and optional representatives. This
comparison checks those workflows against pinned GUDHI and upstream Ripser C++
implementations. It is a **local resource snapshot with correctness validation**,
not a before/after study or a claim of complete GUDHI feature parity.

The measured Rust H0 path performs well on these inputs. F2 H1 still trails
Ripser; generic prime-field computation and explicit expansion have larger gaps.
These observations help prioritize optimization. Results depend on the workload,
requested output and environment; some approximation samples vary substantially.

This is the maintained comparison page. Reruns update this file, including its
measured revisions and conclusions; Git history preserves previous versions.

## What is compared

The following table describes the exercised APIs and adapters, not every feature
of the upstream libraries. All measured computation runs in native executables;
Python's standard library prepares fixtures, launches processes and checks output.
Native execution avoids Python-wrapper overhead but still includes the native
copies and conversions specified by each protocol.

| Tested contract | Cocycle Rust | GUDHI C++ reference | Upstream Ripser C++ reference |
| --- | --- | --- | --- |
| Exact matrix/flag persistence | Specialized F2 H0/H1 and generic prime fields | Simplex tree expansion and persistent cohomology | Implicit dense/sparse persistence |
| Retained explicit complex | Frozen simplices with boundary/cofacet incidence | Simplex tree; different storage contract | Diagram-only reference |
| Sparse Rips approximation | Sampling, modified edges, higher-simplex blockers | Sparse Rips constructor and blocker expansion | No matching approximation constructor |
| Cycles and query-scale cocycles | Optional owned bases; Rust algebraic tests | Diagram-only adapter | Diagram-only adapter |
| Points, upper/square matrices, checked metrics | Public input/construction workflows | Precomputed lower-matrix or unchecked reference only | Precomputed lower-matrix reference; approximation excluded |
| Edge collapse before persistence | Not a measured Rust path | Separate one-pass collapse path in H0/H1 suite | Not a measured Ripser path |

Reference-only rows validate diagrams without competing on unequal requested work.
For implementation details, see the [Rips guide](../../docs/guides/rips.md),
[construction guide](../../docs/guides/rips-construction.md),
[sparse guide](../../docs/guides/sparse-rips.md) and
[native correctness contracts](../../tools/README.md#native-rips-correctness-checks).

## Measured code and environment

| Item | Recorded value |
| --- | --- |
| Rust kernel | Merge commit `6dbfd4298217158831c8df48a4caefec9730b5e9`, [PR #2](https://github.com/huangbogeng/cocycle-rs/pull/2) |
| Performance harness | `f33812f417fd182eaa22eadacfc181dcfa45d753` |
| Exact/sparse correctness harness | `78203f7721f0ce5bee1c57fcac11f84bfe72779c` |
| GUDHI revision | `cba915e3ab8e1f5b1fe26eb44b407285f7af4e78` |
| Ripser revision | `01add51ff64aaf40889483260cc5c3b7d0f2a1e7` |
| Host | AMD EPYC 7H12; x86_64 Linux 5.15.0-185-generic, glibc 2.35 |
| Build | rustc 1.92.0, Cargo release library and optimized worker; g++ 11.4.0, C++17 `-O3 -DNDEBUG`; no custom `RUSTFLAGS` |
| Controls | Serial fresh processes pinned to CPU 2; 2048 MiB address-space cap; frequency, thermal state and competing load uncontrolled |
| Sampling | 12 measured processes per performance cell; no outlier removal |
| Pipeline | `cocycle-rips-pipeline-v2`; one additional discarded warmup; position-balanced rounds, order seed 0; 30 s process timeout |
| H0/H1 | `cocycle-native-v1`; no warmup; shuffled order seed 20260920; 60 s process timeout |

The kernel manifests and `src/` matched the stated merge commit; measured trees
were clean and source fingerprints were checked after the runs. This document's
revision is not the measured revision. Native pins are source baselines, not a
claim to benchmark the latest release. GUDHI's timed sparse sampler uses its
original algorithm with start vertex zero and a read-only permutation accessor;
Ripser has a numeric interval-output hook. See the [pipeline contract](../pipeline/README.md)
and [H0/H1 contract](../protocol.md) for instrumentation and timing boundaries.

## Correctness and run coverage

| Validation | Observed result |
| --- | --- |
| Rust release and tool tests | 97 Rust tests, 5 rustdoc tests and 58 Python tool tests passed; 2 diagnostic profiling tests ignored |
| Exact Rips | 588 fixtures; 1108 native comparisons and 39 protocol checks passed; 68 explicit reference exclusions |
| Sparse approximation | 156 topology comparisons, 153 persistence comparisons and 145 original metric-sampling checks passed |
| Pipeline smoke, attempt 002 | 21 workflows / 59 supported backend groups passed; 4 approximation exclusions |
| Full pipeline, attempt 001 | 25 workflows / 69 groups; 828 measured + 69 warmup processes passed; 6 approximation exclusions |
| H0/H1 baseline, attempt 001 | 12 performance workloads / 48 groups / 576 measured processes passed; 20 semantic fixtures with 76 successful calls and 4 exclusions |

Exact-suite exclusions cover documented precision, field and tiny-input reference
limits. Ripser has no matching sparse approximation constructor; the four H0/H1
exclusions are empty/singleton adapter limits. Unsupported calls are not counted
as agreement. All supported performance samples passed output checks without
timeouts. Native timing adapters check full interval multisets; Rust tests check
representative closure and basis properties.

Smoke attempt 001 exposed an order-dependent controller check: Ripser's zero
explicit-simplex count meant unavailable, but was compared asymmetrically. The
harness fix compares available counts across every applicable backend pair.
The failed attempt remains local and contributes no timing evidence. Attempt 002
and both full timing suites passed; no core algorithm changed during this study.

## Selected performance results

Tables show milliseconds as **median [minimum, maximum]**, with 12 measured samples
per cell. Rows are selected to expose strengths, gaps, ties, cutoffs and different
requested outputs, not to compute an overall score. Complete matrices and all
sample values remain in the local evidence described below.

### Prepared-input H0/H1

These F2 workloads use shared float32-exact, precomputed lower matrices; native
storage is f64 in Rust/GUDHI and f32 in Ripser. Timers begin after input preparation
and end before final JSON export. Uniform inputs use seed 1729; the cutoff row
uses edge length 0.2. H1 rows request H0 through H1, without representatives.

| Workload | Cocycle | GUDHI direct | GUDHI collapse | Ripser |
| --- | --- | --- | --- | --- |
| Uniform H0, n=1024 | 37.081 [36.740, 37.749] | 268.667 [264.755, 272.411] | 3682.013 [3655.707, 3704.987] | 84.424 [83.509, 85.264] |
| Uniform H1, n=128 | 3.832 [3.811, 4.016] | 118.184 [117.649, 121.455] | 9.071 [9.030, 9.223] | 2.665 [2.652, 2.761] |
| Equal-weight H1, n=128 | 2.461 [2.449, 2.500] | 97.092 [96.380, 97.783] | 8.161 [8.098, 8.243] | 0.853 [0.836, 0.884] |
| Uniform H1, n=128, cutoff 0.2 | 1.052 [1.047, 1.073] | 1.508 [1.481, 1.544] | 0.732 [0.720, 0.748] | 0.575 [0.567, 0.598] |

Rust is faster than the measured direct GUDHI and Ripser H0 paths on this row.
Ripser is faster on the selected H1 rows; GUDHI collapse also beats Rust on the
cutoff row. Collapse is explicitly forced in its column: its H0 overhead does
not describe GUDHI's best H0 strategy.

### Complete Rips workflows

Pipeline timers include public validation/conversion, construction, optional
expansion, computation and interval payload export, excluding fixture parsing
and final metrics transport. These times must not be pooled with H0/H1 timings.
Inputs are lower matrices; circle distances are shared float32-exact values.
Nonmetric F3/H2 uses dyadic weights and requests H0 through H2. Approximation
uses an exact dyadic Manhattan metric, epsilon 0.5, start vertex zero, unique
greedy choices and assumed metric hypotheses; it requests F3 H0/H1 diagrams.

| Workflow | Cocycle | GUDHI | Ripser |
| --- | --- | --- | --- |
| Circle F2/H1, n=64 | 2.309 [2.287, 2.339] | 10.474 [10.404, 10.621] | 2.158 [2.140, 2.293] |
| Nonmetric F3/H2, n=24 | 8.996 [8.917, 9.119] | 4.856 [4.801, 5.033] | 1.009 [0.993, 1.042] |
| Same nonmetric input, explicit complex | 23.198 [22.816, 23.652] | 4.896 [4.853, 5.196] | Reference only |
| Sparse approximation F3/H1, n=128 | 20.276 [17.673, 21.943] | 7.974 [7.897, 10.392] | Unsupported |

The nonmetric Rust compute-bin median is 8.979 ms; generic prime-field computation
is a profiling priority on this input. With explicit expansion, Rust spends
11.695 ms in expansion and 10.281 ms in compute. Rust retains bidirectional
incidence and GUDHI retains a tree, so these are workflow costs with different
storage, not an isolated comparison of identical representations. Phase medians
need not sum to the end-to-end median.

Approximation also warrants profiling: Rust's n=128 compute-bin median is
19.206 ms. Smaller n=32 approximation samples varied from 1.493 to 3.671 ms in
Rust and 0.774 to 1.779 ms in GUDHI. This variation limits precise ranking claims;
no slow samples were discarded. Larger inputs and additional environments remain
unmeasured here.

### Process memory on the same pipeline rows

Maximum observed process peak RSS, in KiB, across measured samples. It includes
runtime, parser/input buffers, allocator retention and outputs, not just live
algorithm allocation. Input widths and retained structures differ as above.
Prepared-input RSS and high-water growth are separate raw metrics.

| Workflow | Cocycle | GUDHI | Ripser |
| --- | --- | --- | --- |
| Circle F2/H1, n=64 | 3356 | 6408 | 4212 |
| Nonmetric F3/H2, n=24 | 3624 | 4716 | 3848 |
| Same nonmetric input, explicit complex | 8292 | 4712 | Reference only |
| Sparse approximation F3/H1, n=128 | 4196 | 5032 | Unsupported |

## Reproduction and evidence status

Original raw evidence is **local-only**, with no public artifact URL or guaranteed
retention. The report preserves a reviewed summary, not the original experiment.
The local root is `target/benchmarks/commit-6dbfd4298217/`; it contains the two
correctness `run-001` directories, `pipeline-smoke/run-001` and `run-002`,
`pipeline/run-001`, and `native-baseline/run-001`. Full timing matrices are
`pipeline/run-001/measurements.json` and `native-baseline/run-001/summary.csv`;
raw samples, fixtures, build metadata and validation outcomes accompany them.

| Fingerprint | SHA256 |
| --- | --- |
| Pipeline measured sources | `fe4f3e9ba3173d7fd94e891fa2ab2abc9497ea1a46fcefce73859406d8df5b3d` |
| H0/H1 measured sources | `f58d29ea22b677f0bb85500286ddf3f9188e828774b0fa610ae0f8b3075af82b` |
| Local `evidence-sha256.json` manifest | `30dafab7f1b6fdd49c1b69ba104f6dd9b26f48af8a27680a6c044de3a865d4b9` |

Source fingerprint scope is defined by each controller/builder at the stated
harness revision. The manifest identifies 886 evidence files, excluding build
caches; worker binary and consumed-header hashes remain in build metadata.
A checksum does not make the files publicly available.

To reproduce the performance protocol, use a clean checkout of the performance
harness revision above with the stated toolchain and [native setup](../native/README.md#setup).
Confirm CPU 2 is available or record a different affinity. The following commands
use fresh attempt directories and assume the recorded local Boost header layout;
omit `--boost-include` for system headers. The original timing runs used `run-001`.

```sh
python3 tools/benchmark_rips_pipeline.py --samples 12 --cpu 2 \
  --kernel-revision 6dbfd4298217158831c8df48a4caefec9730b5e9 \
  --boost-include target/native-sources/boost/usr/include \
  --output target/benchmarks/commit-6dbfd4298217/pipeline/run-002
python3 tools/benchmark_native.py --suite baseline --samples 12 --cpu 2 \
  --boost-include target/native-sources/boost/usr/include \
  --output target/benchmarks/commit-6dbfd4298217/native-baseline/run-002
```

For correctness reproduction, use its recorded harness revision and the
[exact](../../tools/README.md#native-rips-correctness-checks) and
[sparse](../../tools/README.md#native-sparse-rips-checks) commands with fresh output
directories. These instructions reproduce the procedure; identical timings are
not guaranteed. New code or harness revisions require new measured identities.

On the next run, replace this page's results, identities, evidence status and
conclusions together. If only one suite is rerun, label the sections separately.
Keep failures and unmatched scopes explicit. See the [reporting rules](../reporting.md)
for details; raw outputs, logs and archives continue to stay outside Git.
