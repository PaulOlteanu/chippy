use std::io::Write;

use itertools::Itertools;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let f = &args[1];
    let mut as_text = std::fs::read_to_string(f).unwrap();
    as_text.retain(|c| !c.is_whitespace());
    let as_bin: Vec<u8> = as_text
        .chars()
        .tuples()
        .map(|(a, b)| (a.to_digit(16).unwrap() as u8) << 4 | (b.to_digit(16).unwrap() as u8))
        .collect();
    let mut file = std::fs::File::create("roms/rom.c8").unwrap();
    file.write_all(&as_bin).unwrap();
}
