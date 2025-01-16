CXXFLAGS=-g -O0 -I../../../helper-types

all: libresources.so

libresources.so: the_world.pie.o guest.pie.o
	$(CXX) $(CXXFLAGS) -shared -o $@ $^ -Wl,--version-script=guest.verscr
	
%.pie.o: %.cpp
	$(CXX) $(CXXFLAGS) -fPIE -o $@ -c $^

guest.wasm: the_world.cpp guest.cpp
	/opt/wasi-sdk/bin/clang++ -o $@ $^ $(CXXFLAGS) 
	
clean:
	-rm *.o libresources.so
