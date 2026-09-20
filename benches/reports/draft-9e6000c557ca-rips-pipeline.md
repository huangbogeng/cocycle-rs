# Rips subsystem acceptance: draft source 9e6000c557ca

[Benchmarks](../README.md) / Reports

The implemented Rips paths pass the recorded native correctness checks and this
local workflow resource matrix. These measurements also show remaining performance
costs: generic odd-prime/higher-dimensional computation, explicit materialization
and representative extraction can cost substantially more than the specialized
H1 path. This is not a claim of universal parity or speed superiority.

## Revision binding

| Item | Recorded identity |
| --- | --- |
| Measured commit | Unbound: this experiment measured uncommitted working-tree sources |
| Associated PR | Not bound to this run |
| Pipeline source fingerprint | `9e6000c557ca183863336c9406cf39c89a9d058d7fa081cc4871f0631efdb7f2` |
| Baseline commit | Not applicable: cross-library resource snapshot, not a before/after study |
| Execution time | Retained in environment metadata; not the report identity |

This is a draft under the [revision-binding rules](../reporting.md#bind-reports-to-changes-and-measured-commits).
The retained metadata does not establish a measured Rust commit. Neither the
current HEAD nor the later commit carrying this report can fill that gap without
verification of the measured source. The original date-named raw artifact path
remains unchanged. The separate exact/sparse correctness runs retain their own
source fingerprints.

## Evidence classification

Documentation clarification, 2026-09-20: under the shared
[reporting rules](../reporting.md), this run is a **resource snapshot** accompanied
by independent correctness checks. Its fixed backend order and three measured
samples do not meet the requirements for a new comparative performance study.
The selected rows below illustrate costs; the linked measurements contain all
25 workflows and their comparison scopes.

The source fingerprint below identifies the measured working-tree snapshot,
including the pipeline README as it existed during the run. Later documentation
edits do not change that historical identity. This report update adds no new
measurements and leaves retained artifacts unchanged.

## Sources and protocol

- [Pipeline summary](../results/rips-acceptance-2026-09-20/summary.json): 25 workflows,
  69 backend groups, one warmup plus three measured fresh processes per group;
  6 explicit Ripser approximation exclusions, no mismatches or timeouts.
- [Measurements](../results/rips-acceptance-2026-09-20/measurements.json) retain phase
  medians, minimum/maximum times, peak memory and timing-comparability scope.
- [Environment](../results/rips-acceptance-2026-09-20/environment.json) records
  commands, source/header/binary hashes, UTC time, limits and compiler details.
- [Exact correctness](../results/rips-acceptance-2026-09-20/exact/summary.json):
  588 fixtures, 1108 native comparisons, 68 documented exclusions and 39 protocol checks.
- [Sparse correctness](../results/rips-acceptance-2026-09-20/sparse/summary.json):
  156 topology comparisons, 153 persistence comparisons and 145 unmodified metric
  sampling checks under matched conventions.

The host is x86-64 Linux on AMD EPYC 7H12, Rust 1.92.0 and g++ 11.4.0, with the
[pinned upstream sources](../native/sources.json). The [pipeline protocol](../pipeline/README.md)
defines all timing and memory boundaries. No Python TDA library is used. All
measured processes ran serially, separately from compilation/testing workloads;
CPU frequency, scheduler activity and external host load were not controlled.

Pipeline source fingerprint: `9e6000c557ca183863336c9406cf39c89a9d058d7fa081cc4871f0631efdb7f2`.
Each correctness tool hashes its own source set; its fingerprint is recorded
separately. Generated binaries/build caches remain under the local `target/` tree.
Exact fixture files are retained beside the metadata. Large raw result JSON and
compiler logs are losslessly compressed, preserving the original bytes:
[pipeline samples](../results/rips-acceptance-2026-09-20/results.json.gz),
[exact outputs](../results/rips-acceptance-2026-09-20/exact/results.json.gz),
[sparse outputs](../results/rips-acceptance-2026-09-20/sparse/results.json.gz),
[build log](../results/rips-acceptance-2026-09-20/build/build.log.gz).

## Selected end-to-end observations

Times are medians in milliseconds, including input validation/conversion,
construction where applicable, public computation and interval export. Explicit
paths include full construction of the requested skeleton. These small samples
are descriptive measurements, not stable ranking estimates.

| Workflow | Rust ms | GUDHI C++ ms | Ripser C++ ms | Rust maximum process RSS MiB |
| --- | ---: | ---: | ---: | ---: |
| Circle n=64, F2 H1 dense | 2.349 | 10.498 | 2.154 | 3.18 |
| Circle n=64, F2 H1 threshold | 0.147 | 0.293 | 0.175 | 1.35 |
| Circle n=64, explicit threshold complex | 1.638 | 0.299 | — | 2.86 |
| Nonmetric n=24, F3 through H2 | 8.879 | 4.772 | 1.002 | 3.45 |
| Nonmetric n=24, explicit through dimension 3 | 23.088 | 4.777 | — | 7.95 |
| Circle n=64, H1 cycles and cocycles | 17.361 | — | — | 3.20 |
| Bipartite graph plus isolates, n=10000 | 1.231 | 6.454 | 5.586 | 3.59 |
| Metric n=64, F3 sparse approximation | 9.074 | 6.060 | — | 3.39 |
| Metric n=64, approximate explicit complex | 11.482 | 2.916 | — | 4.82 |
| Metric n=64, approximate cycles and cocycles | 26.177 | — | — | 8.18 |
| Metric n=128, F3 sparse approximation | 17.774 | 8.112 | — | 4.06 |

A dash means unsupported or unequal requested work, not a zero time or a failed
correctness check. Ripser does not return an explicit complex; native adapters
also omit basis extraction. Native matrix inputs do not measure Rust's upper/square
conversion, point evaluation or exhaustive metric checking. Those references
are marked correctness-only in the raw measurements.

Peak RSS includes runtime, input/parser high-water marks, allocator retention,
intermediates and outputs. It is not live reducer allocation or a hard library
budget. The 10000-vertex sparse case supplies only 256 edges; this run is consistent
with the documented sparse path, but finite measurements alone do not prove an
asymptotic bound or cover arbitrary dense inputs of that size.

The data contains scheduler variation. For example, the n=64 assumed-metric
approximation takes 6.267–9.682 ms while the checked-metric workflow takes
7.095–9.474 ms. Their total medians happen to reverse order; that is not evidence
that exhaustive checking accelerates reduction. Its construction median increases
from 0.326 to 1.483 ms. Raw samples and phase definitions must accompany any later
performance comparison.

## Acceptance and next decisions

The [R1-R10 audit](../../docs/design/rips-acceptance.md) maps contracts to tests,
resource ownership and practical limits. Explicit construction and requested bases
are real implemented workflows with visible costs; they should be selected for
analyses that need their outputs. The default diagram-only path retains implicit
computation. The next optimization work should first profile generic reduction
and representative extraction on these retained fixtures. These timings alone do
not identify an internal data structure as the cause of a bottleneck.

[Local validation](../results/rips-acceptance-2026-09-20/validation/summary.json)
records 97 Rust tests, 5 rustdoc tests, 10 Markdown examples and 49 Python tests;
debug/release, Rust 1.91, strict lint/docs and all six packaged examples passed.
Package verification compared 120 source files with the workspace. Two real child
process probes also confirmed timeout and virtual-address-space failures are
reported distinctly; these intentional failures are not performance samples. Hosted Windows/macOS CI
for the eventual submitted revision remains a release gate; neither the workflow
file nor older remote CI certifies this unsubmitted working tree. No crate publish
or remote PR/merge is part of this local acceptance record.
