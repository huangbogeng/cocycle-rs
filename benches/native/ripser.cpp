// Upstream Ripser with only its six barcode output statements replaced at build time.
// Each process runs ONCE: upstream's static enumerators retain their first engine.
#include "common.hpp"
namespace native_bench {
std::vector<Interval> pairs;
const Input* context; // Set once; a worker computes exactly one diagram.
void emit(int dim, double birth, double death) { append(pairs, dim, birth, death, *context); }
}
#define main cocycle_unused_ripser_main
#include "ripser_instrumented.cpp"
#undef main

std::vector<native_bench::Interval> compute(compressed_lower_distance_matrix&& matrix,
                                           const native_bench::Input& input) {
  // Match upstream CLI dispatch: a supplied threshold uses its sparse matrix;
  // an unrestricted run uses dense access and computes the enclosing-radius bound.
  if (std::isnan(input.cutoff)) {
    value_t enclosing = std::numeric_limits<value_t>::infinity();
    for (std::size_t i = 0; i < matrix.size(); ++i) {
      value_t radius = 0;
      for (std::size_t j = 0; j < matrix.size(); ++j) radius = std::max(radius, matrix(i, j));
      enclosing = std::min(enclosing, radius);
    }
    ripser<compressed_lower_distance_matrix>(std::move(matrix), input.q, enclosing, 1, 2).compute_barcodes();
  } else {
    ripser<sparse_distance_matrix>(sparse_distance_matrix(matrix, value_t(input.cutoff)),
                                   input.q, value_t(input.cutoff), 1, 2).compute_barcodes();
  }
  auto result = std::move(native_bench::pairs);
  std::sort(result.begin(), result.end());
  return result;
}
int main(int argc, char** argv) {
  try {
    if (argc != 2) throw std::runtime_error("usage: ripser FIXTURE");
    auto input = native_bench::read(argv[1]);
    if (input.n < 2) {
      std::cout << "{\"status\":\"unsupported\",\"reason\":\"upstream compressed matrix adapter requires at least two vertices\"}\n";
      return 0;
    }
    std::vector<value_t> values(input.values.begin(), input.values.end());
    std::vector<double>().swap(input.values); // No retained f64 shadow input in RSS baseline.
    compressed_lower_distance_matrix matrix(std::move(values));
    native_bench::context = &input;
    auto rss = native_bench::memory("VmRSS:"), hwm = native_bench::memory("VmHWM:");
    auto start = std::chrono::steady_clock::now();
    auto diagram = compute(std::move(matrix), input);
    auto elapsed = std::chrono::duration<double, std::milli>(std::chrono::steady_clock::now() - start).count();
    auto peak = native_bench::memory("VmHWM:");
    native_bench::output(input, diagram, elapsed, rss, hwm, peak,
                         std::isnan(input.cutoff) ? "dense" : "sparse_threshold");
  } catch (const std::exception& error) { std::cerr << error.what() << '\n'; return 1; }
}
