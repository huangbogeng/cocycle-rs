// Private native benchmark protocol; no external runtime or JSON dependency.
#pragma once
#include <algorithm>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <tuple>
#include <vector>

namespace native_bench {
struct Input {
  std::size_t n;
  int q;
  double cutoff, diameter = 0;
  std::vector<double> values;
  bool complete() const { return std::isnan(cutoff) || cutoff >= diameter; }
  double threshold() const {
    return std::isnan(cutoff) ? std::numeric_limits<double>::infinity() : cutoff;
  }
  struct Row {
    const double* data;
    double operator[](std::size_t j) const { return data[j]; }
  };
  std::size_t size() const { return n; }
  Row operator[](std::size_t i) const { return {values.data() + i * (i - 1) / 2}; }
};
inline std::uint64_t integer(std::istream& file) {
  unsigned char bytes[8];
  if (!file.read(reinterpret_cast<char*>(bytes), 8)) throw std::runtime_error("truncated fixture");
  std::uint64_t value = 0;
  for (int i = 0; i < 8; ++i) value |= std::uint64_t(bytes[i]) << (8 * i);
  return value;
}
inline double real(std::istream& file) {
  auto bits = integer(file);
  double value;
  static_assert(sizeof(value) == sizeof(bits));
  std::memcpy(&value, &bits, 8);
  return value;
}
inline Input read(const char* path) {
  std::ifstream file(path, std::ios::binary | std::ios::ate);
  auto length = file.tellg();
  file.seekg(0);
  char magic[8];
  if (!file.read(magic, 8) || std::memcmp(magic, "COCYCLE1", 8) || integer(file) != 0)
    throw std::runtime_error("expected COCYCLE1 precomputed distances");
  auto n = integer(file);
  integer(file); // Ambient dimension is metadata only for distance fixtures.
  auto q = integer(file);
  auto cutoff = real(file);
  if (n > std::uint64_t(std::numeric_limits<int>::max()) || q > 1)
    throw std::runtime_error("unsupported vertex count or dimension");
  if (!std::isnan(cutoff) && (!std::isfinite(cutoff) || cutoff < 0 || double(float(cutoff)) != cutoff))
    throw std::runtime_error("cutoff must be finite, nonnegative and float32-exact");
  auto count = n * (n ? n - 1 : 0) / 2;
  if (length < 48 || std::uint64_t(length) != 48 + count * 8)
    throw std::runtime_error("invalid condensed input size");
  Input input{std::size_t(n), int(q), cutoff, 0, {}};
  input.values.reserve(count);
  for (std::uint64_t i = 0; i < count; ++i) {
    double value = real(file);
    if (!std::isfinite(value) || value < 0 || double(float(value)) != value)
      throw std::runtime_error("distances must be finite, nonnegative and float32-exact");
    input.values.push_back(value == 0 ? 0 : value);
    input.diameter = std::max(input.diameter, value);
  }
  return input;
}
using Interval = std::tuple<int, double, char, double>;
inline void append(std::vector<Interval>& out, int dimension, double birth, double death,
                   const Input& input) {
  if (dimension > input.q || death <= birth) return;
  if (!std::isfinite(birth) || std::isnan(death)) throw std::runtime_error("invalid interval");
  char kind = std::isfinite(death) ? 'F' : input.complete() ? 'E' : 'C';
  out.emplace_back(dimension, birth == 0 ? 0 : birth, kind,
                   kind == 'F' ? death : kind == 'E' ? 0 : input.cutoff);
}
inline long memory(const std::string& field) {
  std::ifstream file("/proc/self/status");
  std::string line;
  while (std::getline(file, line))
    if (line.compare(0, field.size(), field) == 0) return std::stol(line.substr(field.size()));
  return -1;
}
inline void nullable(long value) { if (value < 0) std::cout << "null"; else std::cout << value; }
inline void output(const Input& input, const std::vector<Interval>& bars, double elapsed,
                   long rss, long hwm, long peak, const char* path) {
  std::cout << std::setprecision(17) << "{\"status\":\"completed\",\"elapsed_ms\":" << elapsed
            << ",\"execution_path\":\"" << path << "\",\"rss_before_kib\":";
  nullable(rss);
  std::cout << ",\"hwm_before_kib\":"; nullable(hwm);
  std::cout << ",\"peak_rss_kib\":"; nullable(peak);
  std::cout << ",\"coverage\":[";
  if (input.complete()) std::cout << "\"complete\",null]";
  else std::cout << "\"through\"," << input.cutoff << "]";
  std::cout << ",\"intervals\":[";
  bool first = true;
  for (auto [dim, birth, kind, end] : bars) {
    if (!first) std::cout << ',';
    first = false;
    std::cout << '[' << dim << ',' << birth << ",\"" << kind << "\"," << end << ']';
  }
  std::cout << "]}\n";
}
} // namespace native_bench
