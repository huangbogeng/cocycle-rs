# Diagnostic ablation: source b5d8bdd421d5

[Benchmarks](../../README.md) / [Archive](README.md)

## Revision and evidence

| Item | Recorded identity |
| --- | --- |
| Evidence class | Archived instrumented diagnostics |
| Measured commit / PR | Unbound / not recorded; this is not evidence for the commit that later stored the report |
| Source fingerprint | `b5d8bdd421d5a56644b550993662d82d9607d7aaecabf9afc650359c8d65fd27` |
| Protocol | Release-test instrumentation; not production timings |
| Provenance | [Run metadata](../../results/implicit-ablation-2026-09-17.json) |
| Execution time | Retained in the linked metadata; not the report identifier |

This record predates the commit-binding policy. Missing commit, dirty-state and
source-snapshot metadata is not reconstructed from the reporting date. The
migration preserves observations and original artifacts; it is not a new run or
certification under the current comparative-study sampling rules.
This fingerprint hashes Rust source paths and bytes only; it does not identify
the controller or complete build environment. Do not compare it as if it had the
same scope as a production benchmark fingerprint.

## Retained observations

[Raw ablation record](../../results/implicit-ablation-2026-09-17.json): three 128-point
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
