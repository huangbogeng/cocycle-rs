// Pipeline timing boundaries. All worker memory readings belong to this process.
#pragma once
#include "../../tools/reference/rips_common.hpp"
#include <chrono>
#include <array>
#include <sstream>
namespace pipeline {
using Clock=std::chrono::steady_clock;
inline double ms(Clock::time_point start) {return std::chrono::duration<double,std::milli>(Clock::now()-start).count();}
inline long memory(const std::string& key) {
  std::ifstream file("/proc/self/status"); std::string line;
  while(std::getline(file,line)) if(line.compare(0,key.size(),key)==0) return std::stol(line.substr(key.size()));
  throw std::runtime_error("Linux process memory counters unavailable");
}
inline std::string encode(std::vector<reference::Interval>& intervals) {
  std::sort(intervals.begin(),intervals.end());
  std::ostringstream out; out<<std::setprecision(17)<<'[';
  bool first=true;
  for(auto [dim,birth,death]:intervals){
    if(!first)out<<',';first=false;out<<'['<<dim<<','<<birth<<',';
    if(std::isfinite(death))out<<death;else out<<"null";out<<']';
  }out<<']';return out.str();
}
inline void output(double elapsed,const std::array<double,5>& phases,long rss,long baseline,
    const std::string& intervals,std::size_t vertices,std::size_t edges,std::size_t simplices,
    const std::vector<int>& order={}) {
  auto peak=memory("VmHWM:");
  std::cout<<std::setprecision(17)<<"{\"status\":\"completed\",\"elapsed_ms\":"<<elapsed<<",\"phases_ms\":[";
  for(int i=0;i<5;++i){if(i)std::cout<<',';std::cout<<phases[i];}
  std::cout<<"],\"rss_before_kib\":"<<rss<<",\"hwm_before_kib\":"<<baseline<<",\"peak_rss_kib\":"<<peak
      <<",\"intervals\":"<<intervals<<",\"vertices\":"<<vertices<<",\"edges\":"<<edges<<",\"simplices\":"<<simplices<<",\"permutation\":[";
  for(std::size_t i=0;i<order.size();++i){if(i)std::cout<<',';std::cout<<order[i];}
  std::cout<<"]}\n";
}
}
