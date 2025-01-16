use std::env;

fn main() {
    let out = env::var_os("OUT_DIR").unwrap();
    println!(
        r"cargo:rustc-link-search={}/../../../deps",
        out.into_string().unwrap()
    );
}
