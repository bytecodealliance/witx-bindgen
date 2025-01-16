#!/bin/sh
cargo component build
wasm-tools component new target/wasm32-wasi/debug/resources.wasm -o component.wasm --adapt ~/Downloads/wasi_snapshot_preview1.reactor\(1\).wasm 
jco transpile component.wasm -o html --no-typescript --no-wasi-shim     --map wasi:filesystem/*=./bytecodealliance/preview2-shim/filesystem.js     --map wasi:cli/*=./bytecodealliance/preview2-shim/cli.js     --map wasi:cli-base/*=./bytecodealliance/preview2-shim/cli.js     --map wasi:io/*=./bytecodealliance/preview2-shim/io.js     --map test:example/my-interface=./test_example/my-interface.js     --map foo:foo/resources=./resources.js 
