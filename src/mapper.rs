pub trait Mapper {
  fn in_cpu_address_bounds(&self, addr:u16) -> bool;
  fn in_ppu_address_bounds(&self, addr:u16) -> bool;

  fn mapReadAddressFromCPU(&self, addr: u16) -> Result<u16, String>;
  fn mapWriteAddressFromCPU(&self, addr: u16) -> Result<u16, String>;
  fn mapReadAddressFromPPU(&self, addr: u16) -> Result<u16, String>;
  fn mapWriteAddressFromPPU(&self, addr: u16) -> Result<u16, String>;
}

pub struct Mapper000 {
  cpu_address_bounds: (u16, u16),
  ppu_address_bounds: (u16, u16),
  num_PRG_banks: u8,
  num_CHR_banks: u8,
}

impl Mapper000 {
  pub fn new(num_PRG_banks: u8, num_CHR_banks: u8) -> Mapper000 {
    return Mapper000 {
      cpu_address_bounds: (0x8000, 0xFFFF),
      ppu_address_bounds: (0x0000, 0x1FFF),
      num_PRG_banks,
      num_CHR_banks
    }
  }
}

impl Mapper for Mapper000 {

  fn in_cpu_address_bounds(&self, addr:u16) -> bool {
    return addr >= self.cpu_address_bounds.0 && addr <= self.cpu_address_bounds.1;
  }

  fn in_ppu_address_bounds(&self, addr:u16) -> bool {
    return addr >= self.ppu_address_bounds.0 && addr <= self.ppu_address_bounds.1;
  }

  fn mapReadAddressFromCPU(&self, addr: u16) -> Result<u16, String> {
    if self.in_cpu_address_bounds(addr) {
      // if PRGROM is 16KB (1 memory bank)
      //     CPU Address Bus          PRG ROM
      //     0x8000 -> 0xBFFF: Map    0x0000 -> 0x3FFF
      //     0xC000 -> 0xFFFF: Mirror 0x0000 -> 0x3FFF
      // if PRGROM is 32KB (2 memory banks)
      //     CPU Address Bus          PRG ROM
      //     0x8000 -> 0xFFFF: Map    0x0000 -> 0x7FFF
      let mapped_addr = if self.num_PRG_banks > 1 { addr & 0x7FFF } else { addr & 0x3FFF};
      return Ok(mapped_addr);
    } else {
      return Err(String::from("Mapper received a CPU read address outside of CPU bounds!"));
    }
  }

  fn mapWriteAddressFromCPU(&self, addr: u16) -> Result<u16, String> {
    if self.in_cpu_address_bounds(addr) {
      let mapped_addr = if self.num_PRG_banks > 1 { addr & 0x7FFF } else { addr & 0x3FFF};
      return Ok(mapped_addr);
    } else {
      return Err(String::from("Mapper received a CPU write address outside of CPU bounds!"));
    }
  }

  fn mapReadAddressFromPPU(&self, addr: u16) -> Result<u16, String> {
    if self.in_ppu_address_bounds(addr) {
      return Ok(addr);
    } else {
      return Err(String::from("Mapper received a PPU read address outside of PPU bounds!"));
    }
  }

  fn mapWriteAddressFromPPU(&self, addr: u16) -> Result<u16, String> {
    if self.in_ppu_address_bounds(addr) {
      return Ok(addr);
    } else {
      return Err(String::from("Mapper received a PPU write address outside of PPU bounds!"));
    }
  }
}