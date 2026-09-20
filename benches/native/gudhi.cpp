// GUDHI C++ public interfaces: explicit Rips, optional one-pass edge collapse, CAM.
#include "common.hpp"
#include <gudhi/Simplex_tree.h>
#include <gudhi/Rips_complex.h>
#include <gudhi/Persistent_cohomology.h>
#include <gudhi/Flag_complex_edge_collapser.h>

using Tree = Gudhi::Simplex_tree<>;
using CAM = Gudhi::persistent_cohomology::Persistent_cohomology<
    Tree, Gudhi::persistent_cohomology::Field_Zp>;

std::vector<native_bench::Interval> compute(const native_bench::Input& input, bool collapse) {
  Tree tree;
  if (collapse) {
    std::vector<std::tuple<int, int, double>> edges;
    for (std::size_t i = 0; i < input.n; ++i) {
      // Isolated vertices must survive graph collapse.
      tree.insert_simplex({int(i)}, 0);
      for (std::size_t j = 0; j < i; ++j)
        if (input[i][j] <= input.threshold()) edges.emplace_back(i, j, input[i][j]);
    }
    auto remaining = Gudhi::collapse::flag_complex_collapse_edges(std::move(edges));
    for (auto [u, v, value] : remaining) tree.insert_simplex({u, v}, value);
    tree.expansion(input.q + 1);
  } else {
    // A borrowed lower-triangle view; no square-matrix or Python conversion.
    Gudhi::rips_complex::Rips_complex<double> rips(input, input.threshold());
    rips.create_complex(tree, input.q + 1);
  }
  CAM persistence(tree, input.q >= tree.dimension());
  persistence.init_coefficients(2);
  persistence.compute_persistent_cohomology(0);
  std::vector<native_bench::Interval> pairs;
  for (const auto& pair : persistence.get_persistent_pairs()) {
    auto birth = std::get<0>(pair), death = std::get<1>(pair);
    native_bench::append(pairs, tree.dimension(birth), tree.filtration(birth), tree.filtration(death), input);
  }
  std::sort(pairs.begin(), pairs.end());
  return pairs;
}
int main(int argc, char** argv) {
  try {
    if (argc != 3 || (std::string(argv[2]) != "direct" && std::string(argv[2]) != "collapse"))
      throw std::runtime_error("usage: gudhi FIXTURE direct|collapse");
    auto input = native_bench::read(argv[1]);
    auto rss = native_bench::memory("VmRSS:"), hwm = native_bench::memory("VmHWM:");
    auto start = std::chrono::steady_clock::now();
    auto diagram = compute(input, std::string(argv[2]) == "collapse");
    auto elapsed = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
    auto peak = native_bench::memory("VmHWM:");
    native_bench::output(input, diagram, elapsed, rss, hwm, peak, argv[2]);
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
