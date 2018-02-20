# rivlib-rs

Use [Riegl's](www.riegl.com/) [RiVLib](http://www.riegl.com/index.php?id=224) via [Rust](https://www.rust-lang.org/).

**Note: This software was not developed by Riegl.
Please do not contact Riegl for support related to this software.**

This crate is divided into three sub-crates:

- **scanifc-sys**: [`-sys`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#-sys-packages) crate for the C `scanifc` library, part of RiVLib.
- **scanifc**: wrapper crate around the sys crate to improve the interface.
- **scanlib**: library that uses custom C++ wrapper files to interface with `scanlib`, RiVLib's C++ interface.
  There are some functions that are only exposed in the C++ interface, not in the C interface.

This crate is very much a work in progress, there's a lot of functionality not exposed.

## Examples

A complete example of using `scanifc` to count the number of points in a files is in `scanifc/examples/count-points.rs`.

To open a stream of points:

```rust
let stream = scanifc::point3d::Stream::from_path("path/to/file.rxp").open().unwrap();
```

You can specify if the stream should be opened with `sync_to_pps`, which controls whether points are included if they don't include pulse-per-second-sync information:

```rust
scanifc::point3d::Stream::from_path("file.rxp").sync_to_pps(false).open().unwrap();
```

To read inclination data from a file, use `inclinations_from_path`:

```rust
let inclinations = scanlib::inclinations_from_path("file.rxp", false).unwrap();
```
