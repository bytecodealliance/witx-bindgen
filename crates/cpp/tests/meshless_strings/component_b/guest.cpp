#include "the_world_cpp.h"

void exports::foo::foo::strings::A(std::string_view x) {
    ::foo::foo::strings::A(x);
}

wit::string exports::foo::foo::strings::B() {
    return ::foo::foo::strings::B();
}

wit::string exports::foo::foo::strings::C(std::string_view x, std::string_view b) {
    return ::foo::foo::strings::C(x, b);
}
