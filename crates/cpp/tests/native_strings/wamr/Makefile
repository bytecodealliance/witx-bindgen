WIT_BINDGEN=../../../../../target/debug/wit-bindgen

all: libstrings.so guest_release.wasm

libstrings.so: 
	mkdir build
	(cd build; cmake .. ; make)
	
guest_release.wasm: ../the_world.cpp ../guest.cpp
	/opt/wasi-sdk/bin/clang++ -o $@ $^ -I../../../helper-types -g0 -O3
	
clean:
	-rm -r build guest_release.wasm

run:
	LD_LIBRARY_PATH=. ../app-strings

# valgrind:
# 	LD_LIBRARY_PATH=. valgrind ../app-strings

# not yet up to the task
#the-world_bridge.c: $(WIT_BINDGEN)
#	$(WIT_BINDGEN) bridge ../wit --instance guestrelease --include w2c2_guest.h
