# Benchmark reports

[Benchmarks](../README.md)

Formal reports bind to a measured full commit SHA and, when applicable, a PR.
Execution dates are metadata. Use the [reporting rules](../reporting.md) and
[report template](../report-template.md); a report's own commit does not identify
its measured code.

## Commit-bound reports

No retained report currently has a verified measured-commit binding. The source
checks in the [migration audit](archive/migration.md) did not establish one.
New formal reports use `pr-<number>-<head12>-<suite>.md` or
`commit-<sha12>-<suite>.md`, recording full SHAs and upstream pins inside.

## Native drafts

| Report | Measured source | Evidence class and scope |
| --- | --- | --- |
| [Complete Rips workflows](draft-9e6000c557ca-rips-pipeline.md) | `9e6000c557ca`; separate exact/sparse fingerprints in artifacts | Resource snapshot: 25 workflows, 69 backend groups, three measured processes; independent correctness suites |
| [H0/H1 native validation](draft-e1493a10ae91-native-h0h1.md) | `e1493a10ae91`; preserved source archive | Correctness/smoke validation and initial resource snapshots |

Both drafts lack measured-commit attribution. They describe their recorded source
snapshots and cannot establish performance for current HEAD or an eventual merge.
Promotion requires verified source mapping or a fresh committed run.

## Historical Python-wrapper evidence

The [source-indexed archive](archive/README.md) separates explicit and implicit
implementations, shared-precision baselines, scaling experiments and instrumented
ablation. Each record states its full fingerprint, evidence type and attribution
limits. Wrapper observations remain distinct from native C++ measurements.

The [historical wrapper protocol](../python-wrapper-protocol.md) is maintained
separately from experiment reports. The [unattributed Rust baseline](archive/unattributed-rust-baseline.md)
records the oldest CSV's missing source identity explicitly.

## Maintenance records

[Repository cleanup verification](archive/maintenance/source-977573dab74e-cleanup.md)
is a local maintenance record identified by source `977573dab74e`, not a performance
experiment. The [migration audit](archive/migration.md) maps removed document paths
to their replacements and records the provenance checks.

Raw artifacts stay under their original `../results/` paths with unchanged bytes,
source identities, fixtures and outcomes. Document migration does not rerun an
experiment, assign a new source version or upgrade its statistical evidence.
