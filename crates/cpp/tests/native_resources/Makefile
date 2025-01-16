CXXFLAGS=-g -O0 -I../../helper-types
WIT_BINDGEN=../../../../target/debug/wit-bindgen

all: libresources.so app-resources

app-resources: the_world_native.o main.o
	$(CXX) $(CXXFLAGS) -o $@ $^ -L. -lresources

bindgen: wit/resources_simple.wit
	cd guest; ../$(WIT_BINDGEN) cpp ../wit --wasm64 --format
	$(WIT_BINDGEN) cpp wit --wasm64 --format --direct
	cd rust/src ; ../../$(WIT_BINDGEN) rust ../../wit --wasm64
	
clean:
	-rm *.o app-resources

run: app-resources
	LD_LIBRARY_PATH=. ./app-resources

valgrind: app-resources
	LD_LIBRARY_PATH=. valgrind --leak-check=full ./app-resources
