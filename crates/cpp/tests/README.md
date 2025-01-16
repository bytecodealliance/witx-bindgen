
This folder contains examples on how to use the canonical ABI without
a wasm32 target.

The `native_strings` folder contains an example of passing strings, with
the guest in C++ and Rust, the host in C++, and in the w2c folder an
example of a wasm component transpiled to C and then executed natively.
The wamr folder creates a fully binary compatible shared object linking to
wasm-micro-runtime and interpreting the wasm binary.

Please note that this demonstrates that native compilation, wasm2c and wamr are
binary compatible and fully exchangeable.

Sadly the [w2c2](https://github.com/turbolent/w2c2) bridge code generation isn't yet complete.

The `native_resources` folder shows a more complex example using resources,
both guest and host defined ones. This doesn't include a wasm2c deployment.

The `native_mesh` folder shows an example with resources and more than one
component. Optimizing this is work in progress.

The `meshless_resources` and `meshless_strings` folders experiment
with directly linking two components in a shared everything environment.
