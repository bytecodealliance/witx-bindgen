
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include "wamr_env.h"

#define nullptr 0

typedef struct wamr_env wamr_env;

static wamr_env* instance = nullptr;

wamr_env* get_app() {
  if (!instance) {
    instance = create_wamr_env();
  }
  return instance;
}

uint8_t* guestrelease_memory(wamr_env* wamr_env) {
  return (uint8_t*)wasm_runtime_addr_app_to_native(
        wasm_runtime_get_module_inst(wamr_env->exec_env), 0);
}

uint32_t guestrelease_cabi_realloc(wamr_env* wamr_env, uint32_t olda, uint32_t olds, uint32_t align, uint32_t new_size) {
  WASMFunctionInstanceCommon *cabi_alloc_ptr = wasm_runtime_lookup_function(
                                    wasm_runtime_get_module_inst(wamr_env->exec_env), 
                                    "cabi_realloc");
//                                    "(iiii)i");
  wasm_val_t results[1] = {WASM_INIT_VAL};
    wasm_val_t arguments[4] = {WASM_I32_VAL((int32_t)olda), WASM_I32_VAL((int32_t)olds), WASM_I32_VAL((int32_t)align), WASM_I32_VAL((int32_t)new_size)};

    if (!wasm_runtime_call_wasm_a(wamr_env->exec_env, cabi_alloc_ptr, 1, results, 4, arguments))
    {
        const char *exception;
        if ((exception = wasm_runtime_get_exception(wasm_runtime_get_module_inst(wamr_env->exec_env))))
        {
            printf("Exception: %s\n", exception);
        }
        return 0;
    }
    return results[0].of.i32;
}

__attribute__ ((visibility ("default"))) 
void *cabi_realloc(void *ptr, size_t old_size, size_t align,
                              size_t new_size) {
  uint8_t *linmem = guestrelease_memory(get_app());
  uint32_t result = guestrelease_cabi_realloc(get_app(), ptr ? (uint8_t*)ptr-linmem : 0, old_size, align, new_size);
  return result+linmem;
}

// Import IF strings
// Func a GuestImport
extern void fooX3AfooX2FstringsX00a(uint8_t *arg0, size_t arg1);
void fooX3AfooX2Fstrings__a(void*app,uint8_t* arg0,uint32_t arg1) {
  //uint8_t *linmem = guestrelease_memory(get_app());
  fooX3AfooX2FstringsX00a(arg0, arg1);
}

// Func b GuestImport
extern void fooX3AfooX2FstringsX00b(uint8_t *arg0);
void fooX3AfooX2Fstrings__b(void*app,uint8_t* arg0) {
  uint8_t *linmem = guestrelease_memory(get_app());
  static size_t result[2];
  fooX3AfooX2FstringsX00b((uint8_t*)&result);
  uint32_t *result_out = (uint32_t*)(arg0);
  result_out[0] = ((uint8_t*)(result[0]))-linmem;
  result_out[1] = result[1];
}
// Func c GuestImport
extern void fooX3AfooX2FstringsX00c(uint8_t *arg0, size_t arg1,
                                        uint8_t *arg2, size_t arg3,
                                        uint8_t *arg4);
