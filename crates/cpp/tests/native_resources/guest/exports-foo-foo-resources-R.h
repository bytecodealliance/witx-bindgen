/* User class definition file, autogenerated once, then user modified
 * Updated versions of this file are generated into
 * exports-foo-foo-resources-R.h.template.
 */
#include <memory>
namespace exports {
namespace foo {
namespace foo {
namespace resources {
class R : public wit::ResourceExportBase<R> {
  uint32_t value;

public:
  static void Dtor(R *self) { delete self; };
  struct Deleter {
    void operator()(R* ptr) const { R::Dtor(ptr); }
  };
  typedef std::unique_ptr<R, R::Deleter> Owned;
  R(uint32_t a) : value(a) {}
  static Owned New(uint32_t a) { return Owned(new R(a)); }
  void Add(uint32_t b) { value += b; }
  static int32_t ResourceNew(R *self);
  static void ResourceDrop(int32_t id);
  static R* ResourceRep(int32_t id);

  uint32_t GetValue() const { return value; }
};

} // namespace resources
} // namespace foo
} // namespace foo
} // namespace exports
