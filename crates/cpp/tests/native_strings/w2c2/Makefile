CXXFLAGS=-g -O0 -I../../../helper-types
WIT_BINDGEN=../../../../../target/debug/wit-bindgen
W2C2_PATH=$(HOME)/github/w2c2

all: libstrings.so

w2c2_base.h: $(W2C2_PATH)/w2c2/w2c2_base.h
	ln -s $^ .

%.pie.o: %.c
	$(CC) $(CXXFLAGS) -fPIE -o $@ -c $^

libstrings.so: w2c2_guest.pie.o the-world_bridge.pie.o wasi_dummy.pie.o
	$(CC) $(CXXFLAGS) -shared -o $@ $^ -Wl,--version-script=../guest.lds
	
guest_release.wasm: ../the_world.cpp ../guest.cpp
	/opt/wasi-sdk/bin/clang++ -o $@ $^ $(CXXFLAGS) -g0 -O3
	
clean:
	-rm *.o libstrings.so

run:
	LD_LIBRARY_PATH=. ../app-strings

valgrind:
	LD_LIBRARY_PATH=. valgrind ../app-strings

w2c2_guest.c: guest_release.wasm
	$(W2C2_PATH)/build/w2c2/w2c2 $^ $@

the-world_bridge.c: the-world_bridge_target.c
	cp $^ $@

# not yet up to the task
#the-world_bridge.c: $(WIT_BINDGEN)
#	$(WIT_BINDGEN) bridge ../wit --instance guestrelease --include w2c2_guest.h
