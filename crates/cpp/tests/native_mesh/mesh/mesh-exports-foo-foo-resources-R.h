#pragma once
#include <cassert>
#include <cstdint>
#include <utility>
#include <wit-host.h>
namespace mesh {
namespace exports {
namespace foo {
namespace foo {
namespace resources {
class R : public wit::ResourceExportBase {

public:
  ~R();
  R(uint32_t a);
  void Add(uint32_t b) const;
  R(wit::ResourceExportBase &&);
  R(R &&) = default;
  R &operator=(R &&) = default;
};

} // namespace resources
} // namespace foo
} // namespace foo
} // namespace exports
} // namespace mesh
