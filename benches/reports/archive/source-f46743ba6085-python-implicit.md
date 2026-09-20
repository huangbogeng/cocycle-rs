# Implicit H1 transition: source f46743ba6085

[Benchmarks](../../README.md) / [Archive](README.md)

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived Python-wrapper resource snapshot |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `f46743ba60852a1c95ae14b0e9637345bd2127297495bb2d3757d235a2a77449` |
| Protocol | [Historical wrapper protocol](../../python-wrapper-protocol.md) |
| Provenance | [Run metadata](../../results/ripser-implicit-2026-09-17/environment.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.
The explicit baseline column is source `73cf062beff141887ce8380d7c23774bc0f0579e6ca499c6e3f7f8b144d8ba54`;
see the [baseline report](source-73cf062beff1-python-shared-precision.md). Both commits
remain unbound. The f64 supplement uses the candidate fingerprint above.

[Full raw samples](../../results/ripser-implicit-2026-09-17/results.json) and
[summary CSV](../../results/ripser-implicit-2026-09-17/summary.csv) retain all measured outcomes.

## Retained observations

[Shared-precision environment](../../results/ripser-implicit-2026-09-17/environment.json):
the same 30 fixtures and dependency versions as the linked explicit
shared-precision baseline, with CPU 0 and seven samples. All 88 diagram
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

The separate [unquantized f64 run](../../results/gudhi-implicit-2026-09-17/environment.json)
passed 31 fixtures and 62 GUDHI comparisons. Uniform n=128 medians were 2.685 ms
for distance input and 3.156 ms for points; circle distance input took 10.910 ms.
Fixtures matched the historical f64 suite exactly. Precomputed endpoints were
compared exactly; independent point norms used the declared tolerance.
