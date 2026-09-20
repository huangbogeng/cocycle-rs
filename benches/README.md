# Benchmarks

The formal cross-library comparison uses Cocycle's Rust API, GUDHI's C++ API,
and upstream Ripser C++. No Python TDA binding runs in a measured worker.
Python's standard library only prepares fixtures, starts executables, and checks
results. Follow the [native protocol](protocol.md) and
[build/run instructions](native/README.md).

## Layout

| Location | Responsibility |
| --- | --- |
| [rips.rs](rips.rs) | Dependency-free Rust-only benchmark, run with `cargo bench --locked --bench rips` |
| [native/](native/README.md) | Native worker sources, source pins, and setup commands |
| [protocol.md](protocol.md) | Current input, correctness, timing, memory, and evidence contract |
| [reports/](reports/README.md) | Dated conclusions, with native and historical wrapper evidence explicitly labeled |
| `results/` | Retained immutable fixtures, raw measurements, environments, and summaries |

The controller is [tools/benchmark_native.py](../tools/benchmark_native.py).
Fixture generation is shared by current and historical controllers; worker
implementations and measurement protocols remain separate.

## Compared implementations

| Backend ID | Native execution path |
| --- | --- |
| `cocycle` | Rust public Rips API: H0 union-find or implicit H1 cohomology |
| `gudhi_cpp` | C++ `Rips_complex` -> `Simplex_tree` expansion -> CAM persistence |
| `gudhi_collapse_cpp` | C++ threshold edges -> one flag edge collapse -> `Simplex_tree` expansion -> CAM |
| `ripser_cpp` | Upstream C++ Ripser; dense unrestricted or sparse threshold path |

GUDHI's integrated Ripser module is not the upstream Ripser backend. These paths
do not imply a comparison of every GUDHI engine or feature. Point-cloud distance
construction and diagram descriptors remain separate Rust benchmarks.

The retained 2026-09-17 GUDHI/Ripser.py studies are
[historical Python-wrapper measurements](reports/README.md). Their values and
source identities are preserved; they must not be relabeled as native results.

The [native validation record](reports/native-validation-2026-09-20.md) links the
first retained C++ baseline and small scaling results, including their exclusions.

## Evidence lifecycle

Use a new directory under `target/` for exploration. Every native run keeps its
source pins, compiler commands, binary/header hashes, environment, input fixtures,
raw sample outcomes, diagrams, and validated summary. Review the full result
before retaining a run under `results/` and adding a dated report.

Retain failures, exclusions, and resource limits. A smoke run validates the
harness; it does not establish broad performance superiority. Benchmarks are not
CI speed gates, and shared-runner timings are not a local performance baseline.
Raw experiments, C++ sources and binaries are not part of the Rust crate payload.
