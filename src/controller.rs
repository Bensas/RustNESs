/*

Input is processed in the following way:
- When the game needs controller input, it writes to one of the addresses (0x4016 ot 0x4017)
- The controller then gathers all the pressed/unpressed buttons and stores them into a byte
- The CPU can now read from the register 8 times to get the pressd/unpressed value of each button.

- In this implementation, the emulator_input array is updated by the emulator UI program,
and whenever the game writes to location 0x4016 or 0x4017, the data is moved to the data variable that
will be used to return adecuate read values.

*/

use crate::device::Device;

pub struct Controller {
  data: [u8; 2],
  pub emulator_input: [u8; 2]
}

impl Controller {
  pub fn new() -> Self {
    return Controller {
      data: [0; 2],
      emulator_input: [0; 2],
    }
  }
}

impl Device for Controller {
  fn in_memory_bounds(&self, addr: u16)-> bool {
    return addr == 0x4016 || addr == 0x4017;
  }

  fn write(&mut self, addr: u16, data: u8) -> Result<(), String> {
    if addr == 0x4016 {
      self.data[0] = self.emulator_input[0];
      return Ok(());
    } else if addr == 0x4017 {
      self.data[1] = self.emulator_input[1];
      return Ok(());
    }
    return Err(String::from("Read from controller but not from addresses 0x4016 or 0x4017"));
  }

  fn read(&mut self, addr: u16) -> Result<u8, String> {
    if addr == 0x4016 {
      let return_value = (self.data[0] & 0x80 > 0) as u8;
      self.data[0] <<= 1;
      return Ok(return_value);
    } else if addr == 0x4017 {
      let return_value = (self.data[1] & 0x80 > 0) as u8;
      self.data[1] <<= 1;
      return Ok(return_value);
    }
    return Err(String::from("Read from controller but not from addresses 0x4016 or 0x4017"));
  }
}