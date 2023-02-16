pub trait Device {
  fn in_memory_bounds(&self, addr: u16)-> bool;
  fn write(&mut self, addr: u16, data: u8) -> Result<(), String>;
  fn read(&mut self, addr: u16) -> Result<u8, String>;
}