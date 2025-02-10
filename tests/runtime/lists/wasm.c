#include <assert.h>
#include <lists.h>
#include <float.h>
#include <limits.h>
#include <math.h>
#include <stdalign.h>
#include <stdlib.h>
#include <string.h>

uint32_t exports_lists_allocated_bytes(void) {
  return 0;
}

void exports_lists_test_imports() {
  {
    uint8_t list[] = {};
    lists_list_u8_t a;
    a.ptr = list;
    a.len = 0;
    test_lists_test_empty_list_param(&a);
  }

  {
    lists_string_t a;
    lists_string_set(&a, "");
    test_lists_test_empty_string_param(&a);
  }

  {
    lists_list_u8_t a;
    test_lists_test_empty_list_result(&a);
    assert(a.len == 0);
  }

  {
    lists_string_t a;
    test_lists_test_empty_string_result(&a);
    assert(a.len == 0);
  }

  {
    uint8_t list[] = {1, 2, 3, 4};
    lists_list_u8_t a;
    a.ptr = list;
    a.len = 4;
    test_lists_test_list_param(&a);
  }

  {
    lists_string_t a;
    lists_string_set(&a, "foo");
    test_lists_test_list_param2(&a);
  }

  {
    lists_string_t list[3];
    lists_string_set(&list[0], "foo");
    lists_string_set(&list[1], "bar");
    lists_string_set(&list[2], "baz");
    lists_list_string_t a;
    a.ptr = list;
    a.len = 3;
    test_lists_test_list_param3(&a);
  }

  {
    lists_string_t list1[2];
    lists_string_t list2[1];
    lists_string_set(&list1[0], "foo");
    lists_string_set(&list1[1], "bar");
    lists_string_set(&list2[0], "baz");
    lists_list_list_string_t a;
    a.ptr[0].len = 2;
    a.ptr[0].ptr = list1;
    a.ptr[1].len = 1;
    a.ptr[1].ptr = list2;
    a.len = 2;
    test_lists_test_list_param4(&a);
  }

  {
    lists_tuple3_u8_u32_u8_t data[2];
    data[0].f0 = 1;
    data[0].f1 = 2;
    data[0].f2 = 3;
    data[1].f0 = 4;
    data[1].f1 = 5;
    data[1].f2 = 6;
    lists_list_tuple3_u8_u32_u8_t a;
    a.len = 2;
    a.ptr = data;
    test_lists_test_list_param5(&a);
  }

  {
    lists_list_u8_t a;
    test_lists_test_list_result(&a);
    assert(a.len == 5);
    assert(memcmp(a.ptr, "\x01\x02\x03\x04\x05", 5) == 0);
    lists_list_u8_free(&a);
  }

  {
    lists_string_t a;
    test_lists_test_list_result2(&a);
    assert(a.len == 6);
    assert(memcmp(a.ptr, "hello!", 6) == 0);
    lists_string_free(&a);
  }

  {
    lists_list_string_t a;
    test_lists_test_list_result3(&a);
    assert(a.len == 2);
    assert(a.ptr[0].len == 6);
    assert(a.ptr[1].len == 6);
    assert(memcmp(a.ptr[0].ptr, "hello,", 6) == 0);
    assert(memcmp(a.ptr[1].ptr, "world!", 6) == 0);
    lists_list_string_free(&a);
  }

  {
    lists_list_u8_t a, b;
    a.len = 0;
    a.ptr = (unsigned char*) "";
    test_lists_test_list_roundtrip(&a, &b);
    assert(b.len == a.len);
    assert(memcmp(b.ptr, a.ptr, a.len) == 0);
    lists_list_u8_free(&b);

    a.len = 1;
    a.ptr = (unsigned char*) "x";
    test_lists_test_list_roundtrip(&a, &b);
    assert(b.len == a.len);
    assert(memcmp(b.ptr, a.ptr, a.len) == 0);
    lists_list_u8_free(&b);

    a.len = 5;
    a.ptr = (unsigned char*) "hello";
    test_lists_test_list_roundtrip(&a, &b);
    assert(b.len == a.len);
    assert(memcmp(b.ptr, a.ptr, a.len) == 0);
    lists_list_u8_free(&b);
  }

  {
    lists_string_t a, b;
    lists_string_set(&a, "x");
    test_lists_test_string_roundtrip(&a, &b);
    assert(b.len == a.len);
    assert(memcmp(b.ptr, a.ptr, a.len) == 0);
    lists_string_free(&b);

    lists_string_set(&a, "");
    test_lists_test_string_roundtrip(&a, &b);
    assert(b.len == a.len);
    lists_string_free(&b);

    lists_string_set(&a, "hello");
    test_lists_test_string_roundtrip(&a, &b);
    assert(b.len == a.len);
    assert(memcmp(b.ptr, a.ptr, a.len) == 0);
    lists_string_free(&b);

    lists_string_set(&a, "hello ⚑ world");
    test_lists_test_string_roundtrip(&a, &b);
    assert(b.len == a.len);
    assert(memcmp(b.ptr, a.ptr, a.len) == 0);
    lists_string_free(&b);
  }

  {
    uint8_t u8[2] = {0, UCHAR_MAX};
    int8_t s8[2] = {SCHAR_MIN, SCHAR_MAX};
    lists_list_u8_t list_u8 = { u8, 2 };
    lists_list_s8_t list_s8 = { s8, 2 };
    lists_tuple2_list_u8_list_s8_t ret;
    test_lists_test_list_minmax8(&list_u8, &list_s8, &ret);
    assert(ret.f0.len == 2 && ret.f0.ptr[0] == 0 && ret.f0.ptr[1] == UCHAR_MAX);
    assert(ret.f1.len == 2 && ret.f1.ptr[0] == SCHAR_MIN && ret.f1.ptr[1] == SCHAR_MAX);
    lists_list_u8_free(&ret.f0);
    lists_list_s8_free(&ret.f1);
  }

  {
    uint16_t u16[2] = {0, USHRT_MAX};
    int16_t s16[2] = {SHRT_MIN, SHRT_MAX};
    lists_list_u16_t list_u16 = { u16, 2 };
    lists_list_s16_t list_s16 = { s16, 2 };
    lists_tuple2_list_u16_list_s16_t ret;
    test_lists_test_list_minmax16(&list_u16, &list_s16, &ret);
    assert(ret.f0.len == 2 && ret.f0.ptr[0] == 0 && ret.f0.ptr[1] == USHRT_MAX);
    assert(ret.f1.len == 2 && ret.f1.ptr[0] == SHRT_MIN && ret.f1.ptr[1] == SHRT_MAX);
    lists_list_u16_free(&ret.f0);
    lists_list_s16_free(&ret.f1);
  }

  {
    uint32_t u32[2] = {0, UINT_MAX};
    int32_t s32[2] = {INT_MIN, INT_MAX};
    lists_list_u32_t list_u32 = { u32, 2 };
    lists_list_s32_t list_s32 = { s32, 2 };
    lists_tuple2_list_u32_list_s32_t ret;
    test_lists_test_list_minmax32(&list_u32, &list_s32, &ret);
    assert(ret.f0.len == 2 && ret.f0.ptr[0] == 0 && ret.f0.ptr[1] == UINT_MAX);
    assert(ret.f1.len == 2 && ret.f1.ptr[0] == INT_MIN && ret.f1.ptr[1] == INT_MAX);
    lists_list_u32_free(&ret.f0);
    lists_list_s32_free(&ret.f1);
  }

  {
    uint64_t u64[2] = {0, ULLONG_MAX};
    int64_t s64[2] = {LLONG_MIN, LLONG_MAX};
    lists_list_u64_t list_u64 = { u64, 2 };
    lists_list_s64_t list_s64 = { s64, 2 };
    lists_tuple2_list_u64_list_s64_t ret;
    test_lists_test_list_minmax64(&list_u64, &list_s64, &ret);
    assert(ret.f0.len == 2 && ret.f0.ptr[0] == 0 && ret.f0.ptr[1] == ULLONG_MAX);
    assert(ret.f1.len == 2 && ret.f1.ptr[0] == LLONG_MIN && ret.f1.ptr[1] == LLONG_MAX);
    lists_list_u64_free(&ret.f0);
    lists_list_s64_free(&ret.f1);
  }

  {
    float f32[4] = {-FLT_MAX, FLT_MAX, -INFINITY, INFINITY};
    double f64[4] = {-DBL_MAX, DBL_MAX, -INFINITY, INFINITY};
    lists_list_f32_t list_f32 = { f32, 4 };
    lists_list_f64_t list_f64 = { f64, 4 };
    lists_tuple2_list_f32_list_f64_t ret;
    test_lists_test_list_minmax_float(&list_f32, &list_f64, &ret);
    assert(ret.f0.len == 4 && ret.f0.ptr[0] == -FLT_MAX && ret.f0.ptr[1] == FLT_MAX);
    assert(ret.f0.ptr[2] == -INFINITY && ret.f0.ptr[3] == INFINITY);
    assert(ret.f1.len == 4 && ret.f1.ptr[0] == -DBL_MAX && ret.f1.ptr[1] == DBL_MAX);
    assert(ret.f1.ptr[2] == -INFINITY && ret.f1.ptr[3] == INFINITY);
    lists_list_f32_free(&ret.f0);
    lists_list_f64_free(&ret.f1);
  }
}

