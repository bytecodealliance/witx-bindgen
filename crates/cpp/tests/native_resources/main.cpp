
#include "the_world_cpp_native.h"
#include <iostream>

foo::foo::resources::R::Owned foo::foo::resources::Create() { 
    return R::New(1);
}
void foo::foo::resources::Borrows(std::reference_wrapper<R const> o) {
    printf("resource borrowed with %d\n", o.get().GetValue());
}
void foo::foo::resources::Consume(R::Owned o) { 
    printf("resource consumed with %d\n", o->GetValue());
}

int main() {
    auto obj = exports::foo::foo::resources::Create();
    obj.Add(12);
    exports::foo::foo::resources::Borrows(obj);
    exports::foo::foo::resources::Consume(std::move(obj));
    auto obj2 = exports::foo::foo::resources::R{42};
    return 0;
}
