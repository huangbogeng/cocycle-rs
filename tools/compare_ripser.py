"""Independent development comparison; requires ripser==0.6.14 and NumPy.

Runs from any directory. Builds the Rust library and a small protocol adapter in
a temporary directory. Neither Python nor Ripser is required by cargo test.
"""

import importlib.metadata
import math
import platform
from pathlib import Path
import random
import subprocess
import tempfile

import numpy as np
from ripser import ripser


def main():
    repo = Path(__file__).resolve().parents[1]
    rng = random.Random(1729)
    comparisons = 0
    with tempfile.TemporaryDirectory(prefix="cocycle-reference-") as temp:
        target = Path(temp) / "target"
        executable = Path(temp) / ("diagram_dump.exe" if platform.system() == "Windows" else "diagram_dump")
        subprocess.run(
            ["cargo", "build", "--locked", "--offline", "--lib", "--target-dir", str(target)],
            cwd=repo, check=True,
        )
        subprocess.run(
            ["rustc", "--edition=2024", str(repo / "tools/diagram_dump.rs"),
             "--extern", f"cocycle={target / 'debug/libcocycle.rlib'}",
             "-L", f"dependency={target / 'debug/deps'}", "-o", str(executable)],
            check=True,
        )
        # Quarter-integer values are exactly representable in Ripser's float32
        # and Cocycle's float64, isolating persistence from distance rounding.
        for n in range(1, 9):
            for sample in range(8):
                matrix = np.zeros((n, n), dtype=np.float32)
                values = []
                for i in range(n):
                    for j in range(i):
                        value = rng.choice([0.0, 0.25, 0.5, 1.0, 2.0, 4.0])
                        matrix[i, j] = matrix[j, i] = value
                        values.append(value)
                diameter = max(values, default=0.0)
                for dimension in [0, 1]:
                    for cutoff in [None, 0.0, 0.5, 2.0]:
                        complete = cutoff is None or cutoff >= diameter
                        output = subprocess.check_output(
                            [str(executable), str(n), str(dimension),
                             "none" if cutoff is None else str(cutoff), *map(str, values)], text=True,
                        ).splitlines()
                        assert (output[0] == "complete") == complete
                        actual = []
                        for line in output[1:]:
                            dim, birth, kind, endpoint = line.split()
                            actual.append((int(dim), float(birth), kind, float(endpoint)))
                        expected = []
                        diagrams = ripser(
                            matrix, distance_matrix=True, maxdim=dimension, coeff=2,
                            thresh=math.inf if cutoff is None else cutoff,
                        )["dgms"]
                        for dim, bars in enumerate(diagrams):
                            for birth, death in bars:
                                if death <= birth:
                                    continue  # Public Cocycle diagrams omit zero-length bars.
                                if np.isfinite(death):
                                    expected.append((dim, float(birth), "F", float(death)))
                                else:
                                    kind, endpoint = ("E", 0.0) if complete else ("C", cutoff)
                                    expected.append((dim, float(birth), kind, endpoint))
                        assert sorted(actual) == sorted(expected), (n, sample, dimension, cutoff, actual, expected)
                        comparisons += 1
    print(f"PASS: {comparisons} diagram comparisons; seed=1729; n=1..8; F2; ordinary; edge-length")
    print(f"Python {platform.python_version()}; {platform.platform()}")
    for package in ["ripser", "numpy", "scipy"]:
        print(f"{package}=={importlib.metadata.version(package)}")


if __name__ == "__main__":
    main()
