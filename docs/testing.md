# Testing and validation

Tests protect mathematical and public API contracts. Run commands and review
requirements are in [CONTRIBUTING.md](../CONTRIBUTING.md#verification); measurement
protocols belong in the [benchmark guide](../benches/README.md).

## Test layers

| Location | Purpose |
| --- | --- |
| `tests/contracts.rs` | Input shapes, values, options, interval ownership and coverage |
| `tests/euclidean.rs` | Distance numerics, invariance, and point/distance parity |
| `tests/rips.rs` | Hand calculations, independent ranks, stability, truncation, bipartite multiplicity |
| `tests/descriptors.rs` | Formula, endpoint, exclusion, overflow, and empty-result behavior |
| `src/persistence/reference/` | Independent explicit filtration and boundary reducer |
| `src/filtration/rips.rs` tests | Indexing, overflow, and independent cofacet enumeration |
| `src/persistence/rips_cohomology/tests.rs` | Seven optimization settings, duality, and difficult numeric cases |
| `tools/test_*.py` | External comparison and benchmark protocol behavior |

The two ignored profiling tests are deliberately invoked only by developer tools.
They do not represent missing ordinary regression coverage. Instrumented timings
are not production benchmarks.

## Hand-derived cases

Ordinary homology over F2, zero-lifetime intervals omitted. E denotes essential;
C(T) denotes alive at the cutoff, not a finite death.

| Case | Input | H0 | H1 |
| --- | --- | --- | --- |
| V01 | Empty input | Empty | Empty |
| V02 | One vertex | (0,E) | Empty |
| V03 | Two vertices, distance 2 | [0,2), (0,E) | Empty |
| V04 | Duplicate pair, distance 0 | (0,E) | Empty; zero pair omitted |
| V05 | Three vertices, all distances 1 | Two [0,1), (0,E) | Empty |
| V06 | Unit square | Three [0,1), (0,E) | [1,sqrt(2)) |
| V07 | Unit square, T=1 | Three [0,1), (0,C(1)) | (1,C(1)) |
| V08 | Unit square, T=0.5 | Four (0,C(0.5)) | Empty |
| V09 | Hand filtration: vertices 0, edges 1, face 2 | Two [0,1), (0,E) | [1,2) |
| V10 | Four equidistant vertices, 2-skeleton | Three [0,1), (0,E) | Empty; do not export artificial H2 |

V09 is a hand-built filtered complex, not a different cutoff on a three-point
Rips input. It verifies reduction independently of Rips construction. At T=1,
the square's H1 must count in a Betti query; queries above T must fail.
Complete bipartite fixtures additionally use the analytic multiplicities in
[mathematics section 10](mathematics.md#10-analytic-complete-bipartite-filtration).

## Independent invariants and properties

- Faces exist before cofaces, filtration values are monotone, and boundary squared
  is zero. Reduced nonempty columns have distinct pivots.
- Independent dense F2 elimination checks Betti numbers from boundary ranks.
  It must not call the persistence reducer to construct expected values.
- H0 union-find matches reference boundary reduction. Unpaired births are decided
  after reduction, not when an empty column is first encountered.
- Complete and truncated diagrams agree on Betti numbers within known coverage.
  Death beyond T must not be relabeled as death at T.
- Vertex permutations preserve diagram multisets. Translation and orthogonal
  transforms preserve Euclidean results; positive rescaling scales all endpoints.
- Small exhaustive matchings, including diagonal matches, check the fixed-vertex
  perturbation bound on complete diagrams.
- Input overflow, invalid numbers, unsupported dimensions, and query errors must
  remain distinguishable from valid empty results. Computed empty dimensions and
  uncomputed dimensions have different semantics.
- Descriptor tests check entropy ln(2) for two equal lifetimes, zero for one,
  `None` for none, and explicit exclusion of essential/censored intervals.

The cohomology tests compare seven settings: explicit cohomology, clearing,
implicit reconstruction, cone stopping, apparent only, emergent only, and both
shortcuts. They cover all 729 four-vertex distance assignments from {0,1,2},
random f64/nonmetric inputs, ties, adjacent floats, subnormals, huge scales, and
cutoff endpoints. Reversed-transpose matrix tests check pair and unpaired-index
mapping independently of production simplex indexing.

## External comparison

The optional Ripser tool checks 512 generated diagrams. GUDHI and Ripser.py workers
also compare full multisets, including multiplicities, dimensions, coverage, and
endpoint kinds. Infinity from another library is interpreted using the requested
range; it is not automatically essential.

Shared precomputed values isolate persistence from distance construction. When a
reference uses lower precision, quantize the same input for every backend instead
of hiding differences behind larger tolerances. Independent point-cloud norm
comparisons use an explicit tolerance. See [tools](../tools/README.md).

## CI and evidence

CI is configured to run formatting, Clippy, rustdoc, local documentation checks,
debug/release tests on Linux/macOS/Windows, Rust 1.91 tests/checks, package validation,
and external comparison smoke checks. Full performance runs are manual, with no
machine-dependent speed gates. Actual hosted results are available in
[GitHub Actions](https://github.com/huangbogeng/cocycle-rs/actions/workflows/ci.yml);
check the exact commit rather than inferring success from the workflow definition.

Dated performance evidence is indexed under [benchmarks](../benches/README.md).
Its source hashes identify the measured implementation. A directory refactor
changes those hashes even when behavior is preserved; old measurements must not
be relabeled as a new run. The [cleanup verification record](../benches/project-cleanup.md)
records checks for the present reorganization separately.
