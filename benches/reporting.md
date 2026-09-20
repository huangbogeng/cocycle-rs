# Performance reporting rules

[Benchmarks](README.md)

This page owns the rules for new performance claims and retained reports. Each
suite owns its execution contract: [H0/H1 native](protocol.md) or
[Rips pipeline](pipeline/README.md). Use the [report template](report-template.md)
for a new experiment. The source repository stores code and concise reports;
all generated run data lives outside Git.

## Classify the evidence

| Evidence class | Supports | Does not establish |
| --- | --- | --- |
| Correctness / smoke validation | Agreement on the exercised contracts; working adapters | Performance rankings or scaling claims |
| Resource snapshot | Observed time, phase costs and process memory on specified workloads | Stable rankings, universal speedups or asymptotic bounds |
| Comparative performance study | Scoped cross-library or before/after conclusions with repeated, comparable measurements | Behavior on unmeasured inputs, environments or revisions |

State the class and question before the numbers. New external performance
measurements use native GUDHI C++ and upstream Ripser C++. Python may orchestrate
processes; optional Python-wrapper tools do not qualify as native comparisons.
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
visible in that report. Use `target/benchmarks/commit-<sha12>/<suite>/run-<NNN>/`
locally and the same identity in external artifact storage. These directories
are never committed. Counter values distinguish attempts, not revisions.

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

Uncommitted exploratory runs remain local under `target/`, identified by their
source fingerprint and dirty state. They are not version-bound performance
reports and are not committed as draft archives. A report can cite a run only
after verifying the measured inputs against a commit or making a fresh committed
run. A later commit must not be assigned merely because it contains similar work.

Report only identities actually recorded. Controllers that do not emit all
required fields need additional run metadata in the external artifact. The
commit containing the report is distinct from the measured commit.

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

## Storage and evidence lifecycle

| Location | What belongs there | Retention |
| --- | --- | --- |
| Source Git repository | Library code, tests, benchmark workers/generators, protocols and concise PR/commit-bound reports | Maintained source history |
| Small test fixtures | Hand-maintained inputs required by a specific regression test | Reviewed as source; no generated benchmark corpus |
| `target/` | Local measurements, generated fixtures, environments, raw outputs, logs, profiles and archives | Disposable local work; never committed |
| GitHub Actions artifacts | CI comparison outputs, failure diagnostics and run metadata | Currently 14 days in this repository |
| Dedicated external artifact storage | Selected reproducible performance experiments needed beyond CI retention | Explicit durable URL, checksum and retention policy |

Do not commit raw results, generated fixture collections, build/test logs, full
machine/header inventories, binaries, caches or archives. Compressing them into
one file does not make them source. `benches/results/` is not an artifact store;
old data and reports have been removed, with no historical exemption.

GitHub's [workflow artifacts](https://docs.github.com/en/actions/tutorials/store-and-share-data)
are designed to store run outputs separately from source, with configurable
retention. They are not permanent evidence: record run ID, attempt, measured SHA,
artifact URL and expiry. A durable performance claim needs externally preserved
raw samples, fixtures, validation outcomes, environment/build metadata and hashes.
Publish an immutable external archive with a member checksum manifest if needed;
put only its identity, link, checksum and selected conclusions in the report.
No durable external experiment store is configured yet. Local-only or expired
evidence cannot be described as publicly reproducible.

The [Rust compiler performance project](https://github.com/rust-lang/rustc-perf)
separates per-commit collection and performance presentation into dedicated tools.
[airspeed velocity](https://asv.readthedocs.io/en/stable/using.html) likewise warns
that result data can grow large and needs an explicit storage plan. These are
examples of separating benchmark code from data management, not requirements to
add a database or service to this Rust library.

For every selected experiment:

1. Measure a known source revision into a fresh ignored local directory or CI run.
2. Validate all outcomes, including failures, exclusions and unfavorable cases.
3. Preserve the full run in external storage when a lasting report needs it.
4. Commit a concise report with the exact revisions, protocol, outcome, comparison
   scope and external evidence identity; do not copy the raw run into Git.

Use the [report template](report-template.md). Prefer the PR description and CI
job output for routine checks; a successful test run does not need a new document.
Only retain a repository report when it explains an enduring result or decision.
Do not duplicate full numeric matrices or logs in Markdown to bypass this policy.

After staging, run `python3 tools/check_artifacts.py`. CI repeats this check. It
rejects tracked result directories, cache/build paths, log files and archive/
binary suffixes; forced additions are checked too. The check does not infer
whether arbitrary JSON or prose is generated, so review still owns that boundary.
There are no grandfathered artifact blobs. `.gitignore` helps keep local outputs
out of the index but is not the enforcement mechanism.

Storage changes do not change a measurement's source identity. Documentation-only
changes need documentation checks, not a benchmark rerun. Missing evidence stays
explicit. Keep correctness, local package checks, hosted CI for an exact commit
and publication status distinct. Removing artifacts from a branch does not erase
objects already present in Git history; history rewriting is a separate operation.
