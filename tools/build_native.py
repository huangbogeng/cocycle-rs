"""Build pinned native benchmark workers; source downloads are an explicit setup step."""

import hashlib
import json
import os
from pathlib import Path
import shlex
import subprocess

REPO = Path(__file__).resolve().parents[1]
NATIVE = REPO / 'benches/native'
PINS = json.loads((NATIVE / 'sources.json').read_text())


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def checked_source(path, name):
    path = path.resolve()
    revision = subprocess.check_output(['git', '-C', str(path), 'rev-parse', 'HEAD'], text=True).strip()
    if revision != PINS[name]['revision']:
        raise ValueError(f'{name}: expected revision {PINS[name]["revision"]}, got {revision}')
    dirty = subprocess.check_output(['git', '-C', str(path), 'status', '--porcelain', '--untracked-files=all'], text=True)
    if dirty:
        raise ValueError(f'{name}: source checkout must be clean: {dirty[:500]}')
    return path


def instrument_ripser(source):
    """Capture exact numeric pairs, not rounded CLI text. No algorithm edits."""
    replacements = {
        'std::cout << "persistence intervals in dim 0:" << std::endl;': '(void)0;',
        'std::cout << "persistence intervals in dim " << dim << ":" << std::endl;': '(void)0;',
        'std::cout << " [0," << get_diameter(e) << ")" << std::endl;': 'native_bench::emit(0, 0, get_diameter(e));',
        'std::cout << " [0, )" << std::endl;': 'native_bench::emit(0, 0, std::numeric_limits<value_t>::infinity());',
        'std::cout << " [" << diameter << "," << death << ")" << std::endl;': 'native_bench::emit(dim, diameter, death);',
        'std::cout << " [" << diameter << ", )" << std::endl;': 'native_bench::emit(dim, diameter, std::numeric_limits<value_t>::infinity());',
    }
    for old, new in replacements.items():
        if source.count(old) != 1:
            raise ValueError(f'Ripser output adapter no longer matches pinned source: {old}')
        source = source.replace(old, new)
    return source


def source_hash():
    paths = [REPO / 'Cargo.toml', REPO / 'Cargo.lock', *sorted((REPO / 'src').rglob('*.rs')),
             *sorted(NATIVE.glob('*')), *(REPO / 'tools' / name for name in
               ('benchmark_native.py', 'build_native.py', 'benchmark_inputs.py'))]
    digest = hashlib.sha256()
    for path in paths:
        if path.is_file():
            digest.update(path.relative_to(REPO).as_posix().encode() + b'\0' + path.read_bytes())
    return digest.hexdigest()


def build(directory, gudhi_source, ripser_source, boost_include=None, cxx='c++'):
    initial_hash = source_hash()
    gudhi = checked_source(gudhi_source, 'gudhi')
    ripser = checked_source(ripser_source, 'ripser')
    original = ripser / 'ripser.cpp'
    if sha256(original) != PINS['ripser']['source_sha256']:
        raise ValueError('Ripser source bytes differ from pin')
    directory.mkdir()
    generated = directory / 'ripser_instrumented.cpp'
    generated.write_text(instrument_ripser(original.read_text()))
    commands = []

    def run(command):
        commands.append(command)
        with (directory / 'build.log').open('a') as log:
            log.write(shlex.join(command) + '\n')
            log.flush()
            subprocess.run(command, cwd=REPO, check=True, stdout=log, stderr=subprocess.STDOUT)

    target = directory / 'cargo'
    run(['cargo', 'build', '--release', '--locked', '--offline', '--lib', '--target-dir', str(target)])
    run(['rustc', '--edition=2024', '-C', 'opt-level=3', '-D', 'warnings',
         str(NATIVE / 'cocycle.rs'), '--extern', f'cocycle={target}/release/libcocycle.rlib',
         '-L', f'dependency={target}/release/deps', '-o', str(directory / 'cocycle')])
    flags = ['-std=c++17', '-O3', '-DNDEBUG', '-MMD', '-I', str(directory)]
    if boost_include is not None:
        flags += ['-I', str(boost_include.resolve())]
    for include in sorted(gudhi.glob('src/*/include')):
        flags += ['-I', str(include)]
    for backend in ('gudhi', 'ripser'):
        run([cxx, *flags, '-MF', str(directory / f'{backend}.d'),
             str(NATIVE / f'{backend}.cpp'), '-o', str(directory / backend)])
    # Record actual headers consumed, including locally supplied Boost. System
    # headers omitted by -MMD are identified by compiler and Boost version below.
    dependencies = {}
    for name in ('gudhi', 'ripser'):
        text = (directory / f'{name}.d').read_text().replace('\\\n', '')
        for filename in shlex.split(text.split(':', 1)[1]):
            path = Path(filename)
            dependencies[str(path)] = sha256(path)
    macros = subprocess.check_output([cxx, *flags, '-MF', str(directory / 'boost-version.d'),
                                      '-dM', '-E', '-x', 'c++', '-include',
                                      'boost/version.hpp', '/dev/null'], text=True)
    boost_version = next(line.split()[-1] for line in macros.splitlines()
                         if line.startswith('#define BOOST_LIB_VERSION '))
    if source_hash() != initial_hash:
        raise ValueError('benchmark sources changed during compilation; use a new run')
    metadata = {
        'pins': PINS, 'commands': commands, 'dependencies_sha256': dependencies,
        'instrumented_ripser_sha256': sha256(generated), 'boost_version': boost_version,
        'cxx': subprocess.check_output([cxx, '--version'], text=True),
        'rustc': subprocess.check_output(['rustc', '-Vv'], text=True),
        'cargo': subprocess.check_output(['cargo', '--version'], text=True).strip(),
        'rustflags': os.environ.get('RUSTFLAGS'),
        'binaries_sha256': {name: sha256(directory / name) for name in ('cocycle', 'gudhi', 'ripser')},
        'source_sha256': initial_hash,
    }
    (directory / 'build.json').write_text(json.dumps(metadata, indent=2) + '\n')
    return metadata
