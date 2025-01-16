
#include "mesh_cpp_native.h"

mesh::foo::foo::resources::R::R(uint32_t a) 
: impl(exports::foo::foo::resources::R(a)) {}

mesh::foo::foo::resources::R::R(exports::foo::foo::resources::R && a)
: impl(std::move(a)) {}

void mesh::foo::foo::resources::R::Add(uint32_t b) {
    impl.Add(b);
}

mesh::foo::foo::resources::R::Owned 
mesh::foo::foo::resources::Create() {
    return mesh::foo::foo::resources::R::Owned(new mesh::foo::foo::resources::R
        (exports::foo::foo::resources::Create()));
}

void mesh::foo::foo::resources::Consume(mesh::foo::foo::resources::R::Owned obj) {
    exports::foo::foo::resources::Consume(obj->into_inner());
}
