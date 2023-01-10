/*


File: device.rs


*/
pub trait Device {
  fn in_memory_bounds(&self, addr: u16)-> bool;
  fn write(&mut self, addr: u16, content: u8) -> Result<(), String>;
  fn read(&self, addr: u16) -> Result<u8, String>;
}


/*


File: ram64k.rs


*/
pub struct Ram64K {
  memory: [u8; 64 * 1024],
  memory_bounds: (u16, u16)
}

impl Device for Ram64K {

  fn in_memory_bounds(&self, addr: u16)-> bool {
    if addr >= self.memory_bounds.0 && addr <= self.memory_bounds.1 {
      return true;
    } else {
      return false;
    }
  }

  fn write(&mut self, addr: u16, content: u8) -> Result<(), String> {
    if self.in_memory_bounds(addr) {
      self.memory[addr as usize] = content;
      return Ok(());
    } else {
      return Err(String::from("Tried to write outside RAM bounds!"));
    }
  }

  fn read(&self, addr: u16) -> Result<u8, String> {
    if self.in_memory_bounds(addr) {
      return Ok(self.memory[addr as usize]);
    } else {
      return Err(String::from("Tried to read outside RAM bounds!"));
    }
  }
}

/*


File: ben6502.rs


*/
struct Registers {
  a: u8,
  x: u8,
  y: u8,
  sp: u8,
  pc: u8
}

struct Status {
  flags: u8
}

impl Status {

  fn get_carry(&self) -> u8 {
    return self.flags & 1;
  }

  fn set_carry(&mut self, value: u8) {
    if value == 0 {
      let inverted_value = !value;
      self.flags &= inverted_value;
    } else {
      self.flags |= value;
    }
  }
}

#[test]
fn test_status_flags() {
  let status = Status{flags: 0};
  let carry = status.get_carry();
  assert_eq!(carry, 0);
}

pub struct Ben6502 {
  bus: Bus,

  status_flags: Status,

  registers: Registers,
  

}

impl Ben6502 {

  

}


/*


File: bus.rs


*/
struct Bus {
  devices: Vec<Box<dyn Device>>
}

// Assumed to be a 16-bit bus
impl Bus {

  fn new() -> Bus {
    let devices: Vec<Box<dyn Device>> = vec![Box::new(Ram64K{memory: [0; 64*1024], memory_bounds: (0x0000, 0xFFFF)})];
    return Bus {
      devices
    }
  }

  fn read(&mut self, addr: u16, readOnly: bool) -> Result<u8, String> {
    for device in self.devices.iter() {
      if device.in_memory_bounds(addr) {
        return device.read(addr);
      }
    }
    return Err(String::from("Error reading from memory bus (No device found in given address)."))

  }

	fn write(&mut self, addr: u16, content: u8) -> Result<(), String>{
    for device in self.devices.iter_mut() {
      if device.in_memory_bounds(addr) {
        return device.write(addr, content);
      }
    }
    return Err(String::from("Error writing to memory bus (No device found in given address)."))

  }
}



fn main() {
  println!("Hello, world!");
  let mem_bus = Bus::new();
}
