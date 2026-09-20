// Real GUDHI C++ construction/expansion/reduction, no Python wrapper or oracle traversal.
#include "common.hpp"
#include <gudhi/Simplex_tree.h>
#include <gudhi/Persistent_cohomology.h>
#include <gudhi/Rips_complex.h>
#include "Sparse_rips_fixed_start.h"
int main(int argc,char** argv) {
  try {
    if(argc!=6)throw std::runtime_error("usage: pipeline FIXTURE PATH LAYOUT EPSILON REPRESENTATIVES");
    auto input=reference::read(argv[1]);
    if(input.characteristic>46337)throw std::runtime_error("unsupported coefficient field");
    auto baseline=pipeline::memory("VmHWM:"),rss=pipeline::memory("VmRSS:");
    auto start=pipeline::Clock::now();std::array<double,5> phases{};
    auto mark=pipeline::Clock::now();
    for(double v:input.values)if(!std::isfinite(v)||v<0)throw std::runtime_error("invalid distance");
    phases[0]=pipeline::ms(mark);
    using Tree=Gudhi::Simplex_tree<>;Tree tree;std::vector<int> order;
    std::string path=argv[2];
    mark=pipeline::Clock::now();
    if(path.find("approximate")==0) {
      Gudhi::rips_complex::Sparse_rips_complex<double> rips(boost::irange<int>(0,int(input.n)),
          [&](int a,int b){return input(a,b);},std::stod(argv[4]),0.,input.cutoff);
      phases[1]=pipeline::ms(mark);mark=pipeline::Clock::now();
      rips.create_complex(tree,input.q+1);
      phases[2]=pipeline::ms(mark);
      order=rips.benchmark_permutation();
    }else{
      if(input.mode=="dense"){
        Gudhi::rips_complex::Rips_complex<double> rips(input,input.cutoff);rips.create_complex(tree,1);
      }else{
        for(std::size_t v=0;v<input.n;++v)tree.insert_simplex({int(v)},0.);
        for(auto [a,b,value]:input.edges)if(value<=input.cutoff)tree.insert_simplex({int(a),int(b)},value);
      }
      phases[1]=pipeline::ms(mark);mark=pipeline::Clock::now();
      tree.expansion(input.q+1);phases[2]=pipeline::ms(mark);
    }
    mark=pipeline::Clock::now();
    using CAM=Gudhi::persistent_cohomology::Persistent_cohomology<Tree,Gudhi::persistent_cohomology::Field_Zp>;
    CAM persistence(tree,input.q>=tree.dimension());persistence.init_coefficients(input.characteristic);
    persistence.compute_persistent_cohomology(0.);phases[3]=pipeline::ms(mark);
    mark=pipeline::Clock::now();std::vector<reference::Interval> intervals;
    for(auto pair:persistence.get_persistent_pairs()){
      auto birth=std::get<0>(pair),death=std::get<1>(pair);
      reference::emit(intervals,input.q,tree.dimension(birth),tree.filtration(birth),tree.filtration(death));
    }
    auto payload=pipeline::encode(intervals);phases[4]=pipeline::ms(mark);
    auto elapsed=pipeline::ms(start);
    std::size_t edges=0;for(auto s:tree.skeleton_simplex_range(1))if(tree.dimension(s)==1)++edges;
    pipeline::output(elapsed,phases,rss,baseline,payload,tree.num_vertices(),edges,tree.num_simplices(),order);
  }catch(const std::exception& e){std::cerr<<e.what()<<'\n';return 1;}
}
