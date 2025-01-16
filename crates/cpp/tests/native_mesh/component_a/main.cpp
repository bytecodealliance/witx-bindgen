
#include "a_cpp.h"

int main() {
    {
        auto obj = foo::foo::resources::R(5);
        obj.Add(2);
    }
    auto obj2 = foo::foo::resources::Create();
    foo::foo::resources::Consume(std::move(obj2));
    return 0;
}
