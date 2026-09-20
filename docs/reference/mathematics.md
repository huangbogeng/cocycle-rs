# Mathematical specification

[Documentation](../README.md) / Reference

This document defines the mathematical and numerical contracts shared by the
implementation, rustdoc, and tests. It distinguishes mathematical results from
Cocycle's engineering conventions. See [references](bibliography.md) for primary
sources and [testing](../development/testing.md) for independent checks.

## 1. Scope and notation

The current algorithms compute ordinary Vietoris–Rips persistent homology of
finite inputs over $\mathbb F_2$, returning H0 or H0/H1. Reduced homology, other
fields, representative cycles, zigzag, and multiparameter persistence are not
implemented. Nonnegative scales and zero H0 births are Rips-specific; the general
interval representation is not restricted to these dimensions or scales.

| Symbol | Meaning |
| --- | --- |
| $n,d$ | Vertex count and ambient dimension |
| $\delta_{ij}$ | Finite, nonnegative, symmetric dissimilarity; Euclidean distance for point clouds |
| $t,T$ | Filtration scale and optional cutoff |
| $q$ | Maximum requested homology dimension, currently 0 or 1 |
| $\sigma,f(\sigma)$ | Simplex and filtration value |
| $D,R$ | Boundary matrix and reduced matrix |
| $\mathcal D_k$ | Persistence diagram in dimension k, as a multiset |

## 2. Inputs and filtration

### 2.1 Distances

For $X=(x_0,\ldots,x_{n-1})$, with $x_i\in\mathbb R^d$,

$$\delta_{ij}=\lVert x_i-x_j\rVert_2.$$

Vertices are labeled observations. Duplicate coordinates remain distinct
vertices, so zero distance between different vertices is valid (a pseudometric).

**Engineering conventions.** Require $n\ge0$, $d>0$, row-major coordinates of
length $nd$, and finite `f64` coordinates. Dissimilarities must be finite and
nonnegative. Interpret `-0.0` as zero without modifying borrowed input; normalize
stored/output zero to positive zero. Reject unrepresentable distances instead of
substituting infinity. Scaled `hypot` avoids unnecessary square-sum overflow and
underflow. Reduction is combinatorially exact for the computed floating-point
filtration values, not exact real arithmetic.

Precomputed input is the strict lower triangle with an explicit vertex count:

$$
[\delta_{10},\delta_{20},\delta_{21},\ldots],\qquad
\operatorname{offset}(i,j)=i(i-1)/2+j\quad(i>j).
$$

The length is $n(n-1)/2$, the diagonal is implicitly zero, and size arithmetic is
checked. Zero and one vertex both need an empty buffer, distinguished by $n$.
The triangle inequality is not required: symmetric dissimilarities still define
a flag filtration, but their validation does not certify a metric.

### 2.2 Rips filtration

A simplex is a nonempty finite vertex set, with dimension $|\sigma|-1$. A
simplicial complex is closed under nonempty faces. A filtration satisfies
$\tau\subseteq\sigma\Rightarrow f(\tau)\le f(\sigma)$.

Cocycle uses edge-length scales:

$$
f(\{i\})=0,\qquad
f(\sigma)=\max_{i,j\in\sigma}\delta_{ij}\quad(|\sigma|\ge2),
$$

$$
K_t=\{\sigma:f(\sigma)\le t\}\quad(t\ge0),\qquad
K_t=\varnothing\quad(t<0).
$$

The threshold is closed. The parameter is neither a ball radius nor a squared
distance; compare [B21 §2](bibliography.md#b21). A face uses a subset of the vertex
pairs, so its maximum edge cannot be larger. This proves face closure and
$K_s\subseteq K_t$ for $s\le t$ without requiring the triangle inequality.

The explicit reference sorts by `(value, dimension, sorted_vertex_ids)` ascending.
The implicit path uses the descending combinatorial IDs in section 9. Both put
faces before cofaces. Tie refinements affect internal pair IDs but must preserve
the positive-lifetime diagram multiset. Sorting never merges nearby values using
an epsilon.

## 3. Chains, boundaries, and homology

Let $C_k(K;\mathbb F_2)$ have the k-simplices as a basis. A chain can be represented
by a set of simplex indices, with addition given by symmetric difference.

$$
\partial_k[v_0,\ldots,v_k]
=\sum_{r=0}^{k}[v_0,\ldots,\widehat v_r,\ldots,v_k]
\quad(k\ge1),\qquad \partial_0=0.
$$

Each codimension-two face occurs twice when deleting two vertices, and cancels
over F2. Hence $\partial_{k-1}\partial_k=0$, and

$$
Z_k=\ker\partial_k,\quad B_k=\operatorname{im}\partial_{k+1},\quad H_k=Z_k/B_k,
$$

$$
\beta_k=\dim C_k-\operatorname{rank}\partial_k-
\operatorname{rank}\partial_{k+1}.
$$

This rank formula supplies an oracle independent of persistence pairing.
Computing H1 needs triangles: edges determine $\ker\partial_1$, but triangle
boundaries determine which cycles are filled. A $(q+1)$-skeleton suffices for
H0 through Hq; its unpaired higher-dimensional chains are not full Rips output.
See [ZC05](bibliography.md#zc05) for the algebraic background.

## 4. Persistence and reference boundary reduction

Inclusions $K_s\hookrightarrow K_t$ induce homology maps. A finite filtration
over a field admits an interval-multiset description. Finite intervals use
$[b,d)$: present at birth, absent at death.

For ordered simplices $\sigma_0,\ldots,\sigma_{m-1}$, set $D_{ij}=1$ exactly when
$\sigma_i$ is a codimension-one face of $\sigma_j$. Nonzero entries satisfy
$i<j$ and $D^2=0$. `low` is the largest row index of a nonempty column; an empty
column has no pivot. Row zero is valid and must not be an empty sentinel.

```text
pivot_owner = empty map
R = empty columns
for j in increasing filtration order:
    column = boundary(sigma[j])
    while column is nonempty:
        i = low(column)
        if pivot_owner has no i:
            break
        column = xor(column, R[pivot_owner[i]])
    R[j] = column
    if column is nonempty:
        i = low(column)
        pivot_owner[i] = j
        record pair (i, j)

after all columns:
    unpaired_births = {i: R[i] is empty and i is not in pivot_owner}
```

Only earlier columns are added, so $R=DV$ for an invertible upper-triangular V,
preserving every prefix boundary-column space. Each cancellation strictly lowers
the pivot, ensuring termination; nonzero reduced columns have distinct pivots.
The pairing theorem then identifies `(low(R[j]), j)` with persistence pairs and
unpaired empty columns with surviving births. Equal rank alone is not a proof of
this pairing theorem: the direct source is [B21 Proposition 3.1 and Algorithm 1](bibliography.md#b21).

A pair `(i,j)` gives dimension $\dim\sigma_i$ and endpoints $f(\sigma_i),f(\sigma_j)$.
Determine unpaired births only after reduction, and output only dimensions through q.
Internal validation retains zero-length index pairs; public diagrams omit $b=d$
because $[b,b)$ is empty. There is no additional minimum-persistence filter or
representative-cycle output.

Sparse columns use ordered symmetric difference with reusable buffers. An empty
reduced column retains its index marker while its allocation may be reused: an
empty column cannot own a pivot and will not be read for later elimination.
This preserves order, pairs, and the $R=DV$ invariant.

## 5. Complete and truncated computations

Let $\Delta=\max_{i<j}\delta_{ij}$, with $\Delta=0$ if there are no vertex pairs.
At $t\ge\Delta$, the full flag complex of a nonempty input is one full simplex.
Its ordinary persistence has one essential H0 interval and no essential H1.
This is a property of these Rips inputs, not of arbitrary complexes.

`max_edge=None` requests the complete filtration. A finite $T\ge0$ includes all
necessary simplices with $f(\sigma)\le T$; $T\ge\Delta$ also establishes complete
coverage. A computation failure is an error, never a successful truncation.

| Endpoint | Interpretation |
| --- | --- |
| `Finite(d)` | Observed death; interval $[b,d)$ |
| `Essential` | Unpaired in a complete computation; death $+\infty$ |
| `RightCensored { through: T }` | Alive at T; full death is unknown |

In incomplete coverage, all surviving births, including H0, are conservatively
right-censored. The result retains computed dimensions and coverage. A single
optional death value cannot express these distinctions.

One could extend the truncated filtration constantly and write infinite intervals
for its survivors. Censoring instead describes uncertainty about the original
full filtration. Surviving through T is not dying at T.

## 6. H0 by union-find

All vertices are born at zero. Process edges by `(weight, vertex_ids)` and merge
connected components using a deterministic representative rule. An edge joining
two components kills one H0 class at its weight; an edge within a component does
not change H0. Higher-dimensional simplices do not merge additional components.

For complete input, selected edges form a minimum spanning tree; repeated points
allow zero-weight edges. At a cutoff, they form a minimum spanning forest with
surviving components. Test this path against boundary reduction. See
[B21 §4](bibliography.md#b21) for the H0 algorithm context.

## 7. Diagram descriptors

For one dimension, select finite positive lifetimes $\ell_i=d_i-b_i>0$. Let their
count be N and total $L=\sum_i\ell_i$.

| Descriptor | Definition | No finite positive lifetimes |
| --- | --- | --- |
| Finite count | $N$ | 0 |
| Total persistence | $L$, power one without a root | 0 |
| Maximum persistence | $\max_i\ell_i$ | `None` |
| Persistence entropy | $-\sum_i p_i\ln p_i$, $p_i=\ell_i/L$ | `None` |

[A20 Definition 3.1](bibliography.md#a20) uses base-two logarithms. Cocycle explicitly
uses natural logarithms (nats), without division by $\ln N$. One positive interval
has entropy zero; an empty collection has no normalized lifetime distribution.

The summary reports excluded essential and censored counts. It describes observed
finite lifetimes and does not estimate censored deaths or replace them with T.
Non-finite arithmetic results are errors. Total persistence uses compensated
summation. If an extreme probability underflows to zero, its $p\ln p$ contribution
is taken as zero, its limiting value.

Betti curves use all intervals:

$$\beta_k(t)=\#\{[b,d)\in\mathcal D_k:b\le t<d\}.$$

Essential intervals count for $t\ge b$. Censored intervals count throughout the
known range $b\le t\le T$; truncated diagrams reject queries beyond T. Caller-supplied
grid values must be finite, nonnegative, and strictly increasing. Omitting
zero-length pairs does not change Betti numbers at any scale.

## 8. Stability assumptions

For finite metric spaces, Rips diagrams satisfy

$$d_B(\mathcal D_k(X),\mathcal D_k(Y))\le2d_{GH}(X,Y),$$

by [CSO13 Theorem 5.2](bibliography.md#cso13). Bottleneck distance $d_B$ allows
matching to the diagonal; $d_{GH}$ is Gromov–Hausdorff distance. Do not apply this
statement directly to other filtrations, arbitrary nonmetric inputs, or censored
diagrams without establishing the required assumptions.

**Fixed-vertex derivation used in tests.** If
$\max_{ij}|\delta_{ij}-\delta'_{ij}|\le\varepsilon$, each simplex's maximum edge
changes by at most $\varepsilon$. The complete filtrations include into each
other's $\varepsilon$ shifts, giving an $\varepsilon$-interleaving. Applying
[CSO13 Theorem 2.3](bibliography.md#cso13) yields $d_B\le\varepsilon$.
This argument also holds for finite symmetric dissimilarities on the same vertices.

If corresponding Euclidean points move by at most $\eta$, the triangle inequality
bounds each edge change by $2\eta$, giving the corresponding $2\eta$ bound.
Numerical tests additionally allow explicit rounding tolerance. This does not
imply continuity of thresholded interval counts.

## 9. Implicit Rips persistent cohomology

This section specifies the production H1 path. The explicit boundary oracle
remains independent. Cohomology, clearing, implicit columns, and shortcut pairs
have separate conditions; see [B21 §§3.2–3.5 and §4](bibliography.md#b21).

### Numbering and order

For $a<b<c$,

$$
\operatorname{id}(a,b)=\binom b2+a,\qquad
\operatorname{id}(a,b,c)=\binom c3+\binom b2+a.
$$

Forward filtration order is maximum edge ascending, dimension ascending, and
combinatorial ID descending. Faces precede cofaces. Use descending comparison,
not unsigned negation. Exhaustive small-set tests check IDs and inversion.
Changed tie order may change index pairs but not the positive-lifetime multiset.

Cancel binomial denominators before checked multiplication. H1 requires
$\binom n3$ to fit the platform's `usize`, even when a cutoff retains few triangles.
Return `SizeOverflow` otherwise; 32-bit and 64-bit indexing limits differ.

### Coboundaries and pairing direction

The F2 coboundary of an edge consists of triangles containing it within the
cutoff. A transpose alone is insufficient: the small matrix oracle uses
$C=JD^\top J$, reversing both rows and columns. With S simplices, a C pair `(i,j)`
corresponds to the D pair `(S-1-j,S-1-i)`. Restore original dimensions and
filtration values, and also check unpaired indices and censored H1 classes.

Scan edges forward to obtain union-find merges and cycle births. Then process
edge coboundary columns in reverse, with the earliest forward triangle as pivot
(the highest reversed row). A paired edge/triangle yields an H1 birth/death;
an unpaired cycle-birth edge survives. H0 comes from union-find independently.
Clearing skips H0 merge edges: the adjacent-dimension pairing guarantees their
coboundaries reduce to zero. Tests disabling clearing still reduce them, verify
that result, and never mistake them for H1 births. Collect edges even after the
graph becomes connected.

### Implicit reconstruction invariant

Maintain $R=CV$. For each pivot, store
$V_j=e_j+\sum_{k\in A_j}e_k$ rather than retaining $R_j$. To cancel a pivot,
regenerate $Ce_j$ and all $Ce_k$, adding them to the working column while adding
their edge indices to the current transformation. The edges in $A_j$ precede j
in reverse computation order, so V stays unit upper triangular.

Repeated triangles and transformation indices cancel by F2 parity. Cancel all
copies of the candidate pivot before using it; ordinary set deduplication is
incorrect. Each elimination strictly lowers the pivot in reversed row order.
Stored transformations are internal machinery, not public representative cocycles.

### Clearing and shortcut pairs

H0 pairings and H1 clearing must use the same total order. An apparent pair
$(\sigma,\tau)$ requires sigma to be tau's latest facet and tau to be sigma's
earliest cofacet. Omitting its public interval also requires equal filtration
values. Do not indiscriminately remove columns that eventually reduce to zero.

The implemented emergent shortcut is restricted to original edge columns, before
any column addition. Cofacets are enumerated by descending ID, and their values
are at least the edge value. The first equal-valued triangle is therefore the
original column's earliest cofacet. If its pivot is unowned, ordinary reduction
would accept it immediately; retain $V_j=e_j$ without generating remaining rows.
If owned, fall back to full reduction, not the next equal-valued triangle.
Apparent-only mode additionally checks the latest-facet condition.

Zero-lifetime pairs are absent from public output but retain pivot/transform
entries for later elimination. Mid-reduction emergent shortcuts and omission of
apparent-pair pivot entries are not implemented. Test each optimization separately.

### Cone stopping bound

For a nonempty finite symmetric nonnegative input, define

$$r_*=\min_v\max_u\delta(v,u).$$

Choose a minimizing vertex $v_*$. At $t\ge r_*$, all edges to $v_*$ exist.
For every Rips simplex sigma, its union with $v_*$ is also a simplex. Thus the
complex is a cone: one ordinary H0 class and no positive-dimensional homology.
This derivation uses only the maximum-edge rule, not the triangle inequality.

A complete computation may stop at $r_*$; with cutoff T, use $\min(T,r_*)$.
Include all events exactly at the stop. Handle empty input separately and set
$r_*=0$ for a singleton.

Internal stopping does not replace public coverage. No user cutoff still means
`Complete`. If $T<\Delta$, retain `Through(T)` and its censoring convention even
when the cone is reached earlier; never substitute $r_*$ for T.

## 10. Analytic complete-bipartite filtration

Partition n vertices into groups of sizes $a,b>0$, with $a+b=n$. Assign cross-group
distance 1, within-group off-diagonal distance 2, and diagonal zero.

- For $0\le t<1$: vertices only; $\beta_0=n$, $\beta_1=0$.
- For $1\le t<2$: the connected triangle-free graph $K_{a,b}$;
  $\beta_1=E-V+1=ab-a-b+1=(a-1)(b-1)$.
- For $t\ge2$: the full simplex; $\beta_0=1$, $\beta_1=0$.

The complete diagram contains n−1 H0 intervals $[0,1)$ and one $[0,\infty)$;
H1 contains $(a-1)(b-1)$ copies of $[1,2)$. At T=1, H1 survivors and the remaining
H0 class are censored. Exception: a=b=1 has actual diameter 1, so T=1 is complete,
the H0 survivor is essential, and H1 is empty.

This independent derivation checks repeated endpoints, quadratic output
multiplicity, and triangle-free enumeration. The output itself can contain
$\Theta(n^2)$ intervals; do not attribute all their storage to reduction fill-in
or deduplicate intervals to improve measured cost.
