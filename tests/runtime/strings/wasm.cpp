#include <assert.h>
#include <strings_cpp.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

void assert_str(std::string_view str, const char* expected) {
  size_t expected_len = strlen(expected);
  assert(str.size() == expected_len);
  assert(memcmp(str.data(), expected, expected_len) == 0);
}

void exports::strings::TestImports() {
  test::strings::imports::TakeBasic(std::string_view("latin utf16"));

  wit::string str2 = test::strings::imports::ReturnUnicode();
  assert_str(str2.get_view(), "🚀🚀🚀 𠈄𓀀");
}

wit::string exports::strings::ReturnEmpty() {
  // return a non-zero address (follows cabi_realloc logic)
  return wit::string((char const*)1, 0);
}

wit::string exports::strings::Roundtrip(wit::string &&str) {
  assert(str.size() > 0);
  return std::move(str);
}
