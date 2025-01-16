CXXFLAGS=-fPIC -g -O0 -I../../../helper-types

all: libmesh.so

libmesh.so: mesh_native.cpp impl.cpp
	$(CXX) -shared $(CXXFLAGS) -o $@ $^ libcomponent_b.so

bindgen:
	../../../../../target/debug/wit-bindgen cpp ../wit -w mesh --wasm64 --format --direct --split-interfaces --internal-prefix mesh

clean:
	-rm *~ libmesh.so *.o
