// Pinned GUDHI C++ graph construction and explicit flag persistence.
#include "rips_common.hpp"
#include <gudhi/Persistent_cohomology.h>
#include <gudhi/Rips_complex.h>
#include <gudhi/Simplex_tree.h>

int main(int argc, char** argv) {
  try {
    if (argc != 2) throw std::runtime_error("usage: rips_gudhi FIXTURE");
    auto input = reference::read(argv[1]);
    if (input.characteristic > 46337) {
      std::cout << "{\"status\":\"unsupported\",\"reason\":\"pinned Field_Zp supports primes at most 46337\"}\n";
      return 0;
    }
    using Tree = Gudhi::Simplex_tree<>;
    Tree tree;
    if (input.mode == "dense") {
      Gudhi::rips_complex::Rips_complex<double> rips(input, input.cutoff);
      rips.create_complex(tree, input.q + 1);
    } else {
      for (std::size_t v = 0; v < input.n; ++v) tree.insert_simplex({int(v)}, 0.);
      for (auto [a, b, value] : input.edges)
        if (value <= input.cutoff) tree.insert_simplex({int(a), int(b)}, value);
      tree.expansion(input.q + 1);
    }
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
    CAM persistence(tree, input.q >= tree.dimension());
    persistence.init_coefficients(input.characteristic);
    persistence.compute_persistent_cohomology(0.);
    std::vector<reference::Interval> intervals;
    for (auto pair : persistence.get_persistent_pairs()) {
      auto birth = std::get<0>(pair), death = std::get<1>(pair);
      reference::emit(intervals, input.q, tree.dimension(birth), tree.filtration(birth), tree.filtration(death));
    }
    reference::output(std::move(edges), std::move(intervals), &simplices, input.characteristic);
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
