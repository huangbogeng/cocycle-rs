# Shared-precision three-library baseline: source 73cf062beff1

[Benchmarks](../../README.md) / [Archive](README.md)

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived Python-wrapper resource snapshot |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `73cf062beff141887ce8380d7c23774bc0f0579e6ca499c6e3f7f8b144d8ba54` |
| Protocol | [Historical wrapper protocol](../../python-wrapper-protocol.md) |
| Provenance | [Run metadata](../../results/ripser-2026-09-17/environment.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.

[Full raw samples](../../results/ripser-2026-09-17/results.json) and
[summary CSV](../../results/ripser-2026-09-17/summary.csv) retain all measured outcomes.

## Retained observations

[Environment](../../results/ripser-2026-09-17/environment.json): the same CPU and Rust/Python versions as the
[explicit f64 baseline](source-ba9de82639f0-python-explicit.md), GUDHI 3.13.0 and Ripser.py 0.6.14; seven samples. Distances and cutoffs
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
