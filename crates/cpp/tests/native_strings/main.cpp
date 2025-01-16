
#include "the_world_cpp_native.h"
#include <iostream>

void foo::foo::strings::A(std::string_view x) {
    std::cout << x << std::endl;
}
wit::string foo::foo::strings::B() {
    wit::string b = wit::string::from_view(std::string_view("hello B"));
    return b;
}
wit::string foo::foo::strings::C(std::string_view a, std::string_view b) {
    std::cout << a << '|' << b << std::endl;
    wit::string c = wit::string::from_view(std::string_view("hello C"));
    return c;
}

int main() {
    wit::string a = wit::string::from_view(std::string_view("hello A"));
    exports::foo::foo::strings::A(a);

    {
        auto b = exports::foo::foo::strings::B();
        std::cout << b.inner() << std::endl;
        // make sure that b's result is destructed before calling C
    }

    wit::string c1 = wit::string::from_view(std::string_view("hello C1"));
    wit::string c2 = wit::string::from_view(std::string_view("hello C2"));
    auto c = exports::foo::foo::strings::C(c1, c2);
    std::cout << c.inner() << std::endl;
    return 0;
}