void exports_test_lists_test_empty_list_param(lists_list_u8_t *a) {
  assert(a->len == 0);
}

void exports_test_lists_test_empty_string_param(lists_string_t *a) {
  assert(a->len == 0);
}

void exports_test_lists_test_empty_list_result(lists_list_u8_t *ret0) {
  ret0->ptr = 0;
  ret0->len = 0;
}

void exports_test_lists_test_empty_string_result(lists_string_t *ret0) {
  ret0->ptr = 0;
  ret0->len = 0;
}

void exports_test_lists_test_list_param(lists_list_u8_t *a) {
  assert(a->len == 4);
  assert(a->ptr[0] == 1);
  assert(a->ptr[1] == 2);
  assert(a->ptr[2] == 3);
  assert(a->ptr[3] == 4);
  lists_list_u8_free(a);
}

void exports_test_lists_test_list_param2(lists_string_t *a) {
  assert(a->len == 3);
  assert(a->ptr[0] == 'f');
  assert(a->ptr[1] == 'o');
  assert(a->ptr[2] == 'o');
  lists_string_free(a);
}

void exports_test_lists_test_list_param3(lists_list_string_t *a) {
  assert(a->len == 3);
  assert(a->ptr[0].len == 3);
  assert(a->ptr[0].ptr[0] == 'f');
  assert(a->ptr[0].ptr[1] == 'o');
  assert(a->ptr[0].ptr[2] == 'o');

  assert(a->ptr[1].len == 3);
  assert(a->ptr[1].ptr[0] == 'b');
  assert(a->ptr[1].ptr[1] == 'a');
  assert(a->ptr[1].ptr[2] == 'r');

  assert(a->ptr[2].len == 3);
  assert(a->ptr[2].ptr[0] == 'b');
  assert(a->ptr[2].ptr[1] == 'a');
  assert(a->ptr[2].ptr[2] == 'z');

  lists_list_string_free(a);
}

