#pragma once
// minimal WAMR header mock-up for compilation tests
#include <stdint.h>
struct WASMExecEnv;
typedef WASMExecEnv* wasm_exec_env_t;
struct WASMModuleInstanceCommon;
typedef WASMModuleInstanceCommon* wasm_module_inst_t;
typedef void* wasm_function_inst_t;
wasm_module_inst_t wasm_runtime_get_module_inst(wasm_exec_env_t);
void* wasm_runtime_addr_app_to_native(wasm_module_inst_t,int32_t);
struct NativeSymbol {
    const char* name;
    void* func;
    const char* signature;
    void* env;
};
void wasm_runtime_register_natives(char const* module, NativeSymbol const*, unsigned);
bool wasm_runtime_call_wasm_a(wasm_exec_env_t, wasm_function_inst_t, uint32_t, struct wasm_val_t*, uint32_t, struct wasm_val_t*);
wasm_function_inst_t wasm_runtime_lookup_function(wasm_module_inst_t, const char*, const char*);
