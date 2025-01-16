#pragma once
#include <stdint.h>

typedef uint8_t wasm_valkind_t;
enum wasm_valkind_enum {
    WASM_I32,
    WASM_I64,
    WASM_F32,
    WASM_F64,
};
typedef struct wasm_val_t {
    wasm_valkind_t kind;
    uint8_t __padding[7];
    union {
        int32_t i32;
        int64_t i64;
        float f32;
        double f64;
    } of;
} wasm_val_t;

#define WASM_INIT_VAL {.kind = WASM_I32, .of = {.i32 = 0}}
#define WASM_I32_VAL(x) {.kind = WASM_I32, .of = {.i32 =(x)}}
#define WASM_I64_VAL(x) {.kind = WASM_I64, .of = {.i64 =(x)}}
#define WASM_F32_VAL(x) {.kind = WASM_F32, .of = {.f32 =(x)}}
#define WASM_F64_VAL(x) {.kind = WASM_F64, .of = {.f64 =(x)}}
