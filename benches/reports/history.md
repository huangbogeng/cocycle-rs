# Implementation history: 2026-09-17

[Benchmarks](../README.md) / [Reports](README.md)

Historical evidence: external timings use GUDHI/Ripser Python bindings.
These results are not a native C++ performance baseline.

These are retained development measurements from before the repository cleanup.
They identify distinct source versions and protocols. The current algorithm is
implicit H1; the explicit implementation now lives in the test-only reference
module. Raw artifacts are preserved unchanged.

## Explicit reference baseline

[Environment](../results/gudhi-2026-09-17/environment.json): Linux x86_64,
Xeon Platinum 8352V, CPU 0, Rust 1.98.0, Python 3.12.12, GUDHI 3.13.0.
18 semantic and 13 performance fixtures passed 62 diagram comparisons. Times
below are milliseconds, with seven samples per performance case.

| Case | Cocycle explicit | GUDHI direct | GUDHI collapse |
| --- | ---: | ---: | ---: |
| Uniform distances, H0/H1, n=32 | 3.847 | 1.255 | 0.342 |
| Uniform distances, H0/H1, n=64 | 48.780 | 10.426 | 1.591 |
| Uniform distances, H0/H1, n=128 | 771.239 | 97.350 | 9.316 |
| Circle distances, H0/H1, n=128 | 2444.274 | 94.100 | 152.781 |
| Four-cluster distances, H0/H1, n=128 | 401.035 | 94.324 | 6.847 |
| Duplicate-point distances, H0/H1, n=128 | 403.439 | 98.152 | 7.382 |
| Equal distances, H0/H1, n=128 | 165.522 | 82.205 | 3.558 |
| Discrete nonmetric distances, H0/H1, n=64 | 111.795 | 12.449 | 10.846 |
| Uniform distances, H0/H1, n=128, T=0.2 | 2.303 | 1.516 | 0.946 |
| Euclidean points, H0/H1, n=128 | 773.108 | 98.168 | 9.170 |
| Uniform distances, H0 only, n=128 | 0.421 | 2.080 | 9.312 |
| Uniform distances, H0 only, n=512 | 8.298 | 39.866 | 480.584 |
| Uniform distances, H0 only, n=1024 | 36.436 | 190.283 | 3748.160 |

At uniform n=128, explicit Cocycle was about 7.9 times slower than GUDHI direct
and 82.8 times slower than collapse. Collapse reduced the simplex count from
349,632 to 710 for uniform input, but only to 341,631 for the circle. H0-only
comparisons include each public entry point's construction and wrapper costs.
They do not isolate native union-find performance.

The same uniform n=128 process memory, in KiB:

| Backend | RSS after input preparation | Peak RSS | HWM growth |
| --- | ---: | ---: | ---: |
| Cocycle | 1,344 | 56,804 | 55,460 |
| GUDHI direct | 40,788 | 62,988 | 22,200 |
| GUDHI collapse | 40,892 | 42,700 | 1,808 |

Smaller process peak does not imply smaller incremental algorithm memory across
language runtimes. The measurement-time manifest is retained because package
inclusion rules were narrowed after the measurement.

An earlier, unpinned Rust-only run is retained in [linux-x86_64.csv](../results/linux-x86_64.csv).
It used seven samples, one warmup, release optimization, and a shared host. Its
128-point uniform distance median was 798.529 ms. Do not combine it with the
CPU-pinned runs. Reusing explicit column buffers reduced separately measured
process peak RSS from 79,920 to 57,352 KiB without changing reduction order.

## Shared-precision three-library baseline

[Environment](../results/ripser-2026-09-17/environment.json): same CPU, Rust/Python
versions, GUDHI 3.13.0 and Ripser.py 0.6.14; seven samples. Distances and cutoffs
were quantized once to float32-exact values for every backend; Cocycle/GUDHI
retained f64 arithmetic. Direct point-cloud cases were excluded from this group.

18 semantic and 12 performance cases produced 88 passing comparisons. The two
Ripser empty-input cases were marked unsupported with actual output retained:
its dense API inferred one essential H0 from a zero-point buffer. The other
28 Ripser comparisons and all 60 GUDHI comparisons passed. Times are ms.

| Case | Cocycle explicit | GUDHI direct | GUDHI collapse | Ripser.py |
| --- | ---: | ---: | ---: | ---: |
| Uniform distances, H0/H1, n=32 | 4.013 | 1.404 | 0.338 | 0.293 |
| Uniform distances, H0/H1, n=64 | 55.790 | 10.830 | 2.061 | 1.320 |
| Uniform distances, H0/H1, n=128 | 776.283 | 97.314 | 9.199 | 5.838 |
| Circle distances, H0/H1, n=128 | 2215.058 | 87.965 | 146.637 | 31.908 |
| Four-cluster distances, H0/H1, n=128 | 403.745 | 96.033 | 6.347 | 4.432 |
| Duplicate-point distances, H0/H1, n=128 | 409.369 | 97.784 | 7.263 | 6.329 |
| Equal distances, H0/H1, n=128 | 157.508 | 80.123 | 3.464 | 3.869 |
| Nonmetric distances, H0/H1, n=64 | 112.676 | 12.734 | 10.980 | 11.103 |
| Uniform distances, H0/H1, n=128, quantized T≈0.2 | 2.311 | 1.363 | 0.856 | 0.874 |
| Uniform distances, H0 only, n=128 | 0.478 | 2.097 | 9.029 | 1.769 |
| Uniform distances, H0 only, n=512 | 8.343 | 40.148 | 479.753 | 37.648 |
| Uniform distances, H0 only, n=1024 | 36.960 | 191.348 | 3730.017 | 162.703 |

