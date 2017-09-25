extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=scanifc-mt");
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .derive_default(true)
        .generate()
        .expect("Unable to generate bindings");
    let path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(path.join("bindings.rs")).expect(
        "Couldn't write bindings!",
    );
}
