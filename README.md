# rivlib-rs

Use [Riegl's](www.riegl.com/) [RiVLib](http://www.riegl.com/index.php?id=224) via [Rust](https://www.rust-lang.org/).

**Note: This software was not developed by Riegl.
Please do not contact Riegl for support related to this software.**

To use, you'll need to have RiVLib installed somewhere on your library search path, e.g. `/usr/local/lib`.
You'll also need the headers installed to somewhere on your include file search path, e.g. `/usr/local/include`.

## Sub-crates

Underneath the `rivlib` Rust lib, there are two sub-crates:

- *scanifc-sys* uses [bindgen](https://github.com/rust-lang-nursery/rust-bindgen) to build Rust bindings to RiVLib's C interface, `scanifc`.
- *scanlib* uses a custom C++ wrapper to expose functionality from RiVLib's C++ interface, `scanlib`.
  This C++ wrapper must be compiled on your machine when you're building the `rivlib` crate.
