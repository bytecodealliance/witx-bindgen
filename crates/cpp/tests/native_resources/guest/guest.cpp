#include "the_world_cpp.h"
#include <stdio.h>

exports::foo::foo::resources::R::Owned exports::foo::foo::resources::Create() {
    return R::Owned(new R(1));
}

void exports::foo::foo::resources::Borrows(std::reference_wrapper<const exports::foo::foo::resources::R> o) {
    printf("resource borrowed with %d\n", o.get().GetValue());
}

void exports::foo::foo::resources::Consume(R::Owned o) {
    printf("resource consumed with %d\n", o->GetValue());
    o.reset();

    printf("exercise the other direction\n");
    auto obj = ::foo::foo::resources::Create();
    obj.Add(12);
    ::foo::foo::resources::Borrows(obj);
    ::foo::foo::resources::Consume(std::move(obj));
    auto obj2 = ::foo::foo::resources::R{42};
}
