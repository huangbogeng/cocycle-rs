"""Isolated GUDHI Python reference; named to avoid shadowing the gudhi package."""

import importlib.metadata
import json
import math
import os
from pathlib import Path
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from distance_common import PINS, PROTOCOL, number, read_fixture


def main():
    fixture, metric, variant = sys.argv[1:]
    result = {'protocol_id': PROTOCOL, 'metric': metric, 'variant': variant,
              'backend': 'gudhi_python', 'correctness_reference_only': True}
    try:
        versions = {name: importlib.metadata.version(name) for name in PINS['gudhi_python']}
        result['versions'] = versions
        if versions != PINS['gudhi_python']:
            raise RuntimeError('GUDHI oracle environment differs from sources.json pins')
        import gudhi
        import numpy as np
        for name in ('PYTORCH', 'JAX', 'CUPY', 'TENSORFLOW'):
            os.environ['POT_BACKEND_DISABLE_' + name] = '1'
        from gudhi.wasserstein import wasserstein_distance
    except (ImportError, RuntimeError) as error:
        result.update(status='unavailable', error=str(error))
        print(json.dumps(result, allow_nan=False))
        return
    try:
        first, second = read_fixture(fixture)
        arrays = [np.asarray(points, dtype=np.float64).reshape((-1, 2)) for points in (first, second)]
        start = time.perf_counter()
        if metric == 'bottleneck':
            value = gudhi.bottleneck_distance(*arrays, e=0.0)
            result['settings'] = {'e': 0.0, 'backend': 'gudhi.bottleneck_distance'}
        elif metric in ('w1', 'w2'):
            order, norm = (1, math.inf) if metric == 'w1' else (2, 2)
            value = wasserstein_distance(*arrays, order=order, internal_p=norm,
                                         keep_essential_parts=True, enable_autodiff=False,
                                         matching=False)
            result['settings'] = {'order': order, 'internal_p': 'infinity' if metric == 'w1' else 2,
                                  'keep_essential_parts': True,
                                  'backend': 'gudhi.wasserstein/POT emd'}
        else:
            raise ValueError('unknown metric')
        result.update(status='completed', elapsed_ms=(time.perf_counter() - start) * 1000,
                      **number(float(value)))
    except (ValueError, OverflowError) as error:
        result.update(status='rejected', error=str(error))
    print(json.dumps(result, allow_nan=False))


if __name__ == '__main__':
    main()
