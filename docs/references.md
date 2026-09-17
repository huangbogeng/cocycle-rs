# References

These sources identify definitions, theorems, and algorithmic ideas. They are not
runtime dependencies. Cocycle's engineering conventions are stated separately in
the [mathematical specification](mathematics.md).

## ZC05

Afra Zomorodian and Gunnar Carlsson. **Computing Persistent Homology**.
Discrete & Computational Geometry 33, 249–274 (2005).
[Publisher / DOI](https://doi.org/10.1007/s00454-004-1146-y).

Background for persistence over fields and interval classification. The project's
original source review checked bibliographic information and the abstract; the
specific reduction and pairing rules use the full-text source B21 below.

## B21

Ulrich Bauer. **Ripser: efficient computation of Vietoris–Rips persistence
barcodes**. Journal of Applied and Computational Topology (2021).
[arXiv v2](https://arxiv.org/abs/1908.02518v2),
[full text](https://arxiv.org/html/1908.02518v2),
[DOI](https://doi.org/10.1007/s41468-021-00071-5).

Relevant locations: §2 for Rips and filtration ordering; §3.1, Proposition 3.1 and
Algorithm 1 for reduction and pairing; §§3.2–3.5 for clearing, cohomology, implicit
matrices, and apparent pairs; §4 for H0/union-find. Cocycle independently implements
these invariants in Rust. Its internal order must satisfy each optimization's
conditions; citing Ripser does not establish correctness automatically.

## CSO13

Frédéric Chazal, Vin de Silva and Steve Oudot. **Persistence stability for
geometric complexes**.
[arXiv v3 (2013)](https://arxiv.org/abs/1207.3885v3),
[full text](https://arxiv.org/html/1207.3885v3).

Theorem 2.3 connects interleavings and bottleneck stability. Theorem 5.2 bounds
Rips diagrams using Gromov–Hausdorff distance. The fixed-vertex perturbation
argument in Cocycle first derives filtration inclusions, then applies Theorem 2.3.

## A20

Nieves Atienza, Rocío González-Díaz and Manuel Soriano-Trigueros.
**On the stability of persistent entropy and new summary functions for
Topological Data Analysis**. Pattern Recognition 107, 107509 (2020).
[arXiv v7](https://arxiv.org/abs/1803.08304v7),
[full text](https://arxiv.org/html/1803.08304v7),
[DOI](https://doi.org/10.1016/j.patcog.2020.107509).

Definition 3.1 uses base-two logarithms; §3.4 discusses infinite intervals.
Cocycle's natural logarithms, finite-only selection, and empty-result `None` are
explicit project conventions. The paper's title is not an unconditional stability
guarantee for every entropy variant.

## Architecture references

| Project | Source | Design lesson |
| --- | --- | --- |
| GUDHI | [Source modules](https://github.com/GUDHI/gudhi-devel/tree/master/src), [documentation](https://gudhi.inria.fr/python/latest/index.html) | Separate complex representation, filtration, persistence, and diagram tools |
| PHAT | [Paper, §2](https://www.geometrie.tugraz.at/kerber/kerber_papers/phat_jsc.pdf) | Separate boundary-column storage from reduction strategy |
| Ripser | [Repository](https://github.com/Ripser/ripser), B21 | Specialized Rips computations can use implicit representations |
| giotto-tda | [API modules](https://giotto-ai.github.io/gtda-docs/latest/modules/index.html) | TDA includes capabilities beyond a single persistence pipeline |

These are design references, not a statement that Cocycle implements their APIs.

## External validation tools

[GUDHI](https://gudhi.inria.fr/python/latest/) and
[Ripser.py](https://ripser.scikit-tda.org/en/latest/reference/stubs/ripser.ripser.html)
are optional development references. Pin versions and record precision, input,
coefficient field, scale, truncation, and interval conventions for each comparison.

The retained experiments use GUDHI 3.13.0 and Ripser.py 0.6.14. Ripser.py is a
Python binding/fork, distinct from the upstream C++ CLI. Its installed dense
input path converts distances to float32; the comparison suite uses shared
float32-exact inputs while retaining Cocycle/GUDHI f64 arithmetic. The
[online source](https://ripser.scikit-tda.org/en/latest/_modules/ripser/ripser.html)
helps locate this behavior but may describe a different version. See the
[benchmark protocol](../benches/README.md) and recorded artifacts for actual runs.
