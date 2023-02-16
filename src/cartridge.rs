use std::{fs, rc::Rc, sync::{Mutex, Arc}};

use crate::{mapper::{Mapper, Mapper000}, device::Device};

#[derive(Debug, Clone, Copy)]
pub enum MirroringMode {
  Vertical,
  Horizontal,
  OnscreenLo,
  OnscreenHi
}

fn verify_nes_header (file_contents: &Vec<u8>) -> bool{
  return file_contents[0] == ('N' as u8) &&
        file_contents[1] == ('E' as u8) &&
        file_contents[2] == ('S' as u8);
}

fn get_mapper1_from_flags6(flags6: u8) -> u8 {
  return (flags6 >> 4) & 0b1111;
}

fn get_mapper2_from_flags7(flags7: u8) -> u8 {
  return (flags7 >> 4) & 0b1111;
}

fn get_tv_system_1_from_flags9(flags9: u8) -> u8 {
  return flags9 & 0b1;
}

fn get_tv_system_2_from_flags10(flags10: u8) -> u8 {
  return flags10 & 0b11;
}

fn create_mapper_from_number(mapper_num: u8, num_prg_banks: u8, num_chr_banks: u8) -> Result<Box<dyn Mapper>, String> {
  match mapper_num {
    0 => {
      let result = Mapper000::new(num_prg_banks, num_chr_banks);
      return Ok(Box::new(result));
    },
    _ => Err(String::from(format!("Tried to create a mapper using mapper number {}", mapper_num)))
  }
}

// Reference: https://www.nesdev.org/wiki/INES
pub fn create_cartridge_from_ines_file(file_path: &str) -> Result<Cartridge, String> {
  let file_contents = fs::read(file_path).unwrap();
  if !verify_nes_header(&file_contents){
    return Err(String::from("Error while loading ROM file: invalid NES header."));
  }

  let nes_name = &file_contents[0..4];
  let prg_chunks = file_contents[4];
  let chr_chunks = file_contents[5];
  let flags6 = file_contents[6];
  let flags7 = file_contents[7];
  let prg_ram_size = file_contents[8];
  let flags9 = file_contents[9];
  let flags10 = file_contents[10];

  let header = RomHeader{
    name: nes_name.try_into().unwrap(),
    prg_chunks,
    chr_chunks,
    mapper1: get_mapper1_from_flags6(flags6),
    mapper2: get_mapper2_from_flags7(flags7),
    prg_ram_size,
    tv_system_1: get_tv_system_1_from_flags9(flags9),
    tv_system_2: get_tv_system_2_from_flags10(flags10),
  };

  let mirroring_mode = if (flags6 & 0x01) != 0 { MirroringMode::Vertical } else { MirroringMode::Horizontal };

  let mapper = create_mapper_from_number((header.mapper2 << 4) & header.mapper1, prg_chunks, chr_chunks).unwrap();

  let mut cartridge = Cartridge::new(header, mapper, mirroring_mode);

  let prg_data_start_index: usize= if ((flags6 & 0x04 != 0) as bool) { 16 + 512 } else { 16 }; 
  

  // "Discover" File Format
  let n_file_type = 1;

  match n_file_type {
    0 => {

    },
    1 => {

      let prg_data_end_index= prg_data_start_index + (prg_chunks as usize) * 16384;
      for i in prg_data_start_index..prg_data_end_index {
        cartridge.PRG_data.push(file_contents[i as usize]);
      }
      
      let chr_data_start_index= prg_data_end_index;
      let chr_data_end_index= chr_data_start_index + (chr_chunks as usize) * 8192;
      
      for i in chr_data_start_index..chr_data_end_index {
        cartridge.CHR_data.push(file_contents[i as usize]);
      }
    },
    2 => {

    },
    _ => {
      return Err(String::from(format!("File type {} hasn't been implemented", n_file_type)));
    }
  }
  return Ok(cartridge);

}



struct RomHeader {
  name: [u8; 4],
  prg_chunks: u8,
  chr_chunks: u8,
  mapper1: u8,
  mapper2: u8,
  prg_ram_size: u8,
  tv_system_1: u8,
  tv_system_2: u8
  // unused: char[]
}

pub struct Cartridge {
  cpu_memory_bounds: (u16, u16),
  ppu_memory_bounds: (u16, u16),
  rom_header: RomHeader,
  PRG_data: Vec<u8>,
  CHR_data: Vec<u8>,
  mapper: Box<dyn Mapper>,
  pub mirroring_mode: MirroringMode
}

impl Cartridge {
  fn new(rom_header: RomHeader, mapper: Box<dyn Mapper>, mirroring_mode: MirroringMode) -> Cartridge {
    return Cartridge {
      cpu_memory_bounds: (0x8000, 0xFFFF),
      ppu_memory_bounds: (0x0000, 0x1FFF),
      rom_header,
      PRG_data: vec![],
      CHR_data: vec![],
      mapper,
      mirroring_mode
    };
  }

  fn in_ppu_memory_bounds(&self, addr:u16) -> bool {
    return addr >= self.ppu_memory_bounds.0 && addr <= self.ppu_memory_bounds.1;
  }

  fn in_cpu_memory_bounds(&self, addr:u16) -> bool {
    return addr >= self.cpu_memory_bounds.0 && addr <= self.cpu_memory_bounds.1;
  }

}

impl Device for Cartridge {

  fn in_memory_bounds(&self, addr: u16)-> bool {
    if self.in_cpu_memory_bounds(addr) {
      return true;
    } else {
      return false;
    }
  }

  fn write(&mut self, addr: u16, content: u8) -> Result<(), String> {
    if self.in_cpu_memory_bounds(addr) {
      // Write operation from CPU
      let mapped_addr_res = self.mapper.mapWriteAddressFromCPU(addr);
      match mapped_addr_res {
        Ok(mapped_addr) => {
          self.PRG_data[mapped_addr as usize] = content;
          return Ok(());
        },
        Err(message) => {
          return Err(message);
        }
      }
    } else if self.in_ppu_memory_bounds(addr) {
      // Write operation from PPU
      let mapped_addr_res = self.mapper.mapWriteAddressFromPPU(addr);
      match mapped_addr_res {
        Ok(mapped_addr) => {
          while (self.CHR_data.len() <= mapped_addr as usize) {
            self.CHR_data.push(0);
          }
          self.CHR_data[mapped_addr as usize] = content;
          return Ok(());
        },
        Err(message) => {
          return Err(message);
        }
      }        
    } else {
      return Err(format!("Tried to write outside Cartridge bounds! Address: 0x{:X}", addr));
    }
  }

  fn read(&mut self, addr: u16) -> Result<u8, String> {
    if self.in_cpu_memory_bounds(addr) {
      // Read operation from CPU
      let mapped_addr_res = self.mapper.mapReadAddressFromCPU(addr);
      match mapped_addr_res {
        Ok(mapped_addr) => {
          let data = self.PRG_data.get(mapped_addr as usize).unwrap();
          return Ok(*data);
        },
        Err(message) => {
          return Err(message);
        }
      }
    } else if self.in_ppu_memory_bounds(addr) {
      // Read operation from PPU
      let mapped_addr_res = self.mapper.mapReadAddressFromPPU(addr);
      match mapped_addr_res {
        Ok(mapped_addr) => {
          // println!("{}", mapped_addr);
          let data = self.CHR_data.get(mapped_addr as usize).unwrap_or(&0);
          return Ok(*data);
        },
        Err(message) => {
          return Err(message);
        }
      }
    } else {
      return Err(format!("Tried to read outside Cartridge bounds! Address: 0x{:X}", addr));
    }
  }
}