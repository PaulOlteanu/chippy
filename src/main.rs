mod system;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./chippy path/to/rom");
        return;
    }

    let rom_path = &args[1];
    let rom_read = std::fs::read(rom_path).expect("Couldn't open rom");
    let mut rom = [0; 3584];

    let copy_length = std::cmp::min(rom_read.len(), 3584);
    rom[..copy_length].copy_from_slice(&rom_read[..copy_length]);

    let chip8 = system::System::new(&rom);

    chip8.run();
}
