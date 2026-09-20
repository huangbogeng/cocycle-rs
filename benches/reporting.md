# Performance reporting rules

[Benchmarks](README.md)

This page owns the rules for new performance claims and retained reports. Each
suite owns its execution contract: [H0/H1 native](protocol.md) or
[Rips pipeline](pipeline/README.md). Use the [report template](report-template.md)
for a new experiment. These rules do not retroactively change old measurements.

## Classify the evidence

| Evidence class | Supports | Does not establish |
| --- | --- | --- |
| Correctness / smoke validation | Agreement on the exercised contracts; working adapters | Performance rankings or scaling claims |
| Resource snapshot | Observed time, phase costs and process memory on specified workloads | Stable rankings, universal speedups or asymptotic bounds |
| Comparative performance study | Scoped cross-library or before/after conclusions with repeated, comparable measurements | Behavior on unmeasured inputs, environments or revisions |

State the class and question before the numbers. New external performance
measurements use native GUDHI C++ and upstream Ripser C++. Python may orchestrate
processes; Python TDA wrappers belong to separately labeled historical evidence.
Name the actual engine and adapter, not just the library: GUDHI direct expansion,
GUDHI edge collapse and GUDHI's integrated Ripser are different paths.

## Bind reports to changes and measured commits

A report is identified by the code it evaluates, not its execution date. The PR
is the review context; an immutable full commit SHA identifies the measured
revision. A PR number, branch name or current PR head alone is insufficient.
Record these fields explicitly:

| Identity | Required meaning |
| --- | --- |
| PR | Repository and PR URL/number, when applicable; otherwise state no associated PR |
| Measured candidate | Full Rust commit SHA and source fingerprint; name whether it is the PR head or a tested merge commit |
| Measured baseline | Full SHA for a before/after study; for cross-library-only evidence state not applicable and retain upstream pins |
| Harness | Worker/controller/protocol revision and fingerprint when distinct from the measured Rust revision |
| Run | Suite and unique attempt ID; UTC start/end are metadata, not the report key |

Use one canonical report per measured candidate and suite. New report filenames
are `pr-<number>-<head12>-<suite>.md` for a measured PR head, or
`commit-<sha12>-<suite>.md` without a PR or for a separately tested merge commit.
Here `head12`/`sha12` are at least twelve hexadecimal characters from the measured
commit, extended on collision; the report always records the full SHA. A PR report
must never silently follow a moving head. A new measured revision gets a new
report; repeated runs of the same revision get distinct attempt IDs and remain
visible in that report. Use `benches/results/commit-<sha12>/<suite>/run-<NNN>/`
for new retained raw runs. Counter values distinguish attempts, not revisions.

For a before/after report, record both candidate and baseline explicitly and link
each run's artifacts under its own measured revision. The PR target branch is not
a baseline identity. Rebases, squashes and merges produce new commit identities;
results for the previous head remain evidence for that head. Do not relabel them
as a measurement of the resulting merge commit.

Commit the implementation and harness before a formal measurement, verify that
the measured inputs are clean, then add the report/artifacts in a subsequent
commit. The report's own commit is not the measured commit. A documentation-only
follow-up can cite the prior measured revision without rerunning it, but must
not claim that the new revision was measured.

Uncommitted exploratory runs remain drafts keyed by their recorded source
fingerprint: `draft-<fingerprint12>-<suite>.md`. State that the measured commit is
unbound; record the base commit and dirty state only when known. A base commit is
not the identity of the modified source. Preserve a source snapshot/patch,
including untracked measured files, for new retained drafts. Promote a draft only
by verifying its preserved measured inputs against a commit, documenting that
mapping and retaining the original run identity, or by making a fresh committed
run. A later commit must not be assigned merely because it contains similar work.

Existing date-named raw artifacts remain at their original paths. Migrate report
filenames, titles and links to their verified revision identity. Native records
without a verified commit use the draft convention above. Historical wrapper
experiments and instrumented diagnostics belong in `reports/archive/` as
`source-<fingerprint12>-<suite>.md`; this `source-` prefix explicitly denotes a
fingerprint, not a Git commit. Split mixed-version reports into source-specific
records and name both sources for a before/after comparison.

