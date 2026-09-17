# Visual identity

Original vector artwork for Cocycle, distributed under the repository's MIT license.
All assets are self-contained SVG: no scripts, external fonts, or external images.
Edit the SVG sources directly; no graphics build step is required.

| Asset | Use |
| --- | --- |
| [logo.svg](logo.svg) | Standalone square mark |
| [banner.svg](banner.svg) | README wordmark and project identity |
| [filtration.svg](filtration.svg) | Unit-square Rips filtration and H1 interval |

The mark is a triangulated annulus, with a central hole and a highlighted F2
1-cocycle. Its three marked edges meet each triangle in either zero or two edges,
so its coboundary is zero. It evaluates to one on the inner boundary cycle, so it
is not exact. This is a visual reference to the project's name, not a claim that
the public API returns representative cocycles.

The filtration illustration uses four corners of a unit square. At edge length 1
its boundary cycle appears; at sqrt(2), both diagonals and all triangular faces
enter. The crossing of drawn diagonals is not an additional vertex. The finite H1
interval is [1, sqrt(2)), with a closed birth and open death.

The palette uses deep ink (#102D32), mint (#9EDAC2), teal (#087D70), and a light
paper background (#F6F8F4). Explicit backgrounds preserve contrast in both light
and dark README themes. Each SVG contains a title and description for accessibility.
