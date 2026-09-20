# Python-wrapper scaling: source be652a04dba1

[Benchmarks](../../README.md) / [Reports](../README.md)

Historical evidence: external timings use GUDHI/Ripser Python bindings.
These results are not a native C++ performance baseline.

This report retains the first bounded scaling run, its circle supplement, and
work counters for the implicit H1 implementation. The later
[same-size comparison](source-be652a04dba1-python-threeway.md) schedules GUDHI at every size. Original
raw records are unchanged; these timings predate the repository cleanup.

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived Python-wrapper resource snapshot |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `be652a04dba1cff0f6e5feaa7d9a9186400db4516beddc90e020a07be0d728f1` |
| Protocol | [Historical wrapper protocol](../../python-wrapper-protocol.md) |
| Provenance | [Run metadata](../../results/scaling-2026-09-17/environment.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.
Baseline commit: not applicable; cross-library snapshot. The circle supplement
and work diagnostics in the scaling report carry the same source fingerprint,
but their distinct protocols and sample sets remain separate.

## Protocol and coverage

Linux x86_64, Xeon Platinum 8352V, CPU 0, Rust 1.98.0 release; GUDHI 3.13.0 and
Ripser.py 0.6.14. All paths use the same float32-exact dissimilarities; Cocycle and
GUDHI retain f64 kernels. Compare ordinary F2 H0/H1, closed edge scales, and omit
zero-lifetime intervals. Each fresh worker has one warmup and three timed calls.

Each worker has a 60-second total wall budget and 2 GiB virtual-address limit.
These cover startup, imports, warmup, calls, and output, not a single call's timing
or an RSS bound. GUDHI paths were predeclared only for n<=256. Larger entries say
"Not scheduled"; this is not evidence of failure or timeout.

23 fixtures cover uniform 2D/circle through 1024 points, normalized 8D cube samples
on S^7 and 64-level nonmetric matrices through 512, grids and bipartite inputs
through 256. Sphere samples are not uniform on S^7; nonmetric inputs differ from
the older 16-level baseline. Timed API boundaries follow the [protocol](../../python-wrapper-protocol.md).

76 backend results completed, four timed out, and 12 were not scheduled. No
process errors or diagram mismatches occurred. Initially 21/23 cases passed
60 comparisons, including six analytic checks. The circle supplement validated
a 22nd case. Nonmetric512 had no completed result in this run.

[Environment](../../results/scaling-2026-09-17/environment.json),
[raw results](../../results/scaling-2026-09-17/results.json),
[CSV](../../results/scaling-2026-09-17/summary.csv).

## All cases

Times are median milliseconds; Cocycle peak RSS is whole-process MiB, including
input, output, and allocator-retained memory. Cross-language RSS baselines differ.

| Case | Cocycle | GUDHI direct | GUDHI collapse | Ripser.py | Cocycle peak MiB |
| --- | ---: | ---: | ---: | ---: | ---: |
| uniform_h1_128 | 2.750 | 98.044 | 9.322 | 5.841 | 3.38 |
| uniform_h1_256 | 13.952 | 1145.574 | 64.647 | 25.953 | 6.02 |
| uniform_h1_512 | 56.547 | Not scheduled | Not scheduled | 121.010 | 17.05 |
| uniform_h1_1024 | 307.446 | Not scheduled | Not scheduled | 612.848 | 66.38 |
| circle_h1_128 | 11.037 | 88.407 | 152.565 | 33.084 | 4.66 |
| circle_h1_256 | 109.864 | 1037.606 | 1736.721 | 278.026 | 15.23 |
| circle_h1_512 | 1166.236 | Not scheduled | Not scheduled | 2449.535 | 89.59 |
| circle_h1_1024 | 14233.753† | Not scheduled | Not scheduled | Worker timeout | 602.44 |
| sphere8_h1_128 | 6.274 | 101.081 | 115.323 | 15.284 | 3.85 |
| sphere8_h1_256 | 33.288 | 1150.376 | 1217.204 | 79.359 | 8.73 |
| sphere8_h1_512 | 205.490 | Not scheduled | Not scheduled | 489.786 | 26.86 |
| nonmetric_h1_128 | 163.390 | 121.464 | 136.906 | 431.463 | 11.14 |
| nonmetric_h1_256 | 13775.448 | 1441.573 | 1436.955 | Worker timeout | 168.07 |
| nonmetric_h1_512 | Worker timeout | Not scheduled | Not scheduled | Worker timeout | — |
| grid_h1_64 | 0.624 | 10.755 | 1.530 | 1.650 | 2.74 |
| grid_h1_144 | 4.765 | 148.949 | 14.451 | 12.165 | 3.50 |
| grid_h1_256 | 20.313 | 1159.461 | 85.873 | 56.672 | 5.95 |
| bipartite_h1_64 | 2.391 | 9.473 | 1.581 | 7.303 | 2.69 |
| bipartite_h1_64_cutoff | 0.743 | 0.548 | 0.797 | 0.807 | 2.52 |
| bipartite_h1_128 | 17.172 | 85.644 | 8.675 | 52.225 | 3.97 |
| bipartite_h1_128_cutoff | 4.812 | 2.519 | 3.436 | 4.284 | 2.95 |
| bipartite_h1_256 | 112.235 | 931.702 | 36.432 | 382.207 | 8.52 |
| bipartite_h1_256_cutoff | 38.617 | 10.342 | 15.581 | 27.303 | 4.61 |

The dagger marks circle1024: initially unverified because Ripser timed out.
The table retains its three-sample median. A later identical-fixture supplement
matched the full diagram; original statuses remain unchanged, with a separate
[crosscheck record](../../results/scaling-circle-crosscheck-2026-09-17.json).

## Circle supplement

One warmup, one measured call, a 120-second worker budget, the same 2 GiB address
limit, and only Cocycle/Ripser scheduled. All four comparisons passed. Do not mix
these single samples with the main run's three-sample medians or speed ratios.

| Vertices | Cocycle single sample ms | Ripser.py single sample ms |
| --- | ---: | ---: |
| 128 | 11.892 | 32.476 |
| 256 | 106.818 | 268.638 |
| 512 | 1119.031 | 2297.719 |
| 1024 | 14058.308 | 21154.127 |

[Supplement environment](../../results/scaling-circle-single-2026-09-17/environment.json).
The 1024-point circle can complete a single call; the original timeout reflected
the repeated-measurement budget. Nonmetric512's timeout remained unresolved in
this protocol, not a proof that the size can never be computed.

## Work and memory diagnostics

[Main counters](../../results/scaling-work-2026-09-17.json) and
[circle1024 counters](../../results/scaling-circle-work-2026-09-17.json) come from the
same source's release-test build. All 13 diagnostic diagrams matched independently
validated production results. Large inputs did not invoke the cubic reference.
Counters are not production timings.

| Case | Yielded cofacets | Column additions | Stored off-diagonal entries | Largest transform | Peak coboundary heap entries |
| --- | ---: | ---: | ---: | ---: | ---: |
| uniform_h1_128 | 59,255 | 82 | 88 | 28 | 2,229 |
| uniform_h1_512 | 1,328,172 | 565 | 714 | 137 | 49,344 |
| uniform_h1_1024 | 6,998,528 | 1,681 | 2,012 | 297 | 203,973 |
| circle_h1_128 | 381,051 | 965 | 965 | 944 | 69,624 |
| circle_h1_256 | 3,036,328 | 3,781 | 3,781 | 3,739 | 540,432 |
| circle_h1_512 | 24,121,979 | 14,789 | 14,789 | 14,704 | 4,223,992 |
| sphere8_h1_512 | 7,227,567 | 3,618 | 6,482 | 360 | 425,494 |
| nonmetric_h1_128 | 5,579,366 | 4,200 | 10,648 | 444 | 476,405 |
| nonmetric_h1_256 | 470,850,467 | 32,667 | 98,551 | 1,351 | 10,443,181 |
| grid_h1_256 | 1,005,576 | 143 | 143 | 1 | 475 |
| bipartite_h1_256 | 12,306,554 | 16,002 | 16,002 | 126 | 32,008 |
| bipartite_h1_256_cutoff | 0 | 0 | 0 | 0 | 0 |
| circle_h1_1024 | 192,752,808 | 58,821 | 58,821 | 58,651 | 33,507,344 |

Cofacets count accepted triangles, including repeats and shortcut searches, not
rejected candidate vertices. Heap entries include duplicates awaiting parity
cancellation. Transformation counts exclude implicit diagonals. Stored edges are
within the internal stopping scale, not necessarily all input pairs.

1. **Circle working-heap growth.** At 1024 points, 58,821 stored off-diagonal
   transformation entries coexist with a 33,507,344-entry peak coboundary heap.
   At 16 bytes per Entry on this platform, the active entries alone occupy about
   511 MiB, consistent with roughly 603 MiB process RSS. A compression strategy
   still requires a separate correctness and performance experiment.
2. **Nonmetric regeneration.** Accepted cofacets grow from 5,579,366 at n=128 to
   470,850,467 at n=256; retained transformation entries grow from 10,648 to 98,551.
   At n=256 the heap exceeds ten million entries and GUDHI is about 9.6 times
   faster. Two points do not establish a universal complexity law.
3. **Triangle-free enumeration.** Bipartite256 with cutoff 1 yields no triangles
   or column additions but has 16,129 H1 intervals. Shortcut search and fallback
   each scan 254 non-endpoint candidates per cycle edge: 8,193,532 checks inferred
   from source and analytic structure, not measured by the cofacet counter.

Input structure, cutoff, and output size matter alongside point count. The
proposed optimization targets are parity compression, repeated cofacet generation,
and empty-cofacet scan reuse, with pivot-order and parity proofs plus independent
regressions required before adopting changes.

## Reproduction

```sh
python tools/benchmark_scaling.py --cpu 0 --samples 3 --timeout 60 \
  --address-space-mib 2048 --output /tmp/cocycle-scaling
python tools/benchmark_scaling.py --cpu 0 --families circle --samples 1 \
  --timeout 120 --address-space-mib 2048 --gudhi-max-n 0 \
  --output /tmp/cocycle-circle-single
python tools/profile_scaling.py --benchmark /tmp/cocycle-scaling \
  --cases uniform_h1_1024 circle_h1_512 nonmetric_h1_256 bipartite_h1_256_cutoff \
  --output /tmp/cocycle-work.json
```

Use new output paths and an available CPU. Profiling requires a newly verified
benchmark with the current source hash; after a refactor, historical records
cannot satisfy that check. Installation and counter definitions are in
[tools/README.md](../../../tools/README.md). No hosted CI was run for this experiment.
