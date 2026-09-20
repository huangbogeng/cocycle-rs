# Report migration and revision audit

[Benchmarks](../../README.md) / [Reports](../README.md) / [Archive](README.md)

This is a documentation/provenance audit, not a performance experiment. Its Git
search covers the six locally reachable commits listed below, ending at
`00e4541124dde5b14984cf488eb369932941b58a`. It makes no claim about unavailable
remote history. Execution dates are retained only in the original run metadata.

## Attribution checks

The native archive's 42 files reproduce the recorded source SHA-256
`e1493a10ae91a776a50c80c77395baa9b9fa0c7ed8d7892f30e56e6a23e1198f`.
Reconstruction follows its worker/controller hash order: manifests, sorted Rust
sources, sorted native files, then `benchmark_native.py`, `build_native.py` and
`benchmark_inputs.py`; each entry contributes its relative path, a NUL byte and
file contents. The [preserved archive](../../results/native-2026-09-20/source.tar.gz)
remains the reproducible identity of that run.

Every archived file was compared byte-for-byte with each listed Git tree. None
matches. Both the refactor commit `707ddaa12122d5d9ac99fa042d416cfeeaf26375` and
merge `00e4541124dde5b14984cf488eb369932941b58a` differ in these measured files:

- `src/persistence/rips/mod.rs`
- `tools/build_native.py`
- `tools/benchmark_inputs.py`

Earlier commits differ in additional files or lack the native workers. Therefore
the PR that introduced the report cannot be assigned as the measured revision.

Historical wrapper hashes were reconstructed using each revision's
`tools/benchmark_gudhi.py` input list: manifests, sorted Rust sources, controller,
fixture helper when present, driver and requirement files. Diagnostic hashes use
sorted Rust paths and bytes without the NUL separator, matching `profile_rips.py`.
The resulting hashes below match none of the recorded wrapper, cleanup or ablation
identities indexed in this archive. No commit mapping was inferred.

| Checked full commit | Wrapper source SHA-256 | Diagnostic Rust-only SHA-256 |
| --- | --- | --- |
| `00e4541124dde5b14984cf488eb369932941b58a` | `ea881852b5ff972f1bdae394a3612bfb24ed16628eb046c7e39688d253e170c8` | `afbf0062f695b223e314396b01f1e1546bc9a2ff3c9032ac94b7df91bf5f097a` |
| `707ddaa12122d5d9ac99fa042d416cfeeaf26375` | `ea881852b5ff972f1bdae394a3612bfb24ed16628eb046c7e39688d253e170c8` | `afbf0062f695b223e314396b01f1e1546bc9a2ff3c9032ac94b7df91bf5f097a` |
| `21081f451b0efdb5966560cb012d32dfa95c8d13` | `e10ff93df85b21b060aa22150f597a986b650ee333ca201c4dca71bbb771e6e9` | `9470c5a244239f5be0d8c1d889f2fb3baae2b3c6e1cb7119c4db7d39b49b7e6d` |
| `1588b1ccead46eeadcd2423a66423ea6e677b810` | `182c6364ef19de6b7998b26b8f9f676aba444e9b9e1b34204cf39011dee79cc3` | `9470c5a244239f5be0d8c1d889f2fb3baae2b3c6e1cb7119c4db7d39b49b7e6d` |
| `8cb37f5b7289bf346d5926a8f9ff8c7bed94d0ae` | `182c6364ef19de6b7998b26b8f9f676aba444e9b9e1b34204cf39011dee79cc3` | `9470c5a244239f5be0d8c1d889f2fb3baae2b3c6e1cb7119c4db7d39b49b7e6d` |
| `87fc6895c215048d4e9da2e0ad8dd5f99faba19e` | `182c6364ef19de6b7998b26b8f9f676aba444e9b9e1b34204cf39011dee79cc3` | `9470c5a244239f5be0d8c1d889f2fb3baae2b3c6e1cb7119c4db7d39b49b7e6d` |

The current pipeline snapshot was taken from an uncommitted implementation and
retains a distinct source fingerprint. No later report or documentation commit is
assigned to it. The oldest Rust CSV has no recorded source fingerprint at all.

## Document migration map

Paths in the first column are removed paths relative to `benches/reports/`.
The second column links their replacement; observations and numerical tables
remain associated with their original raw evidence.

| Former document | Replacement and classification |
| --- | --- |
| `native-validation-2026-09-20.md` | [Native H0/H1 draft](../draft-e1493a10ae91-native-h0h1.md) |
| `scaling.md` | [Source-bound wrapper scaling archive](source-be652a04dba1-python-scaling.md) |
| `scaling-threeway.md` | [Source-bound wrapper full-schedule archive](source-be652a04dba1-python-threeway.md) |
| `history.md`: explicit baseline | [Explicit source record](source-ba9de82639f0-python-explicit.md) |
| `history.md`: shared-precision baseline | [Shared-precision source record](source-73cf062beff1-python-shared-precision.md) |
| `history.md`: implicit transition | [Candidate and baseline source record](source-f46743ba6085-python-implicit.md) |
| `history.md`: ablation | [Instrumented source record](source-b5d8bdd421d5-h1-ablation.md) |
| `history.md`: earlier Rust baseline | [Unattributed observation](unattributed-rust-baseline.md) |
| `history.md`: correctness narrative | [Historical context in the archive index](README.md#historical-correctness-context) |
| `project-cleanup.md` | [Maintenance archive](maintenance/source-977573dab74e-cleanup.md) |
| `python-protocol.md` | [Historical protocol outside report storage](../../python-wrapper-protocol.md) |

The complete raw evidence tree was hashed before and after migration: 1116 files,
with no added, removed or modified artifacts. Date-named raw directories are
retained as immutable evidence, not used as the identity of new reports. No Rust,
C++ or benchmark-controller behavior changed during this migration.

All 145 original table rows remain verbatim in the migrated records, including
failures and omissions. Source hygiene, local Markdown links and whitespace checks
passed after migration. No performance experiments were rerun.
