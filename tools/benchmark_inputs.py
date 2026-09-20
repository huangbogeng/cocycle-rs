"""Shared deterministic fixtures and analytic expectations; standard library only."""

from dataclasses import dataclass
import math
import random
import struct

HEADER = struct.Struct("<8sQQQQd")


@dataclass
class Case:
    name: str
    n: int
    values: list
    q: int = 1
    cutoff: float | None = None
    mode: str = "distances"
    d: int = 2

    def write(self, path):
        path.write_bytes(
            HEADER.pack(b"COCYCLE1", int(self.mode == "points"), self.n, self.d,
                        self.q, math.nan if self.cutoff is None else self.cutoff)
            + struct.pack(f"<{len(self.values)}d", *self.values)
        )


def uniform(n):
    # Identical generator to benches/rips.rs; no library RNG-version dependency.
    state = 1729
    values = []
    for _ in range(2 * n):
        state = (state * 6364136223846793005 + 1) & ((1 << 64) - 1)
        values.append((state >> 11) / (1 << 53))
    return [values[i:i + 2] for i in range(0, len(values), 2)]


def distances(points):
    return [math.dist(points[i], points[j]) for i in range(len(points)) for j in range(i)]


def float32_case(case):
    """Quantize the filtration once for all backends, never only Ripser's input."""
    if case.mode != "distances":
        raise ValueError("float32 parity suite requires precomputed distances")

    def quantize(value):
        rounded = struct.unpack("<f", struct.pack("<f", value))[0]
        if not math.isfinite(rounded):
            raise ValueError("float32 quantization must remain finite")
        return rounded

    return Case(case.name, case.n, [quantize(v) for v in case.values], case.q,
                None if case.cutoff is None else quantize(case.cutoff), case.mode, case.d)


def semantic_cases():
    geometries = [
        ("empty", [], None), ("singleton", [[0., 0.]], None),
        ("duplicates", [[0., 0.], [0., 0.]], 0.),
        ("pair", [[0., 0.], [2., 0.]], None),
    ]
    square = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]]
    for suffix, cutoff in [("full", None), ("isolated", .5), ("cycle", 1.), ("filled", math.sqrt(2))]:
        geometries.append((f"square_{suffix}", square, cutoff))
    cases = [Case(f"check_{name}_h{q}", len(points), distances(points), q, cutoff)
             for name, points, cutoff in geometries for q in (0, 1)]
    cases += [Case(f"check_tetrahedron_h{q}", 4, [1.] * 6, q) for q in (0, 1)]
    return cases


def performance_cases(quick):
    sizes = [16, 32] if quick else [32, 64, 128]
    n = sizes[-1]
    cases = [Case(f"uniform_h1_{size}", size, distances(uniform(size))) for size in sizes]
    circle = [[math.cos(2 * math.pi * i / n), math.sin(2 * math.pi * i / n)] for i in range(n)]
    clusters = [[x / 10 + (i % 4), y / 10] for i, (x, y) in enumerate(uniform(n))]
    duplicates = (uniform(n // 2) * 2)[:n]
    for family, points in [("circle", circle), ("clusters", clusters), ("duplicates", duplicates)]:
        cases.append(Case(f"{family}_h1_{n}", n, distances(points)))
    cases.append(Case(f"equal_h1_{n}", n, [1.] * (n * (n - 1) // 2)))
    m = sizes[-2]
    rng = random.Random(1729)
    cases.append(Case(f"nonmetric_h1_{m}", m, [rng.randrange(1, 17) / 16 for _ in range(m * (m - 1) // 2)]))
    cases.append(Case(f"uniform_h1_{n}_cutoff", n, distances(uniform(n)), cutoff=.2))
    cases.append(Case(f"uniform_h1_{n}_points", n, [v for p in uniform(n) for v in p], mode="points"))
    for size in ([64] if quick else [128, 512, 1024]):
        cases.append(Case(f"uniform_h0_{size}", size, distances(uniform(size)), q=0))
    return cases



FAMILIES = ("uniform", "circle", "sphere8", "nonmetric", "grid", "bipartite")


def lcg(count):
    state = 1729
    for _ in range(count):
        state = (state * 6364136223846793005 + 1) & ((1 << 64) - 1)
        yield (state >> 11) / (1 << 53)


def make_case(family, n, cutoff=None):
    """Deterministic, explicit input families, quantized once after construction."""
    if n < 2:
        raise ValueError("scaling fixtures need at least two vertices")
    d = 2
    if family == "uniform":
        values = distances(uniform(n))
    elif family == "circle":
        points = [[math.cos(2 * math.pi * i / n), math.sin(2 * math.pi * i / n)] for i in range(n)]
        values = distances(points)
    elif family == "sphere8":
        # Normalized cube samples on S^7 in R^8, NOT uniform sphere samples.
        d = 8
        rng = iter(lcg(d * n))
        points = []
        for _ in range(n):
            row = [2 * next(rng) - 1 for _ in range(d)]
            norm = math.hypot(*row)
            points.append([x / norm for x in row])
        values = distances(points)
    elif family == "nonmetric":
        values = [(int(x * 64) + 1) / 64 for x in lcg(n * (n - 1) // 2)]
    elif family == "grid":
        side = math.isqrt(n)
        if side * side != n:
            raise ValueError("grid needs a square vertex count")
        values = distances([[i / (side - 1), j / (side - 1)]
                                 for i in range(side) for j in range(side)])
    elif family == "bipartite":
        # K_(a,b) at t=1; all remaining edges and the full simplex at t=2.
        half = n // 2
        values = [1. if (i < half) != (j < half) else 2. for i in range(n) for j in range(i)]
    else:
        raise ValueError("unknown family")
    suffix = "_cutoff" if cutoff is not None else ""
    return float32_case(Case(f"{family}_h1_{n}{suffix}", n, values, cutoff=cutoff, d=d))


def case_specs(quick=False):
    for family in FAMILIES:
        if family in ("uniform", "circle"):
            sizes = [16, 32] if quick else [128, 256, 512, 1024]
        elif family == "grid":
            sizes = [16, 36] if quick else [64, 144, 256]
        elif family == "bipartite":
            sizes = [16, 32] if quick else [64, 128, 256]
        else:
            sizes = [16, 32] if quick else [128, 256, 512]
        for n in sizes:
            for cutoff in ([None, 1.] if family == "bipartite" else [None]):
                yield family, n, cutoff


def bipartite_diagram(n, cutoff):
    """Independent analytic H0/H1 multiset; connected graph has E-V+1 cycles."""
    a, b = n // 2, n - n // 2
    if cutoff not in (None, 1.):
        raise ValueError("analytic fixture supports full range or cutoff=1")
    full = cutoff is None or n == 2
    intervals = [[0, 0., "F", 1.] for _ in range(n - 1)]
    intervals.append([0, 0., "E" if full else "C", 0. if full else 1.])
    intervals.extend([[1, 1., "F" if full else "C", 2. if full else 1.]
                     for _ in range((a - 1) * (b - 1))])
    return {"coverage": ["complete", None] if full else ["through", 1.], "intervals": intervals}
