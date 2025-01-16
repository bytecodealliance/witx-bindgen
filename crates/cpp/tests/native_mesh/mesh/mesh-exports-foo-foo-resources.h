#pragma once
#include "mesh-exports-foo-foo-resources-R.h"
#include <cstdint>
#include <utility>
// export_interface Interface(Id { idx: 0 })
namespace mesh {
namespace exports {
namespace foo {
namespace foo {
namespace resources {
R Create();
void Consume(R &&o);
} // namespace resources
} // namespace foo
} // namespace foo
} // namespace exports
} // namespace mesh
