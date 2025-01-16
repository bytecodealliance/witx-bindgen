#include <smoke_cpp.h>
//#include <stdio.h>

void exports::smoke::Thunk() {
    test::smoke::imports::Thunk();
}
