// Generated by `wit-bindgen` 0.3.0. DO NOT EDIT!

// Ensure that the *_component_type.o object is linked in
#ifdef __wasm32__
extern void __component_type_object_force_link_the_world(void);
void __component_type_object_force_link_the_world_public_use_in_this_compilation_unit(
    void) {
  __component_type_object_force_link_the_world();
}
#endif
#include "the_world_cpp.h"
#include <cstdlib> // realloc

extern "C" void *cabi_realloc(void *ptr, size_t old_size, size_t align,
                              size_t new_size);

__attribute__((__weak__, __export_name__("cabi_realloc"))) void *
cabi_realloc(void *ptr, size_t old_size, size_t align, size_t new_size) {
  (void)old_size;
  if (new_size == 0)
    return (void *)align;
  void *ret = realloc(ptr, new_size);
  if (!ret)
    abort();
  return ret;
}

extern "C" __attribute__((import_module("foo:foo/strings")))
__attribute__((import_name("a"))) void
a_fooX3AfooX2FstringsX00a(uint8_t *, size_t);
extern "C" __attribute__((import_module("foo:foo/strings")))
__attribute__((import_name("b"))) void
a_fooX3AfooX2FstringsX00b(uint8_t *);
extern "C" __attribute__((import_module("foo:foo/strings")))
__attribute__((import_name("c"))) void
a_fooX3AfooX2FstringsX00c(uint8_t *, size_t, uint8_t *, size_t, uint8_t *);
void foo::foo::strings::A(std::string_view x) {
  auto const &vec0 = x;
  auto ptr0 = (uint8_t *)(vec0.data());
  auto len0 = (size_t)(vec0.size());
  a_fooX3AfooX2FstringsX00a(ptr0, len0);
}
wit::string foo::foo::strings::B() {
  size_t ret_area[2];
  uint8_t *ptr0 = (uint8_t *)(&ret_area);
  a_fooX3AfooX2FstringsX00b(ptr0);
  auto len1 = *((size_t *)(ptr0 + sizeof(size_t)));

  return wit::string((char const *)(*((uint8_t **)(ptr0 + 0))), len1);
}
wit::string foo::foo::strings::C(std::string_view a, std::string_view b) {
  auto const &vec0 = a;
  auto ptr0 = (uint8_t *)(vec0.data());
  auto len0 = (size_t)(vec0.size());
  auto const &vec1 = b;
  auto ptr1 = (uint8_t *)(vec1.data());
  auto len1 = (size_t)(vec1.size());
  size_t ret_area[2];
  uint8_t *ptr2 = (uint8_t *)(&ret_area);
  a_fooX3AfooX2FstringsX00c(ptr0, len0, ptr1, len1, ptr2);
  auto len3 = *((size_t *)(ptr2 + sizeof(size_t)));

  return wit::string((char const *)(*((uint8_t **)(ptr2 + 0))), len3);
}
extern "C" __attribute__((__export_name__("foo:foo/strings#a"))) void
fooX3AfooX2FstringsX00a(uint8_t *arg0, size_t arg1) {
  auto len0 = arg1;

  exports::foo::foo::strings::A(wit::string((char const *)(arg0), len0));
}
extern "C" __attribute__((__export_name__("foo:foo/strings#b"))) uint8_t *
fooX3AfooX2FstringsX00b() {
  auto result0 = exports::foo::foo::strings::B();
  static size_t ret_area[2];
  uint8_t *ptr1 = (uint8_t *)(&ret_area);
  auto const &vec2 = result0;
  auto ptr2 = (uint8_t *)(vec2.data());
  auto len2 = (size_t)(vec2.size());
  result0.leak();

  *((size_t *)(ptr1 + sizeof(size_t))) = len2;
  *((uint8_t **)(ptr1 + 0)) = ptr2;
  return ptr1;
}
extern "C"
    __attribute__((__weak__,
                   __export_name__("cabi_post_fooX3AfooX2FstringsX23b"))) void
    cabi_post_fooX3AfooX2FstringsX00b(uint8_t *arg0) {
  if ((*((size_t *)(arg0 + sizeof(size_t)))) > 0) {
    wit::string::drop_raw((void *)(*((uint8_t **)(arg0 + 0))));
  }
}
extern "C" __attribute__((__export_name__("foo:foo/strings#c"))) uint8_t *
fooX3AfooX2FstringsX00c(uint8_t *arg0, size_t arg1, uint8_t *arg2,
                        size_t arg3) {
  auto len0 = arg1;

  auto len1 = arg3;

  auto result2 =
      exports::foo::foo::strings::C(wit::string((char const *)(arg0), len0),
                                    wit::string((char const *)(arg2), len1));
  static size_t ret_area[2];
  uint8_t *ptr3 = (uint8_t *)(&ret_area);
  auto const &vec4 = result2;
  auto ptr4 = (uint8_t *)(vec4.data());
  auto len4 = (size_t)(vec4.size());
  result2.leak();

  *((size_t *)(ptr3 + sizeof(size_t))) = len4;
  *((uint8_t **)(ptr3 + 0)) = ptr4;
  return ptr3;
}
extern "C"
    __attribute__((__weak__,
                   __export_name__("cabi_post_fooX3AfooX2FstringsX23c"))) void
    cabi_post_fooX3AfooX2FstringsX00c(uint8_t *arg0) {
  if ((*((size_t *)(arg0 + sizeof(size_t)))) > 0) {
    wit::string::drop_raw((void *)(*((uint8_t **)(arg0 + 0))));
  }
}

// Component Adapters
