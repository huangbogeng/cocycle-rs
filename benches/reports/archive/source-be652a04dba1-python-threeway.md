# Python-wrapper full-schedule comparison: source be652a04dba1

[Benchmarks](../../README.md) / [Reports](../README.md)

Historical evidence: external timings use GUDHI/Ripser Python bindings.
These results are not a native C++ performance baseline.

All 23 fixtures from the [scaling study](source-be652a04dba1-python-scaling.md) were rerun with both GUDHI
paths scheduled through 1024 vertices. Neither production algorithms nor benchmark
scripts changed for this experiment. Raw results remain separate from earlier
runs. These measurements predate the subsequent repository cleanup.

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived Python-wrapper resource snapshot |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `be652a04dba1cff0f6e5feaa7d9a9186400db4516beddc90e020a07be0d728f1` |
| Protocol | [Historical wrapper protocol](../../python-wrapper-protocol.md) |
| Provenance | [Run metadata](../../results/scaling-threeway-2026-09-17/environment.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.
Baseline commit: not applicable; cross-library snapshot. The circle supplement
and work diagnostics in the scaling report carry the same source fingerprint,
but their distinct protocols and sample sets remain separate.

## Protocol

- Cocycle, GUDHI 3.13.0 SimplexTree and one edge-collapse pass, and Ripser.py 0.6.14.
- Linux x86_64, Xeon Platinum 8352V, CPU 0; Rust 1.98.0 release.
- Identical float32-exact distances and cutoffs; Cocycle/GUDHI kernels remain f64.
  Ordinary F2 H0/H1, closed edge thresholds, zero-lifetime intervals omitted.
- Fresh workers, one warmup, three measured calls; medians reported. Input-layout
  preparation is outside timing; public API construction, computation, and output
  destruction are included.
- Uniform 180-second whole-worker budget and 16 GiB virtual-address limit. Timeout
  includes imports, warmup, samples, and output. It is not a single-call lower
  bound, and the address-space limit is not RSS.
- Source/dependency versions match the earlier scaling run. Samples from different
  experiments are not combined.

[Environment/status](../../results/scaling-threeway-2026-09-17/environment.json),
[raw results](../../results/scaling-threeway-2026-09-17/results.json),
[CSV](../../results/scaling-threeway-2026-09-17/summary.csv).

## Results and validation

All 92 workers were scheduled: 87 completed, five timed out, no process/protocol
errors, and no diagram mismatches. All four paths completed on 20 fixtures.
Every fixture's completed outputs passed comparison: 70 checks, including six
analytic bipartite checks.

Only GUDHI's two paths completed nonmetric512. They share GUDHI's persistence
implementation, so agreement does not establish Cocycle correctness or validation
across independent libraries on that fixture. On the other 22 fixtures, Cocycle
completed and matched at least one external library.

The [audit](../../results/scaling-threeway-2026-09-17/audit.json) confirmed all 23 fixtures
were byte-identical to the earlier run, source/script hashes matched at measurement,
all completed outputs contained three samples, and all available historical
diagrams matched. Times below are median **milliseconds**; no timing or speed ratio
is estimated for timeout entries.

| Case | Cocycle | GUDHI direct | GUDHI collapse | Ripser.py |
| --- | ---: | ---: | ---: | ---: |
| uniform_h1_128 | 2.702 | 97.736 | 9.186 | 5.861 |
| uniform_h1_256 | 11.833 | 1104.629 | 63.993 | 26.588 |
| uniform_h1_512 | 56.989 | 11640.556 | 479.665 | 122.681 |
| uniform_h1_1024 | 300.202 | Worker timeout | 3714.035 | 604.296 |
| circle_h1_128 | 11.112 | 88.459 | 146.888 | 31.640 |
| circle_h1_256 | 101.668 | 1019.055 | 1624.099 | 265.991 |
| circle_h1_512 | 1123.312 | 10671.993 | 21707.409 | 2296.934 |
| circle_h1_1024 | 14369.469 | Worker timeout | Worker timeout | 21309.816 |
| sphere8_h1_128 | 6.238 | 102.811 | 119.210 | 14.952 |
| sphere8_h1_256 | 35.576 | 1158.733 | 1269.814 | 79.752 |
| sphere8_h1_512 | 204.278 | 11928.831 | 12599.614 | 468.845 |
| nonmetric_h1_128 | 161.736 | 120.390 | 123.872 | 423.116 |
| nonmetric_h1_256 | 13797.575 | 1453.097 | 1412.129 | 38979.483 |
| nonmetric_h1_512 | Worker timeout | 16950.193 | 16899.903 | Worker timeout |
| grid_h1_64 | 0.698 | 10.806 | 1.530 | 1.649 |
| grid_h1_144 | 4.883 | 147.141 | 14.053 | 12.029 |
| grid_h1_256 | 19.547 | 1161.378 | 84.269 | 55.526 |
| bipartite_h1_64 | 2.123 | 9.772 | 1.571 | 7.348 |
| bipartite_h1_64_cutoff | 0.651 | 0.540 | 0.789 | 0.812 |
| bipartite_h1_128 | 15.728 | 86.962 | 8.250 | 57.225 |
| bipartite_h1_128_cutoff | 4.811 | 2.340 | 3.385 | 4.355 |
| bipartite_h1_256 | 111.237 | 913.235 | 36.848 | 387.072 |
| bipartite_h1_256_cutoff | 38.237 | 10.360 | 15.557 | 27.624 |

## Interpretation

- Uniform1024: Cocycle about 300 ms, Ripser.py 604 ms, GUDHI collapse 3714 ms;
  Cocycle is about 2.0 and 12.4 times faster, respectively. GUDHI direct timed out.
- Circle512: Cocycle 1.12 s, Ripser.py 2.30 s, GUDHI direct 10.67 s, collapse
  21.71 s. At 1024, both GUDHI paths timed out; Cocycle took 14.37 s and Ripser.py
  21.31 s. Timeout does not determine GUDHI's single-call duration.
- Nonmetric256: GUDHI collapse 1.41 s, Cocycle 13.80 s, Ripser.py 38.98 s.
  GUDHI is about 9.8 times faster than Cocycle. At 512, both GUDHI paths took
  about 16.9 s while the other libraries exhausted the worker budget.
- Bipartite256: full filtration favors GUDHI collapse (36.85 ms versus Cocycle
  111.24 ms); cutoff 1 favors GUDHI direct (10.36 ms versus Cocycle 38.24 ms).

The previous GUDHI coverage gap above 256 vertices is now replaced by actual
completed/timeout records. These observations apply to the listed interfaces,
fixtures, F2/H0/H1, and resource budget, not all GUDHI APIs or all TDA problems.

## Measured structural difference

A complete 512-vertex 2-skeleton has 22,370,048 simplices. After one edge-collapse
pass and expansion, uniform input retains 3,094, but the circle retains 22,239,743.
These are worker-reported counts, not inferred from timings. This explains why
collapse reduces subsequent work very differently across input families.

Memory fields record process peak RSS, including Python runtime/dependencies;
cross-language RSS ratios are not pure algorithm-storage ratios.

## Reproduction

```sh
python tools/benchmark_scaling.py --cpu 0 --samples 3 --timeout 180 \
  --address-space-mib 16384 --gudhi-max-n 1024 \
  --output /tmp/cocycle-scaling-threeway
```

Use a new output directory and a CPU available on the host. Larger limits are
explicit experiment arguments, not new tool defaults or core-library policies.
