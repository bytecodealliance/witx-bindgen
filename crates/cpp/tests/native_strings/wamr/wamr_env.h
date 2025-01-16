#pragma once
#include "wasm_c_api.h"
#include "wasm_export.h"

void register_functions();

struct wamr_env {
  char global_heap_buf[512 * 1024];
  char *buffer;
  char error_buf[128];

  wasm_module_t module;
  wasm_module_inst_t module_inst;
  wasm_exec_env_t exec_env;
};

struct wamr_env *create_wamr_env();
void free_wamr_env(struct wamr_env *);
