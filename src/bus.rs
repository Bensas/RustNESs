use std::{sync::{Arc, Mutex}, cell::RefCell, rc::Rc};

use crate::{device::Device, ben2C02::Ben2C02, hex_utils, cartridge::create_cartridge_from_ines_file, ram::Ram2K, controller::Controller};

pub struct Bus16Bit {
  pub devices: Vec<Rc<RefCell<dyn Device>>>,
  pub PPU: Rc<RefCell<Ben2C02>>,
  pub controller: Rc<RefCell<Controller>>,

  // Direct Memory Access variables
  pub dma_transfer_active: bool,
  pub waiting_for_cycle_alignment: bool,
  pub dma_page: u8,
  pub dma_curr_data: u8,
  pub dma_curr_addr: u16,
}

const DMA_ADDR: u16 = 0x4014;

// Assumed to be a 16-bit bus
impl Bus16Bit {

  pub fn new(rom_file_path: &str) -> Bus16Bit {
    let ram = Rc::new(RefCell::new(Ram2K::new((0x0000, 0x1FFF))));
    let apu_mock = Rc::new(RefCell::new(Ram2K::new((0x4000, 0x4015))));
    let cartridge = Rc::new(RefCell::new(create_cartridge_from_ines_file(rom_file_path).unwrap()));
    let PPU = Rc::new(RefCell::new(Ben2C02::new(cartridge.clone())));
    let controller = Rc::new(RefCell::new(Controller::new()));

    let mut devices: Vec<Rc<RefCell<dyn Device>>> = vec![];
    devices.push(ram);
    devices.push(apu_mock);
    devices.push(PPU.clone());
    devices.push(controller.clone());
    devices.push(cartridge);
    return Bus16Bit {
      devices,
      PPU,
      controller,
      dma_transfer_active: false,
      waiting_for_cycle_alignment: true,
      dma_page: 0x0,
      dma_curr_data: 0x0,
      dma_curr_addr: 0x0,
    }
  }

  pub fn read(&mut self, addr: u16, readOnly: bool) -> Result<u8, String> {
    for device in self.devices.iter() {
      if device.borrow().in_memory_bounds(addr) {
        return device.borrow_mut().read(addr);
      }
    }
    return Ok(0);
    return Err(String::from(format!("Error reading from memory bus (No device found in given address: 0x{:x}).", addr)));
  }

  pub fn read_word_little_endian(&mut self, addr: u16, readOnly: bool) -> Result<u16, String> {
    let low = self.read(addr, false);
    let high = self.read(addr + 1, false);

    if (low.is_ok() && high.is_ok()) {
      let result = ((high.unwrap() as u16) << 8) + (low.unwrap() as u16);
      return Ok(result);
    } else {
      return Err(String::from(format!("Error reading word from memory bus (No device found in given address: 0x{:x}).", addr)));
    }
  }

  pub fn write(&mut self, addr: u16, content: u8) -> Result<(), String>{
    if (addr == DMA_ADDR) {
      self.dma_page = content;
      self.dma_curr_addr = (self.dma_page as u16) << 8;
      self.dma_transfer_active = true;
      self.waiting_for_cycle_alignment = true;
      self.dma_curr_data = 0;
      return Ok(());
    }
    for device in self.devices.iter_mut() {
      if device.borrow().in_memory_bounds(addr) {
        return device.borrow_mut().write(addr, content);
      }
    }
    return Err(format!("Error writing to memory bus (No device found in given address: 0x{:X}", addr));
  }

  pub fn get_memory_content_as_string(&mut self, start_addr: u16, end_addr: u16) -> String {
    let mut result = String::new();
    for curr_addr in start_addr..end_addr {
      let memory_content = self.read(curr_addr, false).unwrap();
      result.push_str(&hex_utils::decimal_byte_to_hex_str(memory_content));
      result.push_str(" ");
    }
    return result;
  }

  pub fn get_memory_content_as_vec(&mut self, start_addr: u16, end_addr: u16) -> Vec<u8> {
    let mut result = vec![];
    for curr_addr in start_addr..end_addr {
      let memory_content = self.read(curr_addr, false).unwrap();
      result.push(memory_content);
    }
    return result;
  }

  // pub fn get_PPU(&mut self) -> Rc<RefCell<Ben2C02>> {
  //   return self.PPU;
  // }
}


#[cfg(test)]
mod bus_tests {
  use crate::Bus16Bit;

  // #[test]
  // fn test_get_memory_content_as_string() {
  //   let bus = Bus16Bit::new("hey_mona.nes");
  //   println!("{}", bus.get_memory_content_as_string(0, 100));
  // }

}