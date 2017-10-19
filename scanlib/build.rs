extern crate cc;

use cc::Build;

fn main() {
    println!("cargo:rustc-link-lib=static=scanlib-mt");
    println!("cargo:rustc-link-lib=static=riboost_system-mt");
    println!("cargo:rustc-link-lib=static=riboost_filesystem-mt");
    println!("cargo:rustc-link-search=/usr/local/lib");
    Build::new()
        .cpp(true)
        .flag("-std=c++11")
        .file("src/scanlib.cpp")
        .compile("scanlib_wrapper");
}
