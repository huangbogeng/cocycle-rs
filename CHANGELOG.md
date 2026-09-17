# Changelog

## 0.1.0 (unreleased)

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
