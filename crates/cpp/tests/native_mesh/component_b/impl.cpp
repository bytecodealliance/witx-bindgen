#include "b_cpp.h"

exports::foo::foo::resources::R::Owned exports::foo::foo::resources::Create() {
    return R::New(17);
}
void exports::foo::foo::resources::Consume(R::Owned o) {
    printf("Consumed with %d\n", o->get_value());
}
