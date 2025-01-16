use std::env;

fn main() {
    let source_dir = env::var("OUT_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}/deps", source_dir);
    println!("cargo:rustc-link-lib=dylib=rust_comp_b");
}
