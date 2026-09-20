// Upstream Ripser implicit dense/sparse computation, float32 coefficients enabled.
#include "common.hpp"
namespace native_bench {
std::vector<reference::Interval> intervals;
int requested_dimension;
void emit(int dim,double birth,double death){reference::emit(intervals,requested_dimension,dim,birth,death);}
}
#define USE_COEFFICIENTS
#define main cocycle_unused_ripser_main
#include "ripser_instrumented.cpp"
#undef main
int main(int argc,char** argv){
  try{
    if(argc!=6)throw std::runtime_error("usage: pipeline FIXTURE PATH LAYOUT EPSILON REPRESENTATIVES");
    auto input=reference::read(argv[1]);
    if(input.n<2||input.characteristic>251)throw std::runtime_error("unsupported input");
    if(std::string(argv[2]).find("approximate")==0)throw std::runtime_error("no approximation constructor");
    auto baseline=pipeline::memory("VmHWM:"),rss=pipeline::memory("VmRSS:");
    auto start=pipeline::Clock::now();std::array<double,5> phases{};auto mark=pipeline::Clock::now();
    for(double v:input.values)if(!std::isfinite(v)||v<0||double(value_t(v))!=v)throw std::runtime_error("distance not float32-exact");
    for(auto [a,b,v]:input.edges)if(!std::isfinite(v)||v<0||double(value_t(v))!=v)throw std::runtime_error("edge not float32-exact");
    if(std::isfinite(input.cutoff)&&double(value_t(input.cutoff))!=input.cutoff)throw std::runtime_error("cutoff not float32-exact");
    phases[0]=pipeline::ms(mark);native_bench::requested_dimension=input.q;
    std::size_t edges=0;
    if(std::string(argv[2])=="dense"){
      mark=pipeline::Clock::now();
      std::vector<value_t> values(input.values.begin(),input.values.end());
      compressed_lower_distance_matrix matrix(std::move(values));phases[1]=pipeline::ms(mark);
      mark=pipeline::Clock::now();
      ripser<compressed_lower_distance_matrix>(std::move(matrix),input.q,value_t(input.cutoff),1,coefficient_t(input.characteristic)).compute_barcodes();
      phases[3]=pipeline::ms(mark);edges=input.values.size();
    }else{
      mark=pipeline::Clock::now();std::vector<std::vector<index_diameter_t>> neighbors(input.n);
      auto add=[&](std::size_t a,std::size_t b,double v){if(v<=input.cutoff){neighbors[a].emplace_back(b,value_t(v));neighbors[b].emplace_back(a,value_t(v));++edges;}};
      if(input.mode=="dense")for(std::size_t b=0;b<input.n;++b)for(std::size_t a=0;a<b;++a)add(a,b,input(a,b));
      else for(auto [a,b,v]:input.edges)add(a,b,v);
      for(auto& row:neighbors)std::sort(row.begin(),row.end());
      sparse_distance_matrix matrix(std::move(neighbors),edges);phases[1]=pipeline::ms(mark);
      mark=pipeline::Clock::now();
      ripser<sparse_distance_matrix>(std::move(matrix),input.q,value_t(input.cutoff),1,coefficient_t(input.characteristic)).compute_barcodes();
      phases[3]=pipeline::ms(mark);
    }
    mark=pipeline::Clock::now();auto payload=pipeline::encode(native_bench::intervals);phases[4]=pipeline::ms(mark);
    pipeline::output(pipeline::ms(start),phases,rss,baseline,payload,input.n,edges,0);
  }catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}
}
