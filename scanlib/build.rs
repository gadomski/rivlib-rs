extern crate cc;

fn main() {
    println!("cargo:rustc-link-lib=scanlib-mt");
    println!("cargo:rustc-link-lib=riboost_system-mt");
    println!("cargo:rustc-link-lib=riboost_filesystem-mt");
    cc::Build::new()
        .cpp(true)
        .flag("-std=c++11")
        .file("src/pointcloud.cpp")
        .compile("scanlib_wrapper");
}
