# Native benchmark workers

[Benchmarks](../README.md) / [Protocol](../protocol.md)

The measured executables are Rust and C++ only. The Python controller uses the
standard library; do not install GUDHI or Ripser Python wheels for this suite.
Current resource and memory measurement support is Linux.

## Setup

Requirements: Rust 1.91+, Python 3.10+, a C++17 compiler, Git, and Boost headers.
The Rust crate gains no dependencies. CGAL, Eigen, TBB and Boost compiled
libraries are not needed for these Rips/CAM/collapse paths.

From the repository root, use fresh source directories and the exact revisions
in [sources.json](sources.json):

```sh
mkdir -p target/native-sources
git clone --filter=blob:none --no-checkout https://github.com/GUDHI/gudhi-devel.git target/native-sources/gudhi
git -C target/native-sources/gudhi sparse-checkout init --cone
git -C target/native-sources/gudhi sparse-checkout set src/common src/Simplex_tree src/Persistent_cohomology src/Rips_complex src/Collapse src/Subsampling src/Spatial_searching
git -C target/native-sources/gudhi fetch --depth=1 origin cba915e3ab8e1f5b1fe26eb44b407285f7af4e78
git -C target/native-sources/gudhi checkout --detach FETCH_HEAD
git clone https://github.com/Ripser/ripser.git target/native-sources/ripser
git -C target/native-sources/ripser checkout --detach 01add51ff64aaf40889483260cc5c3b7d0f2a1e7
```

These are explicit source baselines, not claims to use the latest release.
The builder requires clean checkouts and performs no downloads. Obtain Boost
headers through your system package manager (`libboost-dev` on Debian/Ubuntu),
or supply an extracted header directory with `--boost-include /path/to/include`.
The consumed header hashes and Boost version are recorded for each build.

## Run

```sh
python3 tools/benchmark_native.py --quick --samples 2 --output target/native-smoke
python3 tools/benchmark_native.py --samples 3 --output target/native-baseline
python3 tools/benchmark_native.py --suite scaling --samples 3 --timeout 60 --address-space-mib 2048 --output target/native-scaling
```

All output directories must be new. With locally extracted Boost headers, add
`--boost-include target/native-sources/boost/usr/include` if that is your actual
extraction path. `--gudhi-source`, `--ripser-source`, and `--cxx` accept alternative
local locations/compilers while keeping source pins enforced. `--cpu N` pins
workers to an available Linux CPU; inspect `os.sched_getaffinity(0)` first.

`--quick` reduces input sizes; it does not bypass correctness checks. The full
scaling suite can exceed its declared limits. Failed or limited runs return a
nonzero status and preserve completed results. See [validation rules](../protocol.md#validation-and-failure-handling).

## Files and artifacts

| File | Responsibility |
| --- | --- |
| [cocycle.rs](cocycle.rs) | Rust public API adapter |
| [gudhi.cpp](gudhi.cpp) | Direct and one-pass collapse GUDHI C++ paths |
| [ripser.cpp](ripser.cpp) | One-call upstream Ripser C++ adapter |
| [common.hpp](common.hpp) | Binary input, coverage normalization and JSON output for C++ workers |
| [sources.json](sources.json) | Authoritative upstream source pins |
| [build_native.py](../../tools/build_native.py) | Source verification, output-only Ripser adaptation and compilation |
| [benchmark_native.py](../../tools/benchmark_native.py) | Serial sampling, subprocess limits, validation and summaries |
| [benchmark_inputs.py](../../tools/benchmark_inputs.py) | Shared deterministic fixtures and analytic expectations |

Each output contains `environment.json`, `results.json`, `summary.csv`,
`manifest-at-measurement.toml`, and `fixtures/`. Its `build/` holds executables,
`build.log`, `build.json`, and `ripser_instrumented.cpp` with the upstream license.
Build metadata records precise commands, source revisions and hashes. Exploratory
Cargo products and executables need not be committed; retain build metadata and
the experiment's complete validation evidence when writing a report.

Run standard-library tool tests with
`python3 -m unittest discover -s tools -p 'test_*.py'` and check the Rust adapter
with `rustfmt --edition 2024 --check benches/native/cocycle.rs`. The native smoke
suite itself verifies compilation and mathematical output across all four paths.
