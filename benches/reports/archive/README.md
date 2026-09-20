# Historical experiment archive

[Benchmarks](../../README.md) / [Reports](../README.md)

These source-identified records preserve historical observations. Measured commit
and PR attribution is unavailable; none is a current native performance baseline.
Execution dates remain in raw metadata. New experiments follow the
[commit-binding rules](../../reporting.md#bind-reports-to-changes-and-measured-commits).

| Record | Source identity | Evidence |
| --- | --- | --- |
| [Explicit baseline](source-ba9de82639f0-python-explicit.md) | `ba9de82639f0` | Python-wrapper resource snapshot |
| [Shared-precision baseline](source-73cf062beff1-python-shared-precision.md) | `73cf062beff1` | Python-wrapper resource snapshot |
| [Implicit transition](source-f46743ba6085-python-implicit.md) | `f46743ba6085`, baseline `73cf062beff1` | Historical before/after observations; commits unbound |
| [H1 ablation](source-b5d8bdd421d5-h1-ablation.md) | `b5d8bdd421d5` (Rust source only) | Instrumented diagnostics |
| [Scaling and circle supplement](source-be652a04dba1-python-scaling.md) | `be652a04dba1` | Wrapper snapshots and separate work counters |
| [Full-schedule comparison](source-be652a04dba1-python-threeway.md) | `be652a04dba1` | Wrapper snapshot with all paths scheduled |
| [Earlier Rust baseline](unattributed-rust-baseline.md) | Not recorded | Unattributed observation; unusable as a revision baseline |
| [Cleanup checks](maintenance/source-977573dab74e-cleanup.md) | `977573dab74e` | Maintenance verification, not performance |

The [wrapper protocol](../../python-wrapper-protocol.md) owns historical timing
boundaries; it is not an experiment report. The [migration audit](migration.md)
records why commit attribution was not inferred and where old documents moved.

## Historical correctness context

The following narrative predates version-bound reporting. Its stage counts are
preserved as historical context; no measured commit is inferred from them.

The initial explicit version passed 49 Rust tests and five doctests, including
100 small independent-rank cases, 900 H0/reference comparisons, 1,024 column XOR
checks, 80 perturbation cases, and 512 Ripser diagram comparisons. The implicit
transition passed 59 Rust tests and five doctests; the bipartite regression brought
the count to 60. Local debug/release and Rust 1.91 runs passed at those stages.
They were local checks, not evidence of hosted cross-platform CI or publication.

Current test obligations live in [testing](../../../docs/development/testing.md); new cleanup checks
are recorded separately in [project cleanup](maintenance/source-977573dab74e-cleanup.md).
