# Roadmap

This document records priorities, not a delivery schedule. Supported behavior is
listed in the [user guide](guide.md); completed user-visible work belongs in the
[changelog](../CHANGELOG.md).

## Current scope

The implemented core provides validated point clouds and dissimilarities,
ordinary Rips H0/H1 over F2, owned diagrams with explicit censoring, and basic
descriptors. H1 uses implicit cohomology; the explicit boundary implementation is
a test oracle. The crate has not been published.

## Next priorities

1. **First crates.io release.** Verify the crates.io name and publisher, confirm
   hosted CI for the release commit, and review the release package using
   the [release procedure](../CONTRIBUTING.md#release-procedure).
2. **Difficult H1 inputs.** Investigate parity compression of working heaps,
   repeated cofacet generation during reconstruction, and redundant empty-cofacet
   scans. The [same-size comparison](../benches/scaling-threeway.md) shows clear
   weaknesses on nonmetric and bipartite inputs. These optimizations are proposed,
   not implemented.
3. **Select the next mathematical capability.** Candidates include higher Rips
   dimensions, diagram distances/representations, and other filtrations. Choose
   one concrete API and independent validation plan before expanding the crate.

Performance changes must preserve pivot order, F2 parity, multiplicity, and public
coverage. Check each optimization separately against the reference and external
implementations; do not remove hard cases or use point count as a universal limit.

## Feature admission

A new capability needs a mathematical specification, clear input/output contracts,
an independent correctness check, and a reasonable ownership/error model.
Avoid speculative backend registries, unused traits, placeholder modules, and
application-specific research features. Language bindings remain separate from
the kernel's Rust API and release process.