This baseline motivated the implicit algorithm. Its approximate 133-fold uniform
and 69-fold circle gap to Ripser.py describes this historical version only.
Python memory baselines include imported dependencies; raw RSS is not a measure
of reduced-matrix storage alone.

## Implicit H1 transition

[Shared-precision environment](../results/ripser-implicit-2026-09-17/environment.json):
identical 30 fixtures, CPU 0, seven samples and dependency versions. All 88 diagram
comparisons passed with the same two explicit Ripser exclusions. Old and new
columns below are separate runs on the same fixtures; times are ms.

| Case | Cocycle explicit | Cocycle implicit | Speedup | GUDHI direct | GUDHI collapse | Ripser.py |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| uniform_h1_32 | 4.013 | 0.093 | 43.3× | 1.259 | 0.338 | 0.286 |
| uniform_h1_64 | 55.790 | 0.551 | 101.3× | 10.393 | 1.576 | 1.305 |
| uniform_h1_128 | 776.283 | 2.684 | 289.2× | 97.619 | 9.716 | 5.818 |
| circle_h1_128 | 2215.058 | 11.096 | 199.6× | 87.329 | 145.443 | 31.925 |
| clusters_h1_128 | 403.745 | 2.629 | 153.6× | 96.908 | 6.408 | 4.386 |
| duplicates_h1_128 | 409.369 | 3.518 | 116.4× | 97.128 | 7.271 | 6.286 |
| equal_h1_128 | 157.508 | 1.730 | 91.0× | 82.488 | 3.491 | 3.860 |
| nonmetric_h1_64 | 112.676 | 4.026 | 28.0× | 12.473 | 11.012 | 11.031 |
| uniform_h1_128_cutoff | 2.311 | 0.641 | 3.6× | 1.364 | 1.103 | 0.885 |
| uniform_h0_128 | 0.478 | 0.430 | 1.1× | 2.014 | 8.955 | 1.792 |
| uniform_h0_512 | 8.343 | 8.398 | 1.0× | 45.624 | 482.537 | 36.298 |
| uniform_h0_1024 | 36.960 | 36.680 | 1.0× | 190.848 | 3802.105 | 164.936 |

Uniform n=128 Cocycle peak RSS fell from 56,848 to 3,412 KiB; circle peak fell from
56,716 to 4,908 KiB. These are process high-water marks, not exact allocated object
sizes. H0-only code did not change; small timing differences are not evidence of
an H0 optimization. These results do not establish a universal ranking against
all Ripser interfaces, dimensions, or larger inputs.

The separate [unquantized f64 run](../results/gudhi-implicit-2026-09-17/environment.json)
passed 31 fixtures and 62 GUDHI comparisons. Uniform n=128 medians were 2.685 ms
for distance input and 3.156 ms for points; circle distance input took 10.910 ms.
Fixtures matched the historical f64 suite exactly. Precomputed endpoints were
compared exactly; independent point norms used the declared tolerance.

## Diagnostic ablation

[Raw ablation record](../results/implicit-ablation-2026-09-17.json): three 128-point
fixtures, six cumulative stages, fresh release-test processes, five samples.
Reference comparisons ran after timing and RSS reads. These times include test
instrumentation and must not be combined with production measurements. The
explicit cohomology stage is different from the old explicit homology algorithm.

| Cumulative stage | Uniform ms | Circle ms | Equal-distance ms |
| --- | ---: | ---: | ---: |
| explicit | 2444.485 | 7062.858 | 217.257 |
| clearing | 66.146 | 91.950 | 49.984 |
| implicit | 22.652 | 35.451 | 11.683 |
| cone | 18.118 | 33.694 | 12.075 |
| apparent | 2.843 | 11.561 | 1.908 |
| emergent | 2.838 | 10.873 | 1.898 |

On uniform input, clearing reduced column additions from 161,719 to 82. Implicit
reconstruction replaced 1,018,762 retained reduced-matrix entries with 88 stored
non-diagonal transformation entries plus implicit unit diagonals. These entries
have different meanings and sizes. Apparent shortcuts reduced yielded cofacets
from 567,589 to 59,255, counting regeneration and shortcut search. All 8,001 H1
candidates on equal-distance input used zero-lifetime shortcuts.

The emergent stage did not reduce operation counts further on these three inputs;
its timing variation does not demonstrate another algorithmic gain. Only
original-column emergent shortcuts were implemented, not mid-reduction shortcuts
or omission of apparent-pair pivot entries.

## Historical correctness checks

The initial explicit version passed 49 Rust tests and five doctests, including
100 small independent-rank cases, 900 H0/reference comparisons, 1,024 column XOR
checks, 80 perturbation cases, and 512 Ripser diagram comparisons. The implicit
transition passed 59 Rust tests and five doctests; the bipartite regression brought
the count to 60. Local debug/release and Rust 1.91 runs passed at those stages.
They were local checks, not evidence of hosted cross-platform CI or publication.

Current test obligations live in [testing](../../docs/development/testing.md); new cleanup checks
are recorded separately in [project cleanup](project-cleanup.md).