void fooX3AfooX2Fstrings__c(void*app,uint8_t* arg0,uint32_t arg1,uint8_t* arg2,uint32_t arg3,uint8_t* arg4) {
  uint8_t *linmem = guestrelease_memory(get_app());
  static size_t result[2];
  fooX3AfooX2FstringsX00c(arg0, arg1, arg2, arg3, (uint8_t*)&result);
  uint32_t *result_out = (uint32_t*)(arg4);
  result_out[0] = ((uint8_t*)(result[0]))-linmem;
  result_out[1] = result[1];
}
// Export IF strings
// Func a GuestExport
__attribute__ ((visibility ("default"))) 
void fooX3AfooX2FstringsX23a(uint8_t *arg0, size_t arg1) {
  wamr_env *wamr_env = get_app();
  uint8_t *linmem = guestrelease_memory(wamr_env);
  WASMFunctionInstanceCommon *func_ptr = wasm_runtime_lookup_function(
                                    wasm_runtime_get_module_inst(wamr_env->exec_env), 
                                    "foo:foo/strings#a");
    wasm_val_t arguments[2] = {WASM_I32_VAL((int32_t)(arg0-linmem)), WASM_I32_VAL((int32_t)arg1)};
    if (!wasm_runtime_call_wasm_a(wamr_env->exec_env, func_ptr, 0, nullptr, 2, arguments))
    {
        const char *exception;
        if ((exception = wasm_runtime_get_exception(wasm_runtime_get_module_inst(wamr_env->exec_env))))
        {
            printf("Exception: %s\n", exception);
        }
    }
}
// Func b GuestExport
__attribute__ ((visibility ("default"))) uint8_t *
fooX3AfooX2FstringsX23b() {
  wamr_env *wamr_env = get_app();
  uint8_t *linmem = guestrelease_memory(wamr_env);
  WASMFunctionInstanceCommon *func_ptr = wasm_runtime_lookup_function(
                                    wasm_runtime_get_module_inst(wamr_env->exec_env), 
                                    "foo:foo/strings#b");
    wasm_val_t results[1] = {WASM_INIT_VAL};
    if (!wasm_runtime_call_wasm_a(wamr_env->exec_env, func_ptr, 1, results, 0, nullptr))
    {
        const char *exception;
        if ((exception = wasm_runtime_get_exception(wasm_runtime_get_module_inst(wamr_env->exec_env))))
        {
            printf("Exception: %s\n", exception);
        }
    }
  uint32_t result = results[0].of.i32;
  static size_t ret_area[3];
  ret_area[0] = (size_t)(((uint32_t*)(linmem+result))[0]+linmem);
  ret_area[1] = ((uint32_t*)(linmem+result))[1];
  ret_area[2] = result;
  return (uint8_t*)ret_area;
}
__attribute__ ((visibility ("default"))) 
void cabi_post_fooX3AfooX2FstringsX23b(uint8_t * arg0) {
  wamr_env *wamr_env = get_app();
  // uint8_t *linmem = guestrelease_memory(wamr_env);
  WASMFunctionInstanceCommon *func_ptr = wasm_runtime_lookup_function(
                                    wasm_runtime_get_module_inst(wamr_env->exec_env), 
                                    "cabi_post_fooX3AfooX2FstringsX23b");
    wasm_val_t arguments[1] = {WASM_I32_VAL((int32_t)(((size_t*)arg0)[2]))};
    if (!wasm_runtime_call_wasm_a(wamr_env->exec_env, func_ptr, 0, nullptr, 1, arguments))
    {
        const char *exception;
        if ((exception = wasm_runtime_get_exception(wasm_runtime_get_module_inst(wamr_env->exec_env))))
        {
            printf("Exception: %s\n", exception);
        }
    }
}
// Func c GuestExport
__attribute__ ((visibility ("default"))) 
uint8_t * fooX3AfooX2FstringsX23c(uint8_t * arg0, size_t arg1, uint8_t *arg2, size_t arg3) {
  wamr_env *wamr_env = get_app();
  uint8_t *linmem = guestrelease_memory(wamr_env);
  WASMFunctionInstanceCommon *func_ptr = wasm_runtime_lookup_function(
                                    wasm_runtime_get_module_inst(wamr_env->exec_env), 
                                    "foo:foo/strings#c");
    wasm_val_t results[1] = {WASM_INIT_VAL};
    wasm_val_t arguments[4] = {WASM_I32_VAL((int32_t)(arg0-linmem)), WASM_I32_VAL((int32_t)arg1), WASM_I32_VAL((int32_t)(arg2-linmem)), WASM_I32_VAL((int32_t)arg3)};

    if (!wasm_runtime_call_wasm_a(wamr_env->exec_env, func_ptr, 1, results, 4, arguments))
    {
        const char *exception;
        if ((exception = wasm_runtime_get_exception(wasm_runtime_get_module_inst(wamr_env->exec_env))))
        {
            printf("Exception: %s\n", exception);
        }
        return 0;
    }
  uint32_t result = results[0].of.i32;
  // arg0-linmem, arg1, arg2-linmem, arg3);
  static size_t ret_area[3];
  ret_area[0] = (size_t)(((uint32_t*)(linmem+result))[0]+linmem);
  ret_area[1] = ((uint32_t*)(linmem+result))[1];
  ret_area[2] = result;
  return (uint8_t*)ret_area;
}
__attribute__ ((visibility ("default"))) 
extern void
cabi_post_fooX3AfooX2FstringsX23c(uint8_t * arg0) {
  wamr_env *wamr_env = get_app();
  // uint8_t *linmem = guestrelease_memory(wamr_env);
  WASMFunctionInstanceCommon *func_ptr = wasm_runtime_lookup_function(
                                    wasm_runtime_get_module_inst(wamr_env->exec_env), 
                                    "cabi_post_fooX3AfooX2FstringsX23c");
    wasm_val_t arguments[1] = {WASM_I32_VAL((int32_t)(((size_t*)arg0)[2]))};
    if (!wasm_runtime_call_wasm_a(wamr_env->exec_env, func_ptr, 0, nullptr, 1, arguments))
    {
        const char *exception;
        if ((exception = wasm_runtime_get_exception(wasm_runtime_get_module_inst(wamr_env->exec_env))))
        {
            printf("Exception: %s\n", exception);
        }
    }
}

//#include "executor.h"

void register_functions() {
  static NativeSymbol foo_foo_strings_funs[] = {
      {"a", (void *)fooX3AfooX2Fstrings__a, "(*~)", nullptr},
      {"b", (void *)fooX3AfooX2Fstrings__b, "(*)", nullptr},
      {"c", (void *)fooX3AfooX2Fstrings__c, "(*~*~*)", nullptr},
  };
  wasm_runtime_register_natives("foo:foo/strings", foo_foo_strings_funs,
                                sizeof(foo_foo_strings_funs) /
                                    sizeof(NativeSymbol));
}
