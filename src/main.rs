extern crate docopt;
extern crate rivlib;
extern crate rustc_serialize;

use std::process::exit;

use docopt::Docopt;

use rivlib::scanifc;

const USAGE: &'static str = "
Get information about a data stream.

Usage:
    rivlib info \
                             <stream> [--sync-to-pps=<bool>]
    rivlib (-h | --help)
    rivlib \
                             --version

Options:
    -h --help                   Display this \
                             message.
    --version                   Display version information \
                             about this library and the scanifc library.
    --sync-to-pps=<bool>   \
                             Force timestamps in rxp streams to be synced to a PPS signal (or \
                             not) [default: true].
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_stream: String,
    cmd_info: bool,
    flag_sync_to_pps: bool,
    flag_version: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                         .and_then(|d| d.decode())
                         .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("rivlib-rs version: {}", env!("CARGO_PKG_VERSION"));
        let (major, minor, build) = scanifc::library_version().unwrap();
        println!("scanifc library version: {}.{}.{}", major, minor, build);
        let (build_version, build_tag) = scanifc::library_info().unwrap();
        println!("scanifc build version: {}", build_version);
        println!("scanifc build tag: {}", build_tag);
        exit(0);
    }

    if args.cmd_info {
        let stream = scanifc::Stream::open(args.arg_stream, args.flag_sync_to_pps).unwrap();
        let points: Vec<_> = stream.into_iter().collect();
        println!("number of points: {}", points.len());
    }
}
