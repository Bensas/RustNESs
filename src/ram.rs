use crate::device::Device;

const RAM_SIZE: u16 = 2 * 1024;
pub struct Ram2K {
  pub memory: [u8; RAM_SIZE as usize],
  pub memory_bounds: (u16, u16)
}

impl Ram2K {
  pub fn new(memory_bounds: (u16, u16)) -> Ram2K {
    return Ram2K {
      memory: [0; 2* 1024],
      memory_bounds
    }
  }
}

impl Device for Ram2K {

  fn in_memory_bounds(&self, addr: u16)-> bool {
    if addr >= self.memory_bounds.0 && addr <= self.memory_bounds.1 {
      return true;
    } else {
      return false;
    }
  }

  fn write(&mut self, addr: u16, content: u8) -> Result<(), String> {
    if self.in_memory_bounds(addr) {
      self.memory[(addr % RAM_SIZE) as usize] = content;
      return Ok(());
    } else {
      return Err(String::from("Tried to write outside RAM bounds!"));
    }
  }

  fn read(&mut self, addr: u16) -> Result<u8, String> {
    if self.in_memory_bounds(addr) {
      return Ok(self.memory[(addr % RAM_SIZE) as usize]);
    } else {
      return Err(String::from("Tried to read outside RAM bounds!"));
    }
  }
}