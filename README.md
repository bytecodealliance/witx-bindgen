<div align="center">
  <h1><code>wit-bindgen</code></h1>

  <p>
    <strong>Guest language bindings generator for
    <a href="https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md">WIT</a>
    and the
    <a href="https://github.com/WebAssembly/component-model">Component Model</a>
    </strong>
  </p>

<strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/wit-bindgen/actions?query=workflow%3ACI"><img src="https://github.com/bytecodealliance/wit-bindgen/workflows/CI/badge.svg" alt="build status" /></a>
    <img src="https://img.shields.io/badge/rustc-stable+-green.svg" alt="supported rustc stable" />
  </p>
</div>

## About

[zulip]: https://bytecodealliance.zulipchat.com/#narrow/stream/327223-wit-bindgen

This project is a suite of bindings generators for languages that are compiled
to WebAssembly and use the [component model]. Bindings are described with
[`*.wit` files][WIT] which specify imports, exports, and facilitate reuse
between bindings definitions.

[WIT]: https://component-model.bytecodealliance.org/design/wit.html
[component model]: https://github.com/WebAssembly/component-model

The `wit-bindgen` repository is currently focused on **guest** programs which
are those compiled to WebAssembly. Executing a component in a host is not
managed in this repository, and some options of how to do so are [described
below][hosts]. Languages developed in this repository are Rust, C, Java (TeaVM
Java), Go (TinyGo), and C#. If you encounter any problems feel free to [open an
issue](https://github.com/bytecodealliance/wit-bindgen/issues/new) or chat with
us on [Zulip][zulip].

## [WIT] as an IDL

The `wit-bindgen` project extensively uses [WIT] definitions to describe imports
and exports. The items supported by [WIT] directly map to the component model
which allows core WebAssembly binaries produced by native compilers to be
transformed into a component. All imports into a WebAssembly binary and all
exports must be described with [WIT]. An example file looks like:

```wit
package example:host;

world host {
  import print: func(msg: string);

  export run: func();
}
```

This describes a "world" which describes both imports and exports that the
WebAssembly component will have available. In this case the host will provide a
`print` function and the component itself will provide a `run` function.

Functionality in [WIT] can also be organized into `interface`s:

```wit
package example:my-game;

interface my-plugin-api {
  record coord {
    x: u32,
    y: u32,
  }

  get-position: func() -> coord;
  set-position: func(pos: coord);

  record monster {
    name: string,
    hp: u32,
    pos: coord,
  }

  monsters: func() -> list<monster>;
}

world my-game {
  import print: func(msg: string);
  import my-plugin-api;

  export run: func();
}
```

Here the `my-plugin-api` interface encapsulates a group of functions, types,
etc. This can then be imported wholesale into the `my-game` world via the
`my-plugin-api` namespace. The structure of a [WIT] document and world will affect the
generated bindings per-language.

For more information about WIT and its syntax see the [online documentation for
WIT][WIT] as well as its [upstream
reference](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md).

## Creating a Component

The end-goal of `wit-bindgen` is to facilitate creation of a
[component][component model]. Once a component is created it can then be handed
off to any one of a number of [host runtimes][hosts] for execution. Creating a
component is not supported natively by any language today, however, so
`wit-bindgen` is only one of the pieces in the process of creating a component.
The general outline for the build process of a component for a compiled language
is:

1. Using `wit-bindgen` source code for the language is generated representing
   bindings to the specified APIs. This source code is then compiled by the
   native compiler and used by user-written code as well.
2. The native language toolchain is used to emit a core WebAssembly module. This
   core wasm module is the "meat" of a component and contains all user-defined
   code compiled to WebAssembly. The most common native target to use for
   compilation today is the `wasm32-wasip1` target.
3. The output core wasm module is transformed into a component using the
   [`wasm-tools`] project, notably the `wasm-tools component new` subcommand.
   This will ingest the native core wasm output and wrap the output into the
   component model binary format.

[`wasm-tools`]: https://github.com/bytecodealliance/wasm-tools

The precise tooling and commands at each of these steps [differs language by
language][guests], but this is the general idea. With a component in-hand the
binary can then be handed off to [a host runtimes][hosts] for execution.

### Creating components: WASI

An important consideration when creating a component today is WASI. All current
native toolchains for languages which have WASI support are using the
`wasi_snapshot_preview1` version of WASI. This definition of WASI was made
with historical `*.witx` files and is not compatible with the component model.
There is, however, a means by which to still create components from modules
that are using `wasi_snapshot_preview1` APIs.

The `wasm-tools component new` subcommand takes an `--adapt` argument which acts
as a way to polyfill non-component-model APIs, like `wasi_snapshot_preview1`,
with component model APIs. The [Wasmtime] runtime publishes [adapter
modules][preview1-build] with each release that are suitable to use with
`--adapt` to implement `wasi_snapshot_preview1` in terms of WASI 0.2. On
Wasmtime's releases page you'll see three modules to choose from:

* [`wasi_snapshot_preview1.command.wasm`] - use this for CLI applications.
* [`wasi_snapshot_preview1.reactor.wasm`] - use this for applications that don't
  have a `main` function for example: for example a process that responds to an
  event.
* [`wasi_snapshot_preview1.proxy.wasm`] - use this for applications fed into
  `wasmtime serve` for example.

Only one adapter is necessary and be sure to look for the [latest
versions][preview1-build] as well.

[preview1-build]: https://github.com/bytecodealliance/wasmtime/releases/latest
[wasmtime]: https://github.com/bytecodealliance/wasmtime
[`wasi_snapshot_preview1.command.wasm`]: https://github.com/bytecodealliance/wasmtime/releases/download/v17.0.0/wasi_snapshot_preview1.command.wasm
[`wasi_snapshot_preview1.reactor.wasm`]: https://github.com/bytecodealliance/wasmtime/releases/download/v17.0.0/wasi_snapshot_preview1.reactor.wasm
[`wasi_snapshot_preview1.proxy.wasm`]: https://github.com/bytecodealliance/wasmtime/releases/download/v17.0.0/wasi_snapshot_preview1.proxy.wasm

## Supported Guest Languages

[guests]: #supported-guest-languages

The `wit-bindgen` project is primarily focused on **guest** languages which are
those compiled to WebAssembly. Each language here already has native support for
execution in WebAssembly at the core wasm layer (e.g. targets the current [core
wasm specification](https://webassembly.github.io/spec/)). Brief instructions
are listed here for each language of how to use it as well.

Each project below will assume the following `*.wit` file in the root of your
project.

```wit
// wit/host.wit
package example:host;

world host {
  import print: func(msg: string);

  export run: func();
}
```

### Guest: Rust

The Rust compiler since version 1.82 supports a native `wasm32-wasip2` target and can be added to
any `rustup`-based toolchain with:

```sh
rustup target add wasm32-wasip2
```

In order to compile a wasi dynamic library, the following must be added to the
`Cargo.toml` file:

```toml
[lib]
crate-type = ["cdylib"]
```

Projects can then depend on `wit-bindgen` by executing:

```sh
cargo add wit-bindgen
```

WIT files are currently added to a `wit/` folder adjacent to your `Cargo.toml`
file. Example code using this then looks like:

```rust
// src/lib.rs

// Use a procedural macro to generate bindings for the world we specified in
// `host.wit`
wit_bindgen::generate!({
    // the name of the world in the `*.wit` input file
    world: "host",
});

// Define a custom type and implement the generated `Guest` trait for it which
// represents implementing all the necessary exported interfaces for this
// component.
struct MyHost;

impl Guest for MyHost {
    fn run() {
        print("Hello, world!");
    }
}

// export! defines that the `MyHost` struct defined below is going to define
// the exports of the `world`, namely the `run` function.
export!(MyHost);
```

By using [`cargo expand`](https://github.com/dtolnay/cargo-expand) or `cargo
doc` you can also explore the generated code. If there's a bug in `wit-bindgen`
and the generated bindings do not compile or if there's an error in the
generated code (which is probably also a bug in `wit-bindgen`), you can use
`WIT_BINDGEN_DEBUG=1` as an environment variable to help debug this.

This project can then be built with:

```sh
cargo build --target wasm32-wasip2
```

This creates a `./target/wasm32-wasip2/debug/my-project.wasm` file which is suitable to execute in any
component runtime. Using `wasm-tools` you can inspect the binary as well, for
example inferring the WIT world that is the component:

```sh
wasm-tools component wit ./target/wasm32-wasip2/debug/my-project.wasm
# world my-component {
#  import print: func(msg: string)
#  export run: func()
# }
```

which in this case, as expected, is the same as the input world.

### Guest: C/C++

C and C++ code can be compiled for the `wasm32-wasip1` target using the [WASI
SDK] project. The releases on that repository have precompiled `clang` binaries
which are pre-configured to compile for WebAssembly.

[WASI SDK]: https://github.com/webassembly/wasi-sdk

To start in C and C++ a `*.c` and `*.h` header file is generated for your
project to use. These files are generated with the [`wit-bindgen` CLI
command][cli-install] in this repository.

```sh
wit-bindgen c ./wit
# Generating "host.c"
# Generating "host.h"
# Generating "host_component_type.o"
```

Some example code using this would then look like

```c
// my-component.c

#include "host.h"

void host_run() {
    host_string_t my_string;
    host_string_set(&my_string, "Hello, world!");

    host_print(&my_string);
}
```

This can then be compiled with `clang` from the [WASI SDK] and assembled into a
component with:

```sh
clang host.c host_component_type.o my-component.c -o my-core.wasm -mexec-model=reactor
wasm-tools component new ./my-core.wasm -o my-component.wasm
```

Like with Rust, you can then inspect the output binary:

```sh
wasm-tools component wit ./my-component.wasm
```

### Guest C#

To generate the bindings:

```
wit-bindgen c-sharp -w command -r native-aot --generate-stub wit/
```

Now you create a c# project file:

```
dotnet new console -o MyApp
cd MyApp
dotnet new nugetconfig
```

In the `nuget.config` after `<clear />`make sure you have: 

```
<add key="dotnet-experimental" value="https://pkgs.dev.azure.com/dnceng/public/_packaging/dotnet-experimental/nuget/v3/index.json" />
<add key="nuget" value="https://api.nuget.org/v3/index.json" />
```

In the MyApp.csproj add the following to the property group:

```
<RuntimeIdentifier>wasi-wasm</RuntimeIdentifier>
<UseAppHost>false</UseAppHost>
<PublishTrimmed>true</PublishTrimmed>
<InvariantGlobalization>true</InvariantGlobalization>
<SelfContained>true</SelfContained>
<AllowUnsafeBlocks>true</AllowUnsafeBlocks>
<WASI_SDK_PATH>path/to/wasi-sdk</WASI_SDK_PATH>
```

Add the native-aot compiler (substitute `win-x64` for `linux-x64` on Linux):

```
dotnet add package Microsoft.DotNet.ILCompiler.LLVM --prerelease 
dotnet add package runtime.win-x64.Microsoft.DotNet.ILCompiler.LLVM --prerelease
```

Now you can build with:

```
dotnet publish
```

Checkout out [componentize-dotnet](https://github.com/bytecodealliance/componentize-dotnet) for a simplified experience.

### Guest: Java

Java bytecode can be compiled to WebAssembly using
[TeaVM-WASI](https://github.com/fermyon/teavm-wasi). With this generator,
`wit-bindgen` will emit `*.java` files which may be used with any JVM language,
e.g. Java, Kotlin, Clojure, Scala, etc.

### Guest: TinyGo

The **new** TinyGo WIT bindings generator is currently in development at the
[wasm-tools-go](https://github.com/bytecodealliance/wasm-tools-go) repository.

To install the `wit-bindgen-go` CLI, run:

```sh
go install github.com/bytecodealliance/wasm-tools-go/cmd/wit-bindgen-go
```
> Note: it requires `wasm-tools` to be installed.

Then, you can generate the bindings for your project:

```sh
wit-bindgen-go generate <path-to-wit-pkg>
```

### Guest: C++-17+

This fork contains code to generate C++ code which uses the std types 
optional, string, string_view, vector, expected to represent generic
WIT types.

This relies on wasi-SDK for guest compilation.

A separate subcommand (cpp-host) will generate C++ host code for 
WebAssembly micro runtime.

### Guest: MoonBit

MoonBit can be compiled to WebAssembly using [its toolchain](https://moonbitlang.com/download):

```sh
moon build --target wasm # --debug to keep symbols
```

The generated core wasm will be found under `target/wasm/release/build/gen/gen.wasm` by default. Then you can use `wasm-tools` to componentize the module:

```
wasm-tools component embed wit target/wasm/release/build/gen/gen.wasm -o target/gen.wasm
wasm-tools component new target/gen.wasm -o target/gen.component.wasm
```

You may use `--gen-dir` to specify which package should be responsible for the exportation. The default is `gen` as mentioned above.
This can be useful having one project that exports multiple worlds.

When using `wit-bindgen moonbit`, you may use `--derive-show` or `--derive-eq` to derive `Show` or `Eq` traits for all types.
You may also use `--derive-error`, which will make types containing `Error` as error types in MoonBit.

You will find the files to be modified with the name `**/stub.mbt`.
To avoid touching the files during regeneration (including `moon.pkg.json` or `moon.mod.json`) you may use `--ignore-stub`.

/!\ MoonBit is still evolving, so please check out the [Weekly Updates](https://www.moonbitlang.com/weekly-updates/) for any breaking changes or deprecations.

### Guest: Other Languages

Guest component support for JavaScript and Python is available in
[componentize-js](https://github.com/bytecodealliance/ComponentizeJS) and
[componentize-py](https://github.com/bytecodealliance/componentize-py), respectively.
See also
[The WebAssembly Component Model developer's guide](https://component-model.bytecodealliance.org/language-support.html)
for examples of how to build components using various languages.

Other languages such as Ruby, etc, are hoped to be supported one day
with `wit-bindgen` or with components in general. It's recommended to reach out
on [zulip] if you're intersted in contributing a generator for one of these
langauges. It's worth noting, however, that turning an interpreted language into
a component is significantly different from how compiled languages currently
work (e.g. Rust or C/C++). It's expected that the first interpreted language
will require a lot of design work, but once that's implemented the others can
ideally relatively quickly follow suit and stay within the confines of the
first design.

## CLI Installation

[cli-install]: #cli-installation

To install the CLI for this tool (which isn't the only way it can be used), run
the following cargo command. This will let you generate the bindings for any
supported language.

```
cargo install wit-bindgen-cli
```

This CLI **IS NOT** stable and may change, do not expect it to be or rely on it
being stable. Please reach out to us on [zulip] if you'd like to depend on it,
so we can figure out a better alternative for your use case.

## Host Runtimes for Components

[hosts]: #host-runtimes-for-components

The `wit-bindgen` project is intended to facilitate in generating a component,
but once a component is in your hands the next thing to do is to actually
execute that somewhere. This is not under the purview of `wit-bindgen` itself
but these are some resources and runtimes which can help you work with
components:

- Rust: the [`wasmtime` crate](https://docs.rs/wasmtime) is an implementation of
  a native component runtime that can run any WIT `world`. It additionally comes
  with a [`bindgen!`
  macro](https://docs.rs/wasmtime/latest/wasmtime/component/macro.bindgen.html)
  which acts similar to the `generate!` macro in this repository. This macro
  takes a [WIT] package as input and generates `trait`-based bindings for the
  runtime to implement and use.

- JS: the [`jco`] project can be used to execute components in JS
  either on the web or outside the browser in a runtime such as `node`. This
  project generates a polyfill for a single concrete component to execute in a
  JS environment by extracting the core WebAssembly modules that make up a
  component and generating JS glue to interact between the host and these
  modules.

- Python: the [`wasmtime`](https://github.com/bytecodealliance/wasmtime-py)
  project [on PyPI](https://pypi.org/project/wasmtime/) has a `bindgen` mode
  that works similar to the JS integration. Given a concrete component this will
  generate Python source code to interact with the component using an embedding
  of Wasmtime for its core WebAssembly support.

- C++-17+: see above chapter for WAMR host code generation.

- Tooling: the [`wasm-tools`] project can be used to inspect and modify
  low-level details of components. For example as previously mentioned you can
  inspect the WIT-based interface of a component with `wasm-tools component
wit`. You can link two components together with `wasm-tools compose` as well.

[`jco`]: https://github.com/bytecodealliance/jco

Note that the runtimes above are generally intended to work with arbitrary
components, not necessarily only those created by `wit-bindgen`. This is also
not necessarily an exhaustive listing of what can execute a component.

## Building and Testing

To build the cli:

```
cargo build
```

Learn more how to run the tests in the [testing document](tests/README.md).

# Versioning and Releases

This repository's crates and CLI are all currently versioned at `0.X.Y` where
`Y` is frequently `0` and `X` increases most of the time with publishes. This
means that changes are published as possibly-API-breaking changes as development
continues here.

Also, this repository does not currently have a strict release cadence. Releases
are done on an as-needed basis. If you'd like a release done please feel free to
reach out on [Zulip], file an issue, leave a comment on a PR, or otherwise
contact a maintainer.

[Zulip]: https://bytecodealliance.zulipchat.com/

For maintainers, the release process looks like:

* Go to [this link](https://github.com/bytecodealliance/wit-bindgen/actions/workflows/release-process.yml)
* Click on "Run workflow" in the UI.
* Use the default `bump` argument and hit "Run workflow"
* Wait for a PR to be created by CI. You can watch the "Actions" tab for if
  things go wrong.
* When the PR opens, close it then reopen it. Don't ask questions.
* Review the PR, approve it, then queue it for merge.

That should be it, but be sure to keep an eye on CI in case anything goes wrong.

# License

This project is triple licenced under the Apache 2/ Apache 2 with LLVM exceptions/ MIT licences. The reasoning for this is:
- Apache 2/ MIT is common in the rust ecosystem.
- Apache 2/ MIT is used in the rust standard library, and some of this code may be migrated there.
- Some of this code may be used in compiler output, and the Apache 2 with LLVM exceptions licence is useful for this.

For more details see
- [Apache 2 Licence](LICENSE-APACHE)
- [Apache 2 Licence with LLVM exceptions](LICENSE-Apache-2.0_WITH_LLVM-exception)
- [MIT Licence](LICENSE-MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache 2/ Apache 2 with LLVM exceptions/ MIT licenses,
shall be licensed as above, without any additional terms or conditions.
