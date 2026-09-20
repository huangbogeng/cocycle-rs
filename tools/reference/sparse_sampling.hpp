// Reference-only deterministic sampling hook. No Rust outputs are inputs here.
#ifndef COCYCLE_SPARSE_SAMPLING_HPP
#define COCYCLE_SPARSE_SAMPLING_HPP
#include <gudhi/choose_n_farthest_points.h>
#include <algorithm>
#include <limits>
#include <stdexcept>
#include <vector>
namespace sparse_reference {
inline std::size_t start_vertex = 0;
inline std::vector<int> permutation;
inline std::vector<double> radii;
inline bool metric_sampling_checked = false;
template<class Distance, class Range, class PointOutput, class RadiusOutput>
void sample(Distance distance, const Range& points, std::size_t, std::size_t,
            PointOutput output, RadiusOutput radius_output) {
  const auto n = boost::size(points);
  std::vector<bool> selected(n, false);
  bool unique_positive_choices = true;
  // Recompute nearest distances from scratch each round: deliberately independent
  // of Rust's incremental nearest-distance array and upstream metric pruning.
  for (std::size_t round = 0; round < n; ++round) {
    std::size_t best = start_vertex;
    double farthest = std::numeric_limits<double>::infinity();
    if (round) {
      farthest = -1;
      bool tied = false;
      for (std::size_t v = 0; v < n; ++v) {
        if (selected[v]) continue;
        double nearest = std::numeric_limits<double>::infinity();
        for (auto p : permutation) nearest = std::min(nearest, distance(points[v], p));
        if (nearest > farthest) { farthest = nearest; best = v; tied = false; }
        else if (nearest == farthest) tied = true;
      }
      if (farthest > 0 && tied) unique_positive_choices = false;
    }
    selected[best] = true;
    permutation.push_back(points[best]); radii.push_back(farthest);
    *output++ = points[best]; *radius_output++ = farthest;
  }
  // Also exercise unmodified upstream metric sampling when tie policies cannot
  // affect its positive-radius prefix. Zero-radius tails do not affect topology.
  if (unique_positive_choices && n) {
    std::vector<int> upstream_order;
    std::vector<double> upstream_radii;
    Gudhi::subsampling::choose_n_farthest_points_metric(distance, points, n, start_vertex,
        std::back_inserter(upstream_order), std::back_inserter(upstream_radii));
    for (std::size_t i = 0; i < n && radii[i] > 0; ++i)
      if (upstream_order[i] != permutation[i] || upstream_radii[i] != radii[i])
        throw std::runtime_error("upstream metric sampling disagrees on unique choices");
    metric_sampling_checked = true;
  }
}
}
#endif
