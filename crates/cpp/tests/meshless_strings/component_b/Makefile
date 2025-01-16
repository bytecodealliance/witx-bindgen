CXXFLAGS=-g -O0 -I../../../helper-types

all: libcomponent_b.a

libcomponent_b.a: the_world.o guest.o
	ar rcvs $@ $^

bindgen:
	../../../../../target/debug/wit-bindgen cpp ../wit --symmetric --new-api --format

clean:
	-rm *~ *.a *.o
