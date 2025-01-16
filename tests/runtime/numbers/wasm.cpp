#include <assert.h>
#include <limits.h>
#include <math.h>
#include <numbers_cpp.h>

uint8_t exports::test::numbers::test::RoundtripU8(uint8_t a) {
  return a;
}

int8_t exports::test::numbers::test::RoundtripS8(int8_t a) {
  return a;
}

uint16_t exports::test::numbers::test::RoundtripU16(uint16_t a) {
  return a;
}

int16_t exports::test::numbers::test::RoundtripS16(int16_t a) {
  return a;
}

uint32_t exports::test::numbers::test::RoundtripU32(uint32_t a) {
  return a;
}

int32_t exports::test::numbers::test::RoundtripS32(int32_t a) {
  return a;
}

uint64_t exports::test::numbers::test::RoundtripU64(uint64_t a) {
  return a;
}

int64_t exports::test::numbers::test::RoundtripS64(int64_t a) {
  return a;
}

float exports::test::numbers::test::RoundtripF32(float a) {
  return a;
}

double exports::test::numbers::test::RoundtripF64(double a) {
  return a;
}

uint32_t exports::test::numbers::test::RoundtripChar(uint32_t a) {
  return a;
}

static uint32_t SCALAR = 0;

void exports::test::numbers::test::SetScalar(uint32_t a) {
  SCALAR = a;
}

uint32_t exports::test::numbers::test::GetScalar(void) {
  return SCALAR;
}


void exports::numbers::TestImports() {
  assert(::test::numbers::test::RoundtripU8(1) == 1);
  assert(::test::numbers::test::RoundtripU8(0) == 0);
  assert(::test::numbers::test::RoundtripU8(UCHAR_MAX) == UCHAR_MAX);

  assert(::test::numbers::test::RoundtripS8(1) == 1);
  assert(::test::numbers::test::RoundtripS8(SCHAR_MIN) == SCHAR_MIN);
  assert(::test::numbers::test::RoundtripS8(SCHAR_MAX) == SCHAR_MAX);

  assert(::test::numbers::test::RoundtripU16(1) == 1);
  assert(::test::numbers::test::RoundtripU16(0) == 0);
  assert(::test::numbers::test::RoundtripU16(USHRT_MAX) == USHRT_MAX);

  assert(::test::numbers::test::RoundtripS16(1) == 1);
  assert(::test::numbers::test::RoundtripS16(SHRT_MIN) == SHRT_MIN);
  assert(::test::numbers::test::RoundtripS16(SHRT_MAX) == SHRT_MAX);

  assert(::test::numbers::test::RoundtripU32(1) == 1);
  assert(::test::numbers::test::RoundtripU32(0) == 0);
  assert(::test::numbers::test::RoundtripU32(UINT_MAX) == UINT_MAX);

  assert(::test::numbers::test::RoundtripS32(1) == 1);
  assert(::test::numbers::test::RoundtripS32(INT_MIN) == INT_MIN);
  assert(::test::numbers::test::RoundtripS32(INT_MAX) == INT_MAX);

  assert(::test::numbers::test::RoundtripU64(1) == 1);
  assert(::test::numbers::test::RoundtripU64(0) == 0);
  assert(::test::numbers::test::RoundtripU64(ULONG_MAX) == ULONG_MAX);

  assert(::test::numbers::test::RoundtripS64(1) == 1);
  assert(::test::numbers::test::RoundtripS64(LONG_MIN) == LONG_MIN);
  assert(::test::numbers::test::RoundtripS64(LONG_MAX) == LONG_MAX);

  assert(::test::numbers::test::RoundtripF32(1.0) == 1.0);
  assert(::test::numbers::test::RoundtripF32(INFINITY) == INFINITY);
  assert(::test::numbers::test::RoundtripF32(-INFINITY) == -INFINITY);
  assert(isnan(::test::numbers::test::RoundtripF32(NAN)));

  assert(::test::numbers::test::RoundtripF64(1.0) == 1.0);
  assert(::test::numbers::test::RoundtripF64(INFINITY) == INFINITY);
  assert(::test::numbers::test::RoundtripF64(-INFINITY) == -INFINITY);
  assert(isnan(::test::numbers::test::RoundtripF64(NAN)));

  assert(::test::numbers::test::RoundtripChar('a') == 'a');
  assert(::test::numbers::test::RoundtripChar(' ') == ' ');
  assert(::test::numbers::test::RoundtripChar(U'🚩') == U'🚩');

  ::test::numbers::test::SetScalar(2);
  assert(::test::numbers::test::GetScalar() == 2);
  ::test::numbers::test::SetScalar(4);
  assert(::test::numbers::test::GetScalar() == 4);
}
