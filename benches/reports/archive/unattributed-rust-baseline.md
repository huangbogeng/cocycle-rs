# Rust-only baseline: source attribution unavailable

[Benchmarks](../../README.md) / [Archive](README.md)

Evidence class: archived resource observation. Measured commit, PR and source
fingerprint were not recorded. This record cannot serve as a revision-bound
baseline or be promoted to one by assigning a later commit. The original CSV
is retained; missing provenance remains explicit.

## Retained observations

An earlier, unpinned Rust-only run is retained in [linux-x86_64.csv](../../results/linux-x86_64.csv).
It used seven samples, one warmup, release optimization, and a shared host. Its
128-point uniform distance median was 798.529 ms. Do not combine it with the
CPU-pinned runs. Reusing explicit column buffers reduced separately measured
process peak RSS from 79,920 to 57,352 KiB without changing reduction order.

The buffer-reuse memory observation above has no separately identified raw run
in the former narrative; it is historical context, not a reproducible comparison.
