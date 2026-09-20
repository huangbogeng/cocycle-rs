# Benchmark report template

[Benchmarks](README.md) / [Reporting rules](reporting.md)

Copy the sections below into a PR/commit-named file in `benches/reports/`, following
the [identity rules](reporting.md#bind-reports-to-changes-and-measured-commits).
Use a title such as "Rips pipeline: PR #N at SHA" or "Rips pipeline: commit SHA".
Adjust relative links and add the measured revision/PR to the report index. Replace instructions with recorded facts;
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
| Measured candidate commit | Full SHA and link; PR head or tested merge commit; explicitly unbound for a draft |
| Measured baseline commit | Full SHA and link for before/after evidence; otherwise not applicable |
| Harness revision | Full SHA/fingerprint when different from the measured candidate |
| Source fingerprint and dirty state | Link fingerprint scope and preserved source; a dirty base commit is not a measured commit |
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

Link immutable environment metadata, fixtures, all raw samples, validation
results, full summaries, build logs and measured source. Describe any lossless
compression. Record missing artifacts and dated interpretation corrections here;
never rewrite an old run to describe newer code.
