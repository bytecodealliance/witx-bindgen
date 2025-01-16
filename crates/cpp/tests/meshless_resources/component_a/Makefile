CXXFLAGS=-g -O0 -I../../../helper-types

all: component_a

component_a: a.cpp main.cpp
	$(CXX) $(CXXFLAGS) -o $@ $^ -L../component_b -lcomponent_b

bindgen:
	../../../../../target/debug/wit-bindgen cpp ../wit -w a --symmetric --wasm64 --format

clean:
	-rm *~ component_a *.o
