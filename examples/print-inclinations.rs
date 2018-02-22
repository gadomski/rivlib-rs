extern crate rivlib;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(2, args.len());
    let reader = rivlib::Reader::from_path(&args[1]);
    println!("Time,Roll,Pitch");
    for result in reader.inclinations().unwrap() {
        let inclination = result.unwrap();
        println!(
            "{},{:.3},{:.3}",
            inclination.time, inclination.roll, inclination.pitch
        );
    }
}