Keep maintenance verification in `reports/archive/maintenance/` and protocol
specifications outside report storage. A historical observation lacking even a
source fingerprint must be labeled `unattributed-<scope>.md` in the archive and
cannot serve as a revision-bound baseline. Archive indexes and migration audits
are navigation/provenance documents, not experiment reports. Missing metadata
must remain explicit; moving a report never upgrades its evidence class.
Historical naming does not set the convention for new reports. These identity
fields are report requirements; controllers that do not emit them yet require
additional recorded provenance, not invented metadata.

## Establish comparability before measuring

Record the following contract for each workload family. Split rows when any
requested work differs; sharing a final diagram is not sufficient.

| Contract | Required information |
| --- | --- |
| Input | Fixture hash, generator/seed, vertex and edge counts, point dimension or matrix layout, metric/nonmetric assumptions |
| Filtration | Exact Rips, supplied flag graph or sparse approximation; edge-length convention, cutoff inclusivity and missing-edge meaning |
| Algebra and coverage | Homology dimensions, construction dimension, coefficient field, zero-length pair policy, essential/censored endpoints |
| Approximation | Epsilon, minimum radius, initial vertex/tie policy, sampling provenance, blockers, and checked/assumed hypotheses |
| Requested output | Diagram, retained explicit complex/incidence, cycles/cocycles and query scales, exported payload |
| Representation | Scalar widths, input conversions/copies, retained buffers and native structures |

Generate shared fixtures once. When a reference needs float32 inputs, quantize
once for all backends and retain the exact values; do not loosen tolerances to
hide filtration changes. Point-distance or numerical comparisons needing a
tolerance must document its independent justification.

Validate every measured output under the suite's contract, including interval
multiplicity, dimensions and endpoint meaning. Requested topology or bases need
additional structural/algebraic validation. A diagram-only native reference
cannot validate representative vectors or compete with basis extraction timings.
The same restriction applies to precomputed matrices versus point evaluation,
unchecked versus exhaustively checked metrics, and implicit diagrams versus
requested explicit complexes. Mark these references `correctness_reference_only`
where the suite supports that label; leave their comparison cells empty with an
explanation. Unsupported capabilities are explicit exclusions, never zero times.

## Identify the execution protocol

Link the exact protocol and record the worker/controller source fingerprint.
Use the machine-emitted protocol identity when available (`cocycle-native-v1`
for the H0/H1 suite). The pipeline currently records its description and source
hash rather than a separate version ID; preserve both. A prose label alone must
not imply a new protocol was executed.

| Boundary | H0/H1 native | Rips pipeline |
| --- | --- | --- |
| Work | Precomputed ordinary F2 H0/H1 | Matrix/graph/point, explicit, prime-field, representative and approximate workflows |
| Start | After validation and native input preparation | Before measured validation/conversion and construction |
| Ripser dense f64-to-f32 conversion | Before timer | Construction phase inside timer |
| End | Owned normalized intervals and algorithm cleanup, before JSON formatting | Interval payload export and workflow bookkeeping; final metrics transport excluded |
| Samples | Fresh processes; no warmup; shuffled backend order | Fresh processes; one discarded warmup; fixed backend order |
| Last memory reading | Before JSON serialization | After computation/export |

Both exclude fixture I/O and process startup from their internal times. Worker
wall-time limits cover more than that internal window. Record result lifetime
and intermediate destruction boundaries, not just a column called "runtime".
Do not pool samples or compute speedups across these protocols. A future change
to timing, precision, output or preparation requires a distinct protocol revision
and fresh measurements for all compared implementations.

End-to-end time answers a workflow question. Phase times answer narrower
questions only when their boundaries match. Public computation may include result
assembly; it is not automatically reducer-only time. Do not subtract independent
medians to estimate a missing phase or assume phase medians sum to the total.
Native execution avoids wrapper overhead; it does not imply zero copying.

