cmake_minimum_required(VERSION 3.14)
project(wasm_executor)

set (WAMR_BUILD_PLATFORM "linux")
set (WAMR_BUILD_TARGET "X86_64")
set (WAMR_BUILD_INTERP 1)
set (WAMR_BUILD_FAST_INTERP 0)
set (WAMR_BUILD_JIT 0)
set (WAMR_BUILD_FAST_JIT 0)
set (WAMR_BUILD_AOT 0)
set (WAMR_BUILD_LIBC_BUILTIN 1)
set (WAMR_BUILD_LIBC_WASI 1)
set (WAMR_BUILD_SIMD 1)
# set (WAMR_BUILD_LIB_WASI_THREADS 1)
set (WAMR_ROOT_DIR ../../wasm-micro-runtime)

include (${WAMR_ROOT_DIR}/build-scripts/runtime_lib.cmake)
add_library(vmlib ${WAMR_RUNTIME_LIB_SOURCE})

add_library(strings SHARED 
    wamr_env.c the-world_bridge.c 
    ${WAMR_ROOT_DIR}/core/shared/utils/uncommon/bh_read_file.c)
target_include_directories(strings PUBLIC ${CMAKE_CURRENT_LIST_DIR} ${WAMR_ROOT_DIR}/core/shared/utils/uncommon)
target_link_libraries (strings vmlib)
