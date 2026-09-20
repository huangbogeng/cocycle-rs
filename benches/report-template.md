# Benchmark report template

[Benchmarks](README.md) / [Reporting rules](reporting.md)

Use the sections below when creating a topic-based report such as
`benches/reports/rips-comparison.md`, following the
[identity rules](reporting.md#bind-reports-to-changes-and-measured-commits).
For a rerun, update the existing report in place, including its measured revisions,
results, evidence status and conclusions. Git history preserves older versions;
do not create another file or append an archive of old runs. Use a stable title
such as "Cocycle, GUDHI and Ripser: Rips comparison". Adjust relative links and
link the report from the index. Replace instructions with recorded facts;
write "not recorded" for missing evidence. This template contains no measurements.

## Question and conclusion

State the question, evidence class (correctness/smoke, resource snapshot or
comparative study), and the scoped result in a short paragraph. Include the main
limitation next to the conclusion. Identify whether this is cross-library or
before/after evidence; do not imply project-wide superiority.

## Measured revision and environment

| Item | Recorded value |
| --- | --- |
| Associated PR | Repository and PR link/number, or not applicable |
| Measured candidate commit | Full SHA and link; PR head or tested merge commit; must identify the actual measured code |
| Measured baseline commit | Full SHA and link for before/after evidence; otherwise not applicable |
| Harness revision | Full SHA/fingerprint when different from the measured candidate |
| Source fingerprint and dirty state | Link fingerprint scope and preserved source; dirty exploratory runs stay local |
| Suite and attempt ID | Stable run identity under the measured revision |
| UTC start/end | Execution metadata from artifacts |
| Native revisions and adapter instrumentation | Link pins, generated header/source hashes and binaries |
| Protocol identity and execution contract | Link the measured suite/revision |
| Machine and build | CPU, OS, compiler versions/commands, optimization flags |
| Controls and limits | Affinity, frequency/load controls, timeout and address-space cap; state uncontrolled factors |
| Reproduction | Exact commands, seeds, fixture generation, output directory |

## Workloads and comparison contract

Describe why the selected families answer the question. State input size/layout,
precision, filtration/cutoff, homology/construction dimensions, field, approximation
parameters/hypotheses and requested outputs. Explain each backend path and its
inclusion or exclusion. Mark diagram-only correctness references separately.
Link exact fixtures and validation outcomes, including independent expectations.

## Measurement and sampling

State timer start/stop, included preparation/conversions, export and cleanup,
retained result lifetime, memory sampling boundaries and units. Record planned
sample count, warmups, order and summary/spread calculation. Explain any deviation
from the plan; link all attempts and statuses.

## Results

Provide per-workload tables with validated medians, spread, measured count and
absolute process peak RSS, plus phase times when they answer the question. Define
ratio direction if used. Show unsupported, failed, timed-out and unequal-work
cases explicitly; do not assign synthetic timings. State planned/successful/
excluded/failed coverage and link the full matrix if displaying a selection.

## Interpretation and limits

Connect conclusions to specific rows. Discuss regressions, variation, unmatched
outputs, representation differences and unmeasured scope. Distinguish observed
resource behavior from a mathematical bound. Keep local verification, exact-commit
hosted CI and release status separate. Identify the next experiment only when
these results justify it.

## Evidence

State whether evidence is local-only, externally available or expired. When
available, link the external run or durable artifact containing environment metadata,
fixtures, all raw samples, validation results, full matrices and build logs.
Record the artifact checksum, measured SHA, run/attempt IDs, retention/expiry and
retrieval instructions. Do not copy the archive, logs or full generated output
into this repository. State missing or expired evidence explicitly; do not claim
public reproducibility for a local-only run.

For local-only evidence, give its ignored path as code, the available checksum
and reproduction commands. Do not add links that require an untracked local file
to exist. The report summarizes observations; it does not archive the original run.
