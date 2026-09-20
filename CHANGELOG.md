# Changelog

## 0.1.0 (unreleased)

- Consolidate code conventions and change-specific verification rules. Enforce
  source hygiene and Python syntax in CI, validate Markdown reference links,
  and keep native build probe output in its build directory.
- Establish native GUDHI C++ and upstream Ripser C++ benchmark workers with pinned
  sources, shared fixtures, per-sample validation and process memory records.
  Separate the current protocol and native sources from historical wrapper reports.
- Organize documentation into usage guides, mathematical reference, development,
  design, and upstream research. Add task-based navigation and preserve recursive
  link checking, guide examples, and documentation packaging.
- Group Rips options and algorithms in a private module, extract shared
  connectivity, and distinguish edge positions from transformation-column
  positions. Preserve public entry points and update private profiling paths.
- Organize geometry, diagrams, and descriptors into consistent domain directories
  with private implementation files. Clarify internal filtration and result
  assembly names while preserving public paths and mathematical behavior.
- Maintenance: clarify H1 transformation indices and shortcut handling, share
  result normalization at the public entry point, and fix a strict Clippy warning
  in the test reference. Public APIs and mathematical conventions are unchanged.
- Repository organization: isolate the test-only reference algorithm and developer
  profiling, retain public API paths, and provide English user/contributor/math
  documentation with a local-link check in CI.

Initial pure Rust implementation, with no runtime dependencies. Requires Rust 1.91.

- Validated borrowed Euclidean point clouds and condensed symmetric dissimilarities.
- Ordinary Vietoris–Rips H0/H1 persistence over F2, with closed edge-length cutoffs.
- Independent union-find H0 and implicit Rips H1 persistent cohomology; retain
  explicit sparse-column boundary reduction as a test oracle.
- Checked combinatorial indexing, on-demand cofacets, H0 clearing, implicit
  change-of-basis columns, apparent/initial-column emergent shortcuts and a
  cone stopping bound that preserves public coverage and censoring semantics.
- Owned persistence diagrams retaining multiplicity and distinguishing finite,
  essential and right-censored intervals.
- Finite lifetime summaries, natural-log persistence entropy and Betti curves.
- Mathematical contracts, independent rank and stability checks, optional Ripser
  comparison, a runnable example and reproducible performance benchmarks.
- Bounded scaling experiments, analytic bipartite-filtration regression checks,
  explicit timeout/omission records and private workload diagnostics for H1.

H1 avoids constructing the full 2-skeleton; repeated enumeration and reduction
fill-in still limit scalability. Higher homology dimensions, representative
cycles, diagram distances, other filtrations and Polars bindings are future work.
