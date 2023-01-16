mod emulation;
use emulation::{ Bus16Bit, Ben6502};

fn main() {
  println!("Hello, world!");
  let mem_bus = Bus16Bit::new();
  let cpu: Ben6502 = Ben6502::new(mem_bus);
}
