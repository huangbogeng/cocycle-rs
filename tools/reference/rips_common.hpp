// Correctness protocol; all numeric fixtures and sources are recorded by the controller.
#ifndef COCYCLE_RIPS_REFERENCE_HPP
#define COCYCLE_RIPS_REFERENCE_HPP
#include <algorithm>
#include <cmath>
#include <cstdint>
#include <sstream>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <limits>
#include <stdexcept>
#include <string>
#include <tuple>
#include <utility>
#include <vector>

namespace reference {
using Edge = std::tuple<std::size_t, std::size_t, double>;
using Simplex = std::pair<std::vector<int>, double>;
using Interval = std::tuple<int, double, double>;
struct Input {
  std::string mode;
  std::size_t n;
  int q;
  std::uint32_t characteristic = 2;
  double cutoff;
  std::vector<double> values;
  std::vector<Edge> edges;
  std::size_t size() const { return n; }
  struct Row {
    const Input& input;
    std::size_t i;
    double operator[](std::size_t j) const { return input.values.at(i * (i - 1) / 2 + j); }
  };
  Row operator[](std::size_t i) const { return {*this, i}; }
  double operator()(std::size_t i, std::size_t j) const {
    if (i == j) return 0;
    if (i < j) std::swap(i, j);
    return values.at(i * (i - 1) / 2 + j);
  }
};
inline Input read(const char* path) {
  std::ifstream file(path);
  Input input;
  std::string cutoff;
  std::size_t count;
  if (!(file >> input.mode >> input.n >> input.q >> cutoff >> count)) throw std::runtime_error("invalid header");
  std::string rest;
  std::getline(file, rest);
  std::istringstream optional_field(rest);
  optional_field >> std::ws;
  if (optional_field.peek() != std::char_traits<char>::eof()) {
    std::uint64_t characteristic;
    if (!(optional_field >> characteristic) || characteristic > std::numeric_limits<std::uint32_t>::max())
      throw std::runtime_error("characteristic must fit u32");
    optional_field >> std::ws;
    if (!optional_field.eof()) throw std::runtime_error("invalid characteristic token");
    input.characteristic = static_cast<std::uint32_t>(characteristic);
  }
  if (input.characteristic < 2) throw std::runtime_error("characteristic must be prime");
  for (std::uint32_t divisor = 2; divisor <= input.characteristic / divisor; ++divisor)
    if (input.characteristic % divisor == 0) throw std::runtime_error("characteristic must be prime");
  input.cutoff = cutoff == "none" ? std::numeric_limits<double>::infinity() : std::stod(cutoff);
  if (input.mode == "dense") {
    for (std::size_t k = 0; k < count; ++k) {
      double value;
      if (!(file >> value)) throw std::runtime_error("missing value");
      input.values.push_back(value);
    }
    if (count != input.n * (input.n - 1) / 2) throw std::runtime_error("wrong shape");
  } else if (input.mode == "flag") {
    for (std::size_t k = 0; k < count; ++k) {
      std::size_t a, b;
      double value;
      if (!(file >> a >> b >> value) || a >= input.n || b >= input.n || a == b)
        throw std::runtime_error("invalid edge");
      if (b < a) std::swap(a, b);
      input.edges.emplace_back(a, b, value);
    }
  } else throw std::runtime_error("unknown mode");
  return input;
}
inline void emit(std::vector<Interval>& intervals, int q, int dimension, double birth, double death) {
  if (dimension <= q && death > birth) intervals.emplace_back(dimension, birth, death);
}
inline void output(std::vector<Edge> edges, std::vector<Interval> intervals, const std::vector<Simplex>* simplices = nullptr, std::uint32_t characteristic = 2) {
  std::sort(edges.begin(), edges.end());
  std::sort(intervals.begin(), intervals.end());
  std::cout << std::setprecision(17) << "{\"status\":\"ok\",\"edges\":[";
  bool first = true;
  for (auto [a, b, value] : edges) {
    if (!first) std::cout << ',';
    first = false;
    std::cout << '[' << a << ',' << b << ',' << value << ']';
  }
  std::cout << "],\"intervals\":[";
  first = true;
  for (auto [dimension, birth, death] : intervals) {
    if (!first) std::cout << ',';
    first = false;
    std::cout << '[' << dimension << ',' << birth << ',';
    if (std::isfinite(death)) std::cout << death; else std::cout << "null";
    std::cout << ']';
  }
  std::cout << "],\"characteristic\":" << characteristic;
  if (simplices) {
    std::cout << ",\"simplices\":[";
    first = true;
    for (const auto& [vertices, value] : *simplices) {
      if (!first) std::cout << ',';
      first = false;
      std::cout << "[[";
      for (std::size_t i = 0; i < vertices.size(); ++i) {
        if (i) std::cout << ',';
        std::cout << vertices[i];
      }
      std::cout << "]," << value << ']';
    }
    std::cout << ']';
  }
  std::cout << "}\n";
}
}
#endif
