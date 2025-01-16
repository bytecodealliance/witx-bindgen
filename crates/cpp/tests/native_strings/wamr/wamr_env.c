/*
 * Adapted from wasm-micro-runtime/samples/basic/src/main.c
 *
 * Copyright (C) 2019 Intel Corporation.  All rights reserved.
 * SPDX-License-Identifier: Apache-2.0 WITH LLVM-exception
 */

#include "wamr_env.h"
#include "bh_read_file.h"

#define nullptr 0

struct wamr_env *create_wamr_env() {
  struct wamr_env *result = (struct wamr_env *)malloc(sizeof(struct wamr_env));
  char const *wasm_path = "guest_release.wasm";
  const uint32_t stack_size = 65536, heap_size = 2 * stack_size;
  uint32_t buf_size;

  if (!result)
    return nullptr;
  memset(result, 0, sizeof *result);

  RuntimeInitArgs init_args;
  memset(&init_args, 0, sizeof(RuntimeInitArgs));

  init_args.mem_alloc_type = Alloc_With_Pool;
  init_args.mem_alloc_option.pool.heap_buf = result->global_heap_buf;
  init_args.mem_alloc_option.pool.heap_size = sizeof(result->global_heap_buf);
  init_args.running_mode = Mode_Interp;
  if (!wasm_runtime_full_init(&init_args)) {
    printf("Init runtime environment failed.\n");
    return result;
  }

  register_functions();

  wasm_runtime_set_log_level(WASM_LOG_LEVEL_VERBOSE);

  result->buffer = bh_read_file_to_buffer(wasm_path, &buf_size);

  if (!result->buffer) {
    printf("Open wasm app file [%s] failed.\n", wasm_path);
    return result;
  }

  result->module =
      wasm_runtime_load((uint8 *)result->buffer, buf_size, result->error_buf,
                        sizeof(result->error_buf));
  if (!result->module) {
    printf("Load wasm module failed. error: %s\n", result->error_buf);
    return result;
  }

  result->module_inst =
      wasm_runtime_instantiate(result->module, stack_size, heap_size,
                               result->error_buf, sizeof(result->error_buf));

  if (!result->module_inst) {
    printf("Instantiate wasm module failed. error: %s\n", result->error_buf);
    return result;
  }

  result->exec_env =
      wasm_runtime_create_exec_env(result->module_inst, stack_size);
  if (!result->exec_env) {
    printf("Create wasm execution environment failed.\n");
  }

  return result;
}

void free_wamr_env(struct wamr_env *result) {
  if (!result)
    return;
  if (result->exec_env)
    wasm_runtime_destroy_exec_env(result->exec_env);
  if (result->module_inst) {
    wasm_runtime_deinstantiate(result->module_inst);
  }
  if (result->module)
    wasm_runtime_unload(result->module);
  if (result->buffer)
    BH_FREE(result->buffer);
  wasm_runtime_destroy();
  free(result);
}
