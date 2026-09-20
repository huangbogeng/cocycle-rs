# Roadmap

[Documentation](../README.md) / Design

This document records priorities, not a delivery schedule. Supported behavior is
listed in the [user guide](../guides/rips.md); completed user-visible work belongs in the
[changelog](../../CHANGELOG.md).

## Direction

Build a native Rust TDA kernel with GUDHI's C++ capabilities as a reference and
specialized Rips computation where appropriate. Prioritize coherent input and
result contracts, composable analysis operations, ownership, and resource
behavior. Runnable Rust examples demonstrate the kernel; language bindings and
application frameworks are not first-stage deliverables.

The [GUDHI C++ study](../research/gudhi-cpp.md) maps upstream capabilities and remaining
reading work. The [kernel design](kernel.md) proposes responsibility
boundaries and capability gates. These are design inputs, not a feature-parity
promise or a request to copy GUDHI's module layout.

## Current scope

The implemented core provides validated point clouds and dissimilarities,
ordinary Rips H0/H1 over F2, owned diagrams with explicit censoring, and basic
descriptors. H1 uses implicit cohomology; the explicit boundary implementation is
a test oracle. The crate has not been published.

## Next priorities

1. **Native comparison baseline.** Source boundaries are implemented as described
   in the [current architecture](../development/architecture.md).
   The [native harness](../../benches/native/README.md) now compares pinned GUDHI
   and upstream Ripser C++ workers. Extend controlled measurements to the full
   scaling suite and review limits and regressions before drawing performance
   conclusions. Historical Python-wrapper runs remain separate evidence.
2. **A complete analysis capability.** Select one concrete operation from the
   [proposed capability sequence](kernel.md#capability-sequence-and-acceptance-gates),
   with an input/output specification, an independent oracle, and a runnable Rust
   example. Diagram representations or scalar-line persistence are initial
   candidates; do not introduce the whole proposed module tree at once.
3. **Difficult H1 inputs.** Follow the scoped sequence below when addressing Rips
   scaling. The
   [same-size comparison](../../benches/reports/scaling-threeway.md) shows clear weaknesses
   on nonmetric and bipartite inputs. These optimizations are proposed, not
   implemented.

The first crates.io release remains a separate readiness gate: verify the name
and publisher, confirm hosted CI for the release commit, and inspect the package
using the [release procedure](../../CONTRIBUTING.md#release-procedure). This planning
work does not publish a crate or commit to a release date.

Performance changes must preserve pivot order, F2 parity, multiplicity, and public
coverage. Check each optimization separately against the reference and external
implementations; do not remove hard cases or use point count as a universal limit.

## H1 implementation sequence

Keep maintenance refactors separate from changes to enumeration, reduction, or
storage. The current baseline has a single result-normalization path, named
transformation columns, and an isolated original-column shortcut search. Its
test-only reference and seven optimization configurations remain the correctness
checks for subsequent work.

| Order | Work | Evidence required |
| --- | --- | --- |
| 1 | Reuse cofacet enumeration when the original-column shortcut fails, including empty columns | Bipartite cutoff cases retain every censored interval; tie cases still fall back when the earliest pivot is owned; measure candidate scans as well as yielded cofacets |
| 2 | Compress pending working-heap entries by F2 parity | Preserve pivot order and odd multiplicities; measure circle heap peaks and the cost on small/easy inputs |
| 3 | Reduce repeated cofacet generation during transformation-column reconstruction | Nonmetric cases retain complete diagrams; measure regeneration counts together with added cache/storage costs |

For each step, run the reference and external comparisons, then compare the same
fixtures under the same precision and timing boundaries before and after the
change. Include uniform, circle, nonmetric, and full/truncated bipartite inputs.
Keep timeouts and regressions in the results. Candidate-vertex scans are currently
absent from the cofacet counter; zero yielded cofacets does not mean zero work.

Historical benchmark records describe the source at measurement time. A replay
against their saved diagrams checks agreement with those outputs; it is neither
a fresh external-library run nor a new performance baseline. Record new source
hashes and fresh measurements before making a speed or memory claim.

The dense distance buffer and H0 edge sorting remain separate scaling limits.
Sparse inputs, higher dimensions, and a new mathematical capability each require
their own API and validation decision rather than being folded into this work.

## Feature admission

A new capability needs a mathematical specification, clear input/output contracts,
an independent correctness check, and a reasonable ownership/error model.
Avoid speculative backend registries, unused traits, placeholder modules, and
application-specific research features. Language bindings remain separate from
the kernel's Rust API and release process.
