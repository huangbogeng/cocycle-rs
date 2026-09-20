// One process per input: upstream static enumerators retain their first engine.
#include "rips_common.hpp"
namespace native_bench {
std::vector<reference::Interval> intervals;
int requested_dimension;
void emit(int dim, double birth, double death) {
  reference::emit(intervals, requested_dimension, dim, birth, death);
}
}
#define USE_COEFFICIENTS
#define main cocycle_unused_ripser_main
#include "ripser_instrumented.cpp"
#undef main

int main(int argc, char** argv) {
  try {
    if (argc != 2) throw std::runtime_error("usage: rips_ripser FIXTURE");
    auto input = reference::read(argv[1]);
    if (input.characteristic > 251) {
      std::cout << "{\"status\":\"unsupported\",\"reason\":\"pinned coefficient bit width supports primes at most 251\"}\n";
      return 0;
    }
    if (input.n < 2) {
      std::cout << "{\"status\":\"unsupported\",\"reason\":\"upstream engine requires at least two vertices in this adapter\"}\n";
      return 0;
    }
    native_bench::requested_dimension = input.q;
    std::vector<std::vector<index_diameter_t>> neighbors(input.n);
    if (input.mode == "dense") {
      for (std::size_t b = 0; b < input.n; ++b) {
        for (std::size_t a = 0; a < b; ++a) {
          double value = input(a, b);
          if (value <= input.cutoff) {
            neighbors[a].emplace_back(b, value_t(value));
            neighbors[b].emplace_back(a, value_t(value));
          }
        }
      }
    } else {
      for (auto [a, b, value] : input.edges) {
        if (value <= input.cutoff) {
          neighbors[a].emplace_back(b, value_t(value));
          neighbors[b].emplace_back(a, value_t(value));
        }
      }
    }
    std::vector<reference::Edge> edges;
    for (std::size_t a = 0; a < input.n; ++a) {
      std::sort(neighbors[a].begin(), neighbors[a].end());
      for (auto [b, value] : neighbors[a]) if (a < std::size_t(b)) edges.emplace_back(a, b, value);
    }
    sparse_distance_matrix matrix(std::move(neighbors), edges.size());
    ripser<sparse_distance_matrix>(std::move(matrix), input.q, value_t(input.cutoff), 1, coefficient_t(input.characteristic)).compute_barcodes();
    reference::output(std::move(edges), std::move(native_bench::intervals), nullptr, input.characteristic);
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
