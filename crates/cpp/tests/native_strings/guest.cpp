#include "the_world_cpp.h"

void exports::foo::foo::strings::A(wit::string &&x) {
    ::foo::foo::strings::A(x.get_view());
}

wit::string exports::foo::foo::strings::B() {
    return ::foo::foo::strings::B();
}

wit::string exports::foo::foo::strings::C(wit::string &&x, wit::string &&b) {
    return ::foo::foo::strings::C(x.get_view(), b.get_view());
}
