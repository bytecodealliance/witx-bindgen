CXXFLAGS=-g -O0 -I../../../helper-types

all: libcomponent_b.a

libcomponent_b.a: b.o impl.o
	ar rcvs $@ $^

bindgen:
	../../../../../target/debug/wit-bindgen cpp ../wit -w b --symmetric --wasm64 --format

clean:
	-rm *~ *.a *.o
