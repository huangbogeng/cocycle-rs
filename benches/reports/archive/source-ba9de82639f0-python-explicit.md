# Explicit reference baseline: source ba9de82639f0

[Benchmarks](../../README.md) / [Archive](README.md)

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived Python-wrapper resource snapshot |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `ba9de82639f0ee67c11090215a6c3b667c8ce173086d9a91798774d200079d53` |
| Protocol | [Historical wrapper protocol](../../python-wrapper-protocol.md) |
| Provenance | [Run metadata](../../results/gudhi-2026-09-17/environment.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.

[Full raw samples](../../results/gudhi-2026-09-17/results.json) and
[summary CSV](../../results/gudhi-2026-09-17/summary.csv) retain all measured outcomes.

## Retained observations

[Environment](../../results/gudhi-2026-09-17/environment.json): Linux x86_64,
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
