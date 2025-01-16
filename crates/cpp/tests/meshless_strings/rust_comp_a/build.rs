use std::env;

fn main() {
    let source_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo::rustc-link-search=native={}/../component_b",
        source_dir
    );
    println!("cargo::rustc-link-lib=static=component_b");
    println!("cargo::rustc-link-lib=static=stdc++");
}