void exports_test_lists_test_list_param4(lists_list_list_string_t *a) {
  assert(a->len == 2);
  assert(a->ptr[0].len == 2);
  assert(a->ptr[1].len == 1);

  assert(a->ptr[0].ptr[0].len == 3);
  assert(a->ptr[0].ptr[0].ptr[0] == 'f');
  assert(a->ptr[0].ptr[0].ptr[1] == 'o');
  assert(a->ptr[0].ptr[0].ptr[2] == 'o');

  assert(a->ptr[0].ptr[1].len == 3);
  assert(a->ptr[0].ptr[1].ptr[0] == 'b');
  assert(a->ptr[0].ptr[1].ptr[1] == 'a');
  assert(a->ptr[0].ptr[1].ptr[2] == 'r');

  assert(a->ptr[1].ptr[0].len == 3);
  assert(a->ptr[1].ptr[0].ptr[0] == 'b');
  assert(a->ptr[1].ptr[0].ptr[1] == 'a');
  assert(a->ptr[1].ptr[0].ptr[2] == 'z');

  lists_list_list_string_free(a);
}

void exports_test_lists_test_list_param_large(lists_list_string_t *a) {
  assert(a->len == 1000);
  lists_list_string_free(a);
}

void exports_test_lists_test_list_param5(lists_list_tuple3_u8_u32_u8_t *a) {
  assert(a->len == 2);
  assert(a->ptr[0].f0 == 1);
  assert(a->ptr[0].f1 == 2);
  assert(a->ptr[0].f2 == 3);
  assert(a->ptr[1].f0 == 4);
  assert(a->ptr[1].f1 == 5);
  assert(a->ptr[1].f2 == 6);
  lists_list_tuple3_u8_u32_u8_free(a);
}

void exports_test_lists_test_list_result(lists_list_u8_t *ret0) {
  ret0->ptr = (uint8_t *) malloc(5);
  ret0->len = 5;
  ret0->ptr[0] = 1;
  ret0->ptr[1] = 2;
  ret0->ptr[2] = 3;
  ret0->ptr[3] = 4;
  ret0->ptr[4] = 5;
}

void exports_test_lists_test_list_result2(lists_string_t *ret0) {
  lists_string_dup(ret0, "hello!");
}

void exports_test_lists_test_list_result3(lists_list_string_t *ret0) {
  ret0->len = 2;
  ret0->ptr = (lists_string_t *) malloc(2 * sizeof(lists_string_t));

  lists_string_dup(&ret0->ptr[0], "hello,");
  lists_string_dup(&ret0->ptr[1], "world!");
}

void exports_test_lists_test_list_roundtrip(lists_list_u8_t *a, lists_list_u8_t *ret0) {
  *ret0 = *a;
}

void exports_test_lists_test_string_roundtrip(lists_string_t *a, lists_string_t *ret0) {
  *ret0 = *a;
}

void exports_test_lists_test_list_minmax8(lists_list_u8_t *a, lists_list_s8_t *b, lists_tuple2_list_u8_list_s8_t *ret) {
  assert(0); // unimplemented
}

void exports_test_lists_test_list_minmax16(lists_list_u16_t *a, lists_list_s16_t *b, lists_tuple2_list_u16_list_s16_t *ret) {
  assert(0); // unimplemented
}

void exports_test_lists_test_list_minmax32(lists_list_u32_t *a, lists_list_s32_t *b, lists_tuple2_list_u32_list_s32_t *ret) {
  assert(0); // unimplemented
}

void exports_test_lists_test_list_minmax64(lists_list_u64_t *a, lists_list_s64_t *b, lists_tuple2_list_u64_list_s64_t *ret) {
  assert(0); // unimplemented
}

void exports_test_lists_test_list_minmax_float(lists_list_f32_t *a, lists_list_f64_t *b, lists_tuple2_list_f32_list_f64_t *ret) {
  assert(0); // unimplemented
}
