use heck::*;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

macro_rules! codegen_test {
    ($id:ident $name:tt $test:tt) => {
        #[test]
        fn $id() {
            if [
                "go_params",
                "guest-name",
                "import-func",
                "import-and-export-resource",
                "import-and-export-resource-alias",
                "interface-has-go-keyword",
                "issue551",
                "issue573",
                "issue607",
                "issue668",
                "issue929",
                "issue929-no-export",
                "issue929-no-import",
                "issue929-only-methods",
                "small-anonymous",
                "wasi-cli",
                "wasi-clocks",
                "wasi-filesystem",
                "wasi-http",
                "wasi-io",
                "keywords",
                "lift-lower-foreign",
                "lists",
                "multi-return",
                "multiversion",
                "option-result",
                "record-has-go-keyword-and-used-in-fn",
                "resource-alias",
                "resource-borrow-in-record",
                "resource-borrow-in-record-export",
                "resource-local-alias",
                "resource-local-alias-borrow",
                "resource-local-alias-borrow-import",
                "resource-own-in-other-interface",
                "resources",
                "resources-in-aggregates",
                "resources-with-lists",
                "result-empty",
                "ret-areas",
                "return-resource-from-export",
                "same-names5",
                "simple-http",
                // "simple-lists",
                "use-across-interfaces",
                "variants",
                "variants-unioning-types",
                "worlds-with-types",
                "zero-size-tuple",
            ]
            .contains(&$name)
            {
                let test_all_code = env::var_os("CPP_ALL_TESTS").is_some();
                if !test_all_code {
                    return;
                }
            }
            test_helpers::run_world_codegen_test(
                "cpp",
                $test.as_ref(),
                |resolve, world, files| {
                    let mut opts = wit_bindgen_cpp::Opts::default();
                    opts.new_api = true;
                    opts.build().generate(resolve, world, files).unwrap()
                },
                verify,
            );
            let test_host_code = env::var_os("CPP_HOST_TESTS").is_some();
            if test_host_code {
                test_helpers::run_world_codegen_test(
                    "cpp-host",
                    $test.as_ref(),
                    |resolve, world, files| {
                        let mut opts = wit_bindgen_cpp::Opts::default();
                        opts.host = true;
                        opts.build().generate(resolve, world, files).unwrap()
                    },
                    verify_host,
                );
            }
        }
    };
}

test_helpers::codegen_tests!();

fn verify(dir: &Path, name: &str) {
    let name = name.to_snake_case();
    let sdk_path = PathBuf::from(
        env::var_os("WASI_SDK_PATH").expect("environment variable WASI_SDK_PATH should be set"),
    );
    let sysroot = sdk_path.join("share/wasi-sysroot");
    let c_src = dir.join(format!("{name}.cpp"));
    let additional_includes = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .expect("environment variable CARGO_MANIFEST_DIR should get set by cargo"),
    )
    .join("test_headers");

    let shared_args = vec![
        "--sysroot",
        sysroot.to_str().unwrap(),
        "-I",
        dir.to_str().unwrap(),
        "-I",
        additional_includes.to_str().unwrap(),
        // "-Wall",
        // "-Wextra",
        // "-Werror",
        // "-Wno-unused-parameter",
        "-std=c++2b",
        "-c",
        "-o",
    ];

    let mut cmd = Command::new(sdk_path.join("bin/clang++"));
    cmd.args(&shared_args);
    cmd.arg(dir.join("obj.o"));
    cmd.arg(&c_src);
    test_helpers::run_command(&mut cmd);
}

fn verify_host(dir: &Path, name: &str) {
    let name = name.to_snake_case();
    let c_src = dir.join(format!("{name}_host.cpp"));
    let additional_includes = PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR")
            .expect("environment variable CARGO_MANIFEST_DIR should get set by cargo"),
    )
    .join("test_headers");

    let shared_args = vec![
        "-I",
        dir.to_str().unwrap(),
        "-I",
        additional_includes.to_str().unwrap(),
        // "-Wall",
        // "-Wextra",
        // "-Werror",
        // "-Wno-unused-parameter",
        "-std=c++2b",
        "-c",
        "-o",
    ];

    let mut cmd = Command::new("clang++");
    cmd.args(&shared_args);
    cmd.arg(dir.join("obj.o"));
    cmd.arg(&c_src);
    test_helpers::run_command(&mut cmd);
}
