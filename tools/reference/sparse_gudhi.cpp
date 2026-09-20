// Pinned GUDHI sparse edges, blockers and prime-field persistence.
// The generated header changes only the sampling call; see controller metadata.
#include "rips_common.hpp"
#include <gudhi/Persistent_cohomology.h>
#include "sparse_sampling.hpp"
#include "Sparse_rips_instrumented.h"
#include <gudhi/Simplex_tree.h>

int main(int argc, char** argv) {
  try {
    if (argc != 6) throw std::runtime_error("usage: sparse_gudhi FIXTURE EPSILON MIN_RADIUS START DIMENSION");
    auto input = reference::read(argv[1]);
    if (input.mode != "dense" || input.characteristic > 46337) throw std::runtime_error("unsupported input");
    double epsilon = std::stod(argv[2]), minimum = std::stod(argv[3]);
    sparse_reference::start_vertex = std::stoull(argv[4]);
    int dimension = std::stoi(argv[5]);
    using Tree = Gudhi::Simplex_tree<>;
    Tree tree;
    Gudhi::rips_complex::Sparse_rips_complex<double> rips(
        boost::irange<int>(0, int(input.n)),
        [&](int a, int b) { return input(a, b); }, epsilon, minimum, input.cutoff);
    rips.create_complex(tree, dimension);
    // Upstream inserts the graph even for dimension zero. Our explicit API
    // promises vertices only; prune it in the adapter and disclose this step.
    if (dimension == 0) tree.prune_above_dimension(0);
    std::vector<reference::Simplex> simplices;
    for (auto simplex : tree.complex_simplex_range()) {
      std::vector<int> vertices;
      for (auto v : tree.simplex_vertex_range(simplex)) vertices.push_back(v);
      std::sort(vertices.begin(), vertices.end());
      simplices.emplace_back(std::move(vertices), tree.filtration(simplex));
    }
    std::vector<reference::Edge> edges;
    for (auto simplex : tree.skeleton_simplex_range(1)) {
      if (tree.dimension(simplex) != 1) continue;
      std::vector<int> vertices;
      for (auto v : tree.simplex_vertex_range(simplex)) vertices.push_back(v);
      std::sort(vertices.begin(), vertices.end());
      edges.emplace_back(vertices[0], vertices[1], tree.filtration(simplex));
    }
    using CAM = Gudhi::persistent_cohomology::Persistent_cohomology<Tree, Gudhi::persistent_cohomology::Field_Zp>;
    std::vector<reference::Interval> intervals;
    if (dimension > input.q || dimension >= int(input.n)) {
      CAM persistence(tree, input.q >= tree.dimension());
      persistence.init_coefficients(input.characteristic);
      persistence.compute_persistent_cohomology(0.);
      for (auto pair : persistence.get_persistent_pairs()) {
        auto birth = std::get<0>(pair), death = std::get<1>(pair);
        reference::emit(intervals, input.q, tree.dimension(birth), tree.filtration(birth), tree.filtration(death));
      }
    }
    reference::output(std::move(edges), std::move(intervals), &simplices, input.characteristic);
    std::cout << "{\"permutation\":[";
    for (std::size_t i=0; i<sparse_reference::permutation.size(); ++i) {
      if (i) std::cout << ',';
      std::cout << sparse_reference::permutation[i];
    }
    std::cout << "],\"radii\":[";
    for (std::size_t i=0; i<sparse_reference::radii.size(); ++i) {
      if (i) std::cout << ',';
      if (std::isfinite(sparse_reference::radii[i])) std::cout << sparse_reference::radii[i];
      else std::cout << "null";
    }
    std::cout << "],\"metric_sampling_checked\":" << (sparse_reference::metric_sampling_checked ? "true" : "false") << "}\n";
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
