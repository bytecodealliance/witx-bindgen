CXXFLAGS=-g -O0 -I../../../helper-types

all: component_a

component_a: the_world.cpp main.cpp
	$(CXX) $(CXXFLAGS) -o $@ $^ -L../component_b -lcomponent_b

bindgen:
	../../../../../target/debug/wit-bindgen cpp ../wit --symmetric --internal-prefix=comp_a --new_api --format

clean:
	-rm *~ component_a *.o
