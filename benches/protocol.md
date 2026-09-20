# Native benchmark protocol

[Benchmarks](README.md)

Status: implemented native worker protocol, `cocycle-native-v1`. It supersedes
the [Python-wrapper protocol](reports/python-protocol.md) for new performance
comparisons. It does not change the meaning of historical measurements.

## Inputs and precision

Generate a fixture once and give its identical bytes to all workers. The
`COCYCLE1` header records input mode, vertex count, ambient dimension metadata,
maximum homology dimension, and cutoff; values are little-endian f64 in condensed
strict-lower-triangle order. The native comparison accepts precomputed distances
only. File parsing and layout preparation finish before timing.

Quantize distances and cutoffs to float32 once during fixture generation, then
store them losslessly as f64. Cocycle and GUDHI retain f64 computation; upstream
Ripser retains its default `value_t = float`. This gives the same input filtration
without silently changing Ripser's scalar type or loosening endpoint tolerances.
It is not an equal-byte-width kernel comparison. Direct point-cloud entry points
need a separate distance-construction experiment.

All backends compute ordinary H0 or H0/H1 over F2, with closed edge-length
thresholds and zero-length pairs omitted. GUDHI expands through q+1, and includes
the complex's top dimension when necessary to retain triangle-free H1. Collapse
retains isolated vertices and uses exactly one native edge-collapse call. No path
uses sampling or sparse-Rips approximation.

The unrestricted upstream Ripser path uses dense access and computes its enclosing
radius inside the timer. Explicit cutoffs use its sparse threshold representation;
constructing that representation is timed. Public coverage still follows the
caller cutoff and original diameter, not an internal stopping bound.

## Copies and ownership

| Backend | Prepared input | Work counted inside the call |
| --- | --- | --- |
| Cocycle | Borrowed condensed f64 slice | Filtration/index storage, reduction, owned diagram assembly |
| GUDHI direct | Borrowed condensed f64 view with lower-triangle row access | Proximity graph, explicit complex, CAM state, owned interval extraction |
| GUDHI collapse | Same condensed view | Threshold edge list, collapse state, expanded complex, CAM, owned intervals |
| Ripser | Owned condensed f32 matrix prepared once | Dense ownership move or sparse graph construction, reduction, numeric output collection |

There is no ndarray, pybind, Cython, square-matrix conversion, or Python runtime
inside a worker. Native algorithms still allocate, materialize structures, or
copy entries where their interfaces require it. Those costs stay in the measured
operation. The f64-to-f32 Ripser input conversion is explicit preparation; its
f64 temporary is released before the memory baseline. Do not describe the whole
pipeline as zero-copy.

## Timing and isolation

Each sample is a **fresh native process with exactly one computation and no
warmup**. The controller serializes backend runs and shuffles their order with a
recorded seed for each repetition. Semantic fixtures use one sample; performance
fixtures use the configured sample count. CPU affinity is optional and recorded.

Worker clocks start after parsing, validation, options, and native input layout
preparation. They stop when an owned normalized interval result is ready and
algorithm workspaces have been destroyed. The timer includes construction,
reduction, interval extraction/normalization and algorithm cleanup. It excludes
process startup, fixture I/O, JSON formatting, and final result destruction.
These are time-to-result measurements, not isolated reduction-kernel timings.

Upstream Ripser's internal static enumerators retain references to the first
engine in a process. Repeatedly creating engines in that process is unsuitable
for this adapter; using one process per sample avoids changing upstream algorithm
code or reusing dangling state. The output adapter replaces six print statements
with numeric pair collection or no-ops, verified against a pinned file hash. It
keeps the original license and algorithm unchanged. Results are not parsed from
rounded CLI text. The adapter and transformed source hashes are retained.

The wall-time limit covers the whole worker, including startup, parsing,
computation, and serialization. A timeout is not a native-call timing value.
Linux `RLIMIT_AS` caps address space, not resident memory. Both limits are benchmark
controls; the library itself makes no such resource guarantees.

## Memory

Record prepared-input `VmRSS`, prepared-input `VmHWM`, and post-computation
`VmHWM` in KiB for every sample. The owned result is still alive at the last
reading, before JSON serialization. Fresh processes prevent previous samples
from contaminating their high-water marks; input parsing can still establish a
higher mark than the later computation.

Report absolute process peaks and HWM growth separately. Growth can be zero even
when allocations occurred. Process RSS includes input buffers, allocator/runtime
state and result storage; it is not an allocator-level peak or a proof of zero
copying. Ripser's f32 input has a different size from the f64 inputs. Do not turn
raw RSS ratios into claims about equal-representation algorithm memory.

## Validation and failure handling

Check every measured output: dimensions, coverage, endpoint kinds, finite scales,
positive finite lifetime, and multiplicity. Compare exact diagram multisets across
backends and repetitions. Semantic cases use hand-derived expected results;
bipartite cases also use their analytic multiplicities. Rust's independent
explicit oracle remains a separate correctness layer.

Essential endpoints are allowed only with complete coverage. Surviving classes
at an incomplete cutoff are right-censored, including H0. Compare completed
external backends even if Cocycle fails or times out. A lone unverified survivor
gets no summary timing. Mismatches invalidate the case's summary; partial sample
sets, errors and timeouts remain in raw records without being ranked.

The upstream compressed-matrix adapter explicitly excludes n < 2. Record those
cases as `unsupported`, without synthesizing a Ripser diagram or elapsed time;
Cocycle and GUDHI still validate the empty/singleton contract. These adapter
exclusions produce `passed_with_exclusions`. Crashes, malformed output, mismatches,
timeouts, and unverified cases produce a nonzero controller exit status.

## Reproducibility and retention

Pin native sources in [sources.json](native/sources.json). The builder rejects a
wrong revision, modified checkout, or changed Ripser output sites. Local benchmark
sources must also remain unchanged throughout compilation and measurement. Retain compiler
versions and commands, consumed non-system header hashes, Boost version, generated
Ripser source hash, executable hashes, local source hash, fixture hashes, CPU,
affinity, sample order, precision, and limits. System C++ headers belong to the
recorded compiler installation; no claim of a hermetic toolchain is made.

The default baseline combines analytic semantic cases and small varied inputs.
The scaling suite schedules **all four paths for every size**, including hard
nonmetric and bipartite cases; no GUDHI size omission is implicit. Timeouts and
errors remain evidence. Native and historical Python-wrapper samples must never
be pooled into one ranking. Reproduction commands belong in the
[native setup guide](native/README.md).
