//! Count the number of points in the file twice, once with sync-to-pps and once without.

extern crate scanifc;

use scanifc::point3d::Stream;

fn main() {
    let ref path = std::env::args().skip(1).next().expect(
        "Must have one argument (the filename)",
    );
    let with = Stream::from_path(path)
        .sync_to_pps(true)
        .open()
        .unwrap()
        .into_points()
        .unwrap()
        .len();
    println!("With sync_to_pps: {}", with);
    let without = Stream::from_path(path)
        .sync_to_pps(false)
        .open()
        .unwrap()
        .into_points()
        .unwrap()
        .len();
    println!("Without sync_to_pps: {}", without);
}
