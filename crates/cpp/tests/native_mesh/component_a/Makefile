CXXFLAGS=-g -O0 -I../../../helper-types

all: component_a

component_a: a.cpp main.cpp
	$(CXX) $(CXXFLAGS) -o $@ $^ libmesh.so libcomponent_b.so

bindgen:
	../../../../../target/debug/wit-bindgen cpp ../wit -w a --wasm64 --format

clean:
	-rm *~ component_a *.o
