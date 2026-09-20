# Benchmark reports

[Benchmarks](../README.md)

This directory contains concise reports tied to a measured commit and, when
applicable, a PR. It stores neither raw results nor run archives. Follow the
[reporting rules](../reporting.md) and [template](../report-template.md).

There are currently no retained measurement reports. Historical measurements,
unbound working-tree drafts and their generated artifacts have been removed.
Fresh comparisons must identify the exact measured source and link evidence held
outside Git; a future run must not inherit old timings or success claims.

Use `pr-<number>-<head12>-<suite>.md` or `commit-<sha12>-<suite>.md`, recording full
SHAs, comparison scope, a compact result table and external artifact identity.
Routine CI results belong in the PR and CI job output. Commit a report only when
it records an enduring conclusion or decision.
