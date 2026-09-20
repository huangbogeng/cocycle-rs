# Benchmark reports

[Benchmarks](../README.md)

Reports state which source and protocol were measured. New cross-library
performance evidence must use the [native C++ protocol](../protocol.md).

## Native C++ evidence

[Native validation: 2026-09-20](native-validation-2026-09-20.md) records the standard
baseline and small scaling checks, exact source pins, declared exclusions, and
retained raw outputs. It does not claim a complete large-input comparison.

## Historical Python-wrapper evidence

The following external-library results include Python binding and conversion
costs. They remain useful historical observations and correctness evidence, but
are not native GUDHI/Ripser C++ performance measurements.

| Report | Scope |
| --- | --- |
| [Same-size comparison](scaling-threeway.md) | Four paths on 23 scaling fixtures, including larger GUDHI cases |
| [Scaling and difficult inputs](scaling.md) | Bounded wrapper run, circle supplement and private work counters |
| [Implementation history](history.md) | Rust algorithm transitions, wrapper comparisons and ablation evidence |
| [Historical protocol](python-protocol.md) | Exact boundaries used by the retained Python-wrapper runs |

## Maintenance and raw evidence

[Project cleanup verification](project-cleanup.md) records the 2026-09-17
maintenance checks, not a fresh performance experiment. The original Rust-only
baseline is [linux-x86_64.csv](../results/linux-x86_64.csv).

Existing raw records remain under `../results/` with original bytes, source
hashes, fixture files, statuses and diagrams. Moving a report does not update its
measurements. Add native reports only after inspecting a new native run, and
label smoke validation separately from performance conclusions.