## Plan samples and report uncertainty

Record workload selection, repetition count, warmup policy, backend order and
resource limits before the run. Include easy and difficult families, cutoffs,
fields and outputs relevant to the stated claim. For a before/after study, keep
the fixtures, native references, toolchain, hardware and protocol fixed and record
both Rust source identities. Retain regressions as well as improvements.

Resource snapshots may use the suite's small default sample count. For a new
comparative performance study, use at least ten independent measured processes
per compared cell as a project minimum, and increase repetitions or narrow the
claim when variation remains large. Ten is a reporting floor, not a statistical
guarantee. Warmups are retained but excluded from statistics. Publish all measured
samples, the median and a spread measure (at least minimum/maximum). Quantiles or
confidence intervals must state their calculation method and sample count.

Run workers serially without concurrent compilation or testing. Record CPU,
OS, compiler/build flags, affinity and any frequency/load controls; explicitly
state uncontrolled factors. The pipeline currently has fixed order and no
affinity control. Its default three samples support a resource snapshot. Stronger
ranking claims need an order-balanced experiment and a documented protocol
revision implementing that schedule; additional repetitions alone do not remove
order bias. CI smoke timings are not performance baselines.

Do not discard outliers or retry until a favorable run appears. Preserve all
attempts and explain an invalidated run before replacing it. A timeout is a
censored observation under a limit, not the limit itself as a measured runtime.
Crashes, mismatches and incomplete sample groups cannot be ranked. Show planned,
successful, excluded and failed coverage, with the reasons and original outcomes.

Label a ratio's direction: `reference_ms / cocycle_ms` is greater than one when
Cocycle is faster on that comparable row. Prefer per-workload results. An aggregate
needs a predeclared workload set, formula/weights and coverage; never silently drop
hard cases or combine incompatible protocols. A selected table must link the
complete matrix and explain the selection. Report enough digits to inspect the
data without implying precision beyond its variability.

## Report memory as measured

Keep prepared-input RSS, prepared-input high-water mark, absolute process peak,
and high-water growth distinct. State sampling boundaries and units (KiB/MiB).
High-water growth can be zero despite allocations. Process RSS includes runtime,
parser/input buffers, allocator retention, temporaries and retained outputs; it is
not live algorithm allocation. Different f32/f64 widths, trees and stored incidence
must accompany memory comparisons. An algorithm-memory claim needs a separate
allocation measurement with equivalent ownership boundaries.

Distinguish process timeout and `RLIMIT_AS` address-space caps from the library's
cooperative cancellation/work limits. Neither process peak RSS nor successful
completion of a fixture establishes a hard library memory budget.

## Retain and review evidence

Explore in a fresh `target/` directory. Retain reviewed runs and reports under
the [commit identity rules](#bind-reports-to-changes-and-measured-commits), and
add their revision/PR mapping to the [report index](reports/README.md). Keep fixtures, every sample/status, validated
summaries, environment, compiler commands/logs, consumed header and binary hashes,
dependency pins and the exact source fingerprint. Record the Git revision and
dirty state as well as the fingerprint; a commit alone does not identify an
uncommitted build. Formal studies require a preserved measured commit; draft
exploration requires a source snapshot/patch including untracked inputs. A hash
detects differences but cannot reconstruct source by itself.

Retained raw evidence is immutable. Lossless compression may preserve the
original bytes; document the encoding and links. Never replace source hashes or
rerun summaries in an old directory after changing code. Add a dated correction
to a report when its interpretation changes, keeping observed numbers and source
identity intact. Documentation-only changes need documentation checks, not a
benchmark rerun; if a fingerprint includes documentation, the old run still names
its old snapshot. Do not present it as validation of the new fingerprint.

Before accepting a report, check that its conclusion follows from comparable,
validated rows; its limits and failures are visible; and its links lead to the
full evidence. Missing metadata stays explicitly unknown. Separate correctness,
local build/package checks, hosted CI for an exact commit, and publication status.
Passing one does not establish the others. These are report review rules; current
tools enforce only the checks described in their individual protocols.
