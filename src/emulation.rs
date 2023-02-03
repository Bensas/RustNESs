/*


File: device.rs


*/

pub mod Device {
  pub trait Device {
    fn in_memory_bounds(&self, addr: u16)-> bool;
    fn write(&mut self, addr: u16, data: u8) -> Result<(), String>;
    fn read(&mut self, addr: u16) -> Result<u8, String>;
  }
}


/*


File: ram.rs


*/

pub mod Ram {
  use super::Device::Device;

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
}


/*


File: utils.rs


*/

pub mod bitwise_utils {
  pub fn get_bit(source: u8, bit_pos: u8) -> u8{ // bit_pos counted from least significant to most significant
    return (source & (1 << bit_pos) != 0) as u8;
  }

  pub fn set_bit(target: &mut u8, bit_pos: u8, new_value: u8) {
    match new_value {
      0 => *target &= !(1 << bit_pos),
      1 => *target |= (1 << bit_pos),
      _ => panic!("Tried to set_bit with a value other than 0 or 1")
    }
  }
}

pub mod hex_utils {
  pub fn decimal_word_to_hex_str(word: u16) -> String {
    let mut result = String::new();

    let mut factor = 16 as u32;
    let mut curr_val = word as u32;
    while (curr_val != 0) {
      let digit = (curr_val % factor) / (factor/16);
      result.push(decimal_value_to_hex_char(digit as u8));
      curr_val -= curr_val % factor;
      factor *= 16;
    }
    
    return result.chars().rev().collect();
  }

  pub fn decimal_byte_to_hex_str(decimal: u8) -> String {
    let mut result = String::new();
    let least_sig_digit_value = decimal % 16;
    let most_sig_digit_value = (decimal - least_sig_digit_value) / 16;
    result.push(decimal_value_to_hex_char(most_sig_digit_value));
    result.push(decimal_value_to_hex_char(least_sig_digit_value));
    return result;
  }

  fn decimal_value_to_hex_char(val: u8) -> char {
    match val {
      10 => return 'A',
      11 => return 'B',
      12 => return 'C',
      13 => return 'D',
      14 => return 'E',
      15 => return 'F',
      _ => return char::from_digit(val.into(), 10).unwrap()
    }
  }
}

pub mod graphics {
  #[derive(Copy, Debug)]
  pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8
  }

  impl Color {
    pub fn new(red: u8, green: u8, blue: u8) -> Color {
      return Color { red, green, blue };
    }

    pub fn to_iced_color(&self) -> iced::Color {
      return iced::Color::new((self.red as f32) / 255.0, (self.green as f32) / 255.0, (self.blue as f32) / 255.0, 1.0);
    }
  }

  impl Clone for Color {
    fn clone(&self) -> Self {
        Self { red: self.red.clone(), green: self.green.clone(), blue: self.blue.clone() }
    }
  }
}


/*


File: ben6502.rs


*/
pub mod Ben6502 {
  use super::{bitwise_utils, Bus16Bit::Bus16Bit};

  pub struct Registers {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16
  }
  
  pub struct Status {
    flags: u8
  }
  
  impl Status {
  
    fn new() -> Status {
      return Status{
        flags: 0b00100000 // One of the bits is unused and is always set to 1 
      }
    }
  
    fn reset(&mut self) {
      self.flags = 0b00100000;
    }
  
    pub fn get_carry(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 0);
    }
  
    fn set_carry(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 0, value);
    }
  
    pub fn get_zero(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 1);
    }
  
    fn set_zero(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 1, value);
    }
  
    pub fn get_irq_disable(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 2);
    }
  
    fn set_irq_disable(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 2, value);
    }
  
    pub fn get_decimal_mode(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 3);
    }
  
    fn set_decimal_mode(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 3, value);
    }
  
    pub fn get_brk_command(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 4);
    }
  
    fn set_brk_command(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 4, value);
    }
  
    pub fn get_unused_bit(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 5);
    }
  
    fn set_unused_bit(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 5, value);
    }
  
    pub fn get_overflow(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 6);
    }
  
    fn set_overflow(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 6, value);
    }
  
    pub fn get_negative(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 7);
    }
  
    fn set_negative(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 7, value);
    }
  }
  
  #[cfg(test)]
  mod status_tests {
      use super::Status;
  
    #[test]
    fn test_create_status() {
      let status = Status{ flags: 0 };
      assert_eq!(status.flags, 0);
    }
  
    #[test]
    fn test_get_carry() {
      let status = Status{ flags: 0 };
      assert_eq!(status.get_carry(), 0);
    }
  
    #[test]
    fn test_set_carry() {
      let mut status = Status{ flags: 0 };
  
      status.set_carry(1);
      assert_eq!(status.get_carry(), 1);
  
      status.set_carry(0);
      assert_eq!(status.get_carry(), 0);
    }
  
  }
  
  
  #[derive(Debug)]
  enum AddressingMode {
    ACC, // Accum
    IMM, // Immediate
    ABS, // Absolute
    ZP0, // ZeroPage
    ZPX, // ZeroPageX
    ZPY, // ZeroPageY
    ABX, // AbsoluteX
    ABY, // AbsoluteY
    IMP, // Implied
    REL, // Relative
    INX, // IndirectX
    INY, // IndirectY
    IND, // Indirect
  }
  
  #[derive(Debug)]
  enum Instruction {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    XXX // Illegal instruction
  }
  
  struct InstructionData {
    instruction: Instruction, 
    addressing_mode: AddressingMode,
    cycles: u8,
  }
  
  // Original table was taken from https://github.com/OneLoneCoder/olcNES/blob/master/Part%232%20-%20CPU/olc6502.cpp
  // Author: David Barr, aka javidx9 or OneLoneCoder
  const INSTRUCTION_TABLE: [InstructionData; 256] = 
  [
    InstructionData{instruction: Instruction::BRK, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::PHP, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ACC, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
    InstructionData{instruction: Instruction::BPL, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ZPX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::CLC, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ABX, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },
    InstructionData{instruction: Instruction::JSR, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::BIT, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::ROL, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::PLP, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::ROL, addressing_mode: AddressingMode::ACC, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::BIT, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::ROL, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
    InstructionData{instruction: Instruction::BMI, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::ROL, addressing_mode: AddressingMode::ZPX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::SEC, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::AND, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::ROL, addressing_mode: AddressingMode::ABX, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },
    InstructionData{instruction: Instruction::RTI, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::LSR, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::PHA, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::LSR, addressing_mode: AddressingMode::ACC, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::JMP, addressing_mode: AddressingMode::ABS, cycles: 3 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::LSR, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
    InstructionData{instruction: Instruction::BVC, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::LSR, addressing_mode: AddressingMode::ZPX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::CLI, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::EOR, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::LSR, addressing_mode: AddressingMode::ABX, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },
    InstructionData{instruction: Instruction::RTS, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::ROR, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::PLA, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::ROR, addressing_mode: AddressingMode::ACC, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::JMP, addressing_mode: AddressingMode::IND, cycles: 5 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::ROR, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
    InstructionData{instruction: Instruction::BVS, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::ROR, addressing_mode: AddressingMode::ZPX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::SEI, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ADC, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::ROR, addressing_mode: AddressingMode::ABX, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },
    InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::STY, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::STX, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::DEY, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::TXA, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::STY, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::STX, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },
    InstructionData{instruction: Instruction::BCC, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::INY, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::STY, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::STX, addressing_mode: AddressingMode::ZPY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::TYA, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::ABY, cycles: 5 },InstructionData{instruction: Instruction::TXS, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::STA, addressing_mode: AddressingMode::ABX, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },
    InstructionData{instruction: Instruction::LDY, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::LDX, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::LDY, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::LDX, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::TAY, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::TAX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::LDY, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::LDX, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },
    InstructionData{instruction: Instruction::BCS, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::LDY, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::LDX, addressing_mode: AddressingMode::ZPY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::CLV, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::TSX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::LDY, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::LDA, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::LDX, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },
    InstructionData{instruction: Instruction::CPY, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::CPY, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::DEC, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::INY, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::DEX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::CPY, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::DEC, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
    InstructionData{instruction: Instruction::BNE, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::DEC, addressing_mode: AddressingMode::ZPX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::CLD, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::NOP, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::CMP, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::DEC, addressing_mode: AddressingMode::ABX, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },
    InstructionData{instruction: Instruction::CPX, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::CPX, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::INC, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::INX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::NOP, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::CPX, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::INC, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
    InstructionData{instruction: Instruction::BEQ, addressing_mode: AddressingMode::REL, cycles: 2 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::INY, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::ZPX, cycles: 4 },InstructionData{instruction: Instruction::INC, addressing_mode: AddressingMode::ZPX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },InstructionData{instruction: Instruction::SED, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::ABY, cycles: 4 },InstructionData{instruction: Instruction::NOP, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::SBC, addressing_mode: AddressingMode::ABX, cycles: 4 },InstructionData{instruction: Instruction::INC, addressing_mode: AddressingMode::ABX, cycles: 7 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 7 },
  ];
  
  pub const STACK_START_ADDR: u16 = 0x100;
  
  pub const SP_RESET_ADDR: u8 = 0xFD;
  
  pub const PROGRAM_START_POINTER_ADDR: u16 = 0xFFFC;
  
  const INTERRUPT_START_POINTER_ADDR: u16 = 0xFFFE;
  
  const NMI_START_POINTER_ADDR: u16 = 0xFFFA;
  
  pub struct Ben6502 {
    pub bus: Bus16Bit,
  
    pub status: Status,
    pub registers: Registers,
  
  
    pub current_instruction_remaining_cycles: u8,
    needs_additional_cycle: bool,
    // fetched_data: u8,
    absolute_mem_address: u16,
  
    /*
    Regarding relative addresses:
     - They will be represented using 2's Complement
     - In a branch instruction, they are the address relative to the value 
     of the PC AFTER processing the opcode and operand.
    */
    relative_mem_address: i8,
  
  }
  
  impl Ben6502 {
    pub fn new(mem_bus: Bus16Bit) -> Ben6502 {
      let mut result = Ben6502 {
        bus: mem_bus,
        status: Status::new(),
        registers: Registers { a: 0, x: 0, y: 0, sp: 0, pc: 0 },
        current_instruction_remaining_cycles: 0,
        needs_additional_cycle: false,
        absolute_mem_address: 0,
        relative_mem_address: 0
      };
      result.reset();
      return result;
    }
  
    fn set_addressing_mode(&mut self, mode: &AddressingMode) {
      match mode {
        AddressingMode::ACC => {
          // The data will be taken directly from the accumulator register, so we don't need an address to fetch the data
        },
        AddressingMode::IMM => {
          self.absolute_mem_address = self.registers.pc;
          self.registers.pc += 1;
        },
        AddressingMode::ABS => {
          self.absolute_mem_address = self.bus.read_word_little_endian(self.registers.pc, false).unwrap();
          self.registers.pc += 2;
        },
        AddressingMode::ZP0 => {
          let addr_low = self.bus.read(self.registers.pc, false).unwrap();
          self.registers.pc += 1;
          let addr_high = 0;
          self.absolute_mem_address = addr_low as u16;
        },
        AddressingMode::ZPX => {
          let instruction_addr = self.bus.read(self.registers.pc, false).unwrap();
          self.registers.pc += 1;
          self.absolute_mem_address = (instruction_addr as u16 + self.registers.x as u16) & 0x00FF;
        },
        AddressingMode::ZPY => {
          let instruction_addr = self.bus.read(self.registers.pc, false).unwrap();
          self.registers.pc += 1;
          self.absolute_mem_address = (instruction_addr as u16 + self.registers.y as u16) & 0x00FF;
        },
        AddressingMode::ABX => {
          let mem_addr = self.bus.read_word_little_endian(self.registers.pc, false).unwrap();
          self.registers.pc += 2;
  
          self.absolute_mem_address = mem_addr.wrapping_add(self.registers.x as u16);
  
          if (self.absolute_mem_address > (mem_addr & 0xFF00)) { // We crossed a page boundary after adding X to the address
            self.current_instruction_remaining_cycles += 1;
          }
        },
        AddressingMode::ABY => {
          let mem_addr = self.bus.read_word_little_endian(self.registers.pc, false).unwrap();
          self.registers.pc += 2;
          self.absolute_mem_address = mem_addr.wrapping_add(self.registers.y as u16);
  
          if (self.absolute_mem_address > (mem_addr & 0xFF00)) { // We crossed a page boundary after adding X to the address
            self.current_instruction_remaining_cycles += 1;
          }
        },
        AddressingMode::IMP => {
          // Implied addressing means that no address is required to execute the instruction
        },
        AddressingMode::REL => {
          self.relative_mem_address = self.bus.read(self.registers.pc, false).unwrap() as i8;
          self.registers.pc += 1;
        },
        AddressingMode::INX => {
          let instruction_addr = self.bus.read(self.registers.pc, false).unwrap();
          self.registers.pc += 1;
  
          let pointer_to_addr = (instruction_addr as u16 + self.registers.x as u16) & 0x00FF;
  
          self.absolute_mem_address = self.bus.read_word_little_endian(pointer_to_addr, false).unwrap();        
        }
        AddressingMode::INY => {
          let instruction_addr = self.bus.read(self.registers.pc, false).unwrap();
          self.registers.pc += 1;
  
          let pointer_to_addr = (instruction_addr as u16 + self.registers.y as u16) & 0x00FF;
  
          self.absolute_mem_address = self.bus.read_word_little_endian(pointer_to_addr, false).unwrap();        
        },
        AddressingMode::IND => {
          let abs_address_of_low_byte = self.bus.read_word_little_endian(self.registers.pc, false).unwrap();
          self.registers.pc += 2;
          
          let low_byte = self.bus.read(abs_address_of_low_byte, false).unwrap();
          let high_byte: u8;
  
          if ((abs_address_of_low_byte & 0xFF) == 0x00FF) { // We must do this weird thing to simulate a hardware bug in the CPU with page boundaries. https://www.nesdev.org/6502bugs.txt
            high_byte = self.bus.read(abs_address_of_low_byte & 0xFF00, false).unwrap();
          } else {
            high_byte = self.bus.read(abs_address_of_low_byte + 1, false).unwrap();
          }
  
          self.absolute_mem_address = ((high_byte as u16) << 8) + (low_byte as u16);
        },
        _ => return
        
      }
    }
  
  
    fn execute_instruction(&mut self, instruction: &Instruction, addr_mode: &AddressingMode) {
  
      match instruction {
          Instruction::ADC => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = self.registers.a as u16 + operand as u16 + self.status.get_carry() as u16;
            self.status.set_carry( (result > 0x00FF) as u8);
            self.status.set_zero( (result & 0xFF == 0) as u8);
            self.status.set_negative( (result & 0b10000000 != 0) as u8);
            // A beautiful explanation for the following line can be found at https://youtu.be/8XmxKPJDGU0?t=2540
            self.status.set_overflow((((!(self.registers.a as u16 ^ operand as u16) & (self.registers.a as u16 ^ result as u16)) & 0b10000000) != 0) as u8); 
            self.registers.a = (result & 0x00FF) as u8;
            // todo!("Might require an additional clock cycle :S");
          },
          Instruction::AND => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            self.registers.a = self.registers.a & operand;
            self.status.set_zero((self.registers.a == 0) as u8);
            self.status.set_negative((self.registers.a == 0b10000000) as u8);
          },
          Instruction::ASL => {
            let operand;
  
            if matches!(addr_mode, AddressingMode::IMP) {
              operand = self.registers.a;
            } else {
              operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            }
            let result: u16 = (operand as u16) << 1;
            self.status.set_carry((result & 0xFF00 != 0) as u8);
            self.status.set_zero((result & 0xFF == 0) as u8);
            self.status.set_negative(((result & 0b10000000) != 0) as u8);
            if matches!(addr_mode, AddressingMode::IMP) {
              self.registers.a = (result & 0xFF) as u8;
            } else {
              self.bus.write(self.absolute_mem_address, (result & 0xFF) as u8).unwrap();
            }
          },
          Instruction::BCC => {
            if (self.status.get_carry() == 0) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
  
            }
          },
          Instruction::BCS => {
            if (self.status.get_carry() == 1) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
  
            }
          },
          Instruction::BEQ => {
            if (self.status.get_zero() == 1) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
  
            }
          },
          Instruction::BIT => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = self.registers.a & operand;
  
            self.status.set_zero(( (result & 0xFF) == 0) as u8);
            self.status.set_negative(((operand & 0b10000000) != 0) as u8);
            self.status.set_overflow(((operand & 0b01000000) != 0) as u8);
          },
          Instruction::BMI => {
            if (self.status.get_negative() == 1) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
  
            }
          },
          Instruction::BNE => {
            if (self.status.get_zero() == 0) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
  
            }
          },
          Instruction::BPL => {
            if (self.status.get_negative() == 0) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
  
            }
          },
          Instruction::BRK => {
            self.registers.pc += 1;
  
            self.status.set_irq_disable(1);
  
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, ((self.registers.pc >> 8) & 0xFF) as u8).unwrap();
            self.registers.sp -= 1;
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, ( self.registers.pc       & 0xFF) as u8).unwrap();
            self.registers.sp -= 1;
  
            self.status.set_brk_command(1);
  
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, self.status.flags).unwrap();
            self.registers.sp -= 1;
  
            self.status.set_brk_command(0);
  
            self.registers.pc = self.bus.read_word_little_endian(INTERRUPT_START_POINTER_ADDR, false).unwrap();
            
          },
          Instruction::BVC => {
            if (self.status.get_overflow() == 0) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
            }
          },
          Instruction::BVS => {
            if (self.status.get_overflow() == 1) {
              self.current_instruction_remaining_cycles += 1;
              self.absolute_mem_address = (self.registers.pc as i16 + self.relative_mem_address as i16) as u16;
              if ((self.absolute_mem_address & 0xFF00) != (self.registers.pc & 0xFF)){ // If there is a page jump
                self.current_instruction_remaining_cycles += 1;
              }
              self.registers.pc = self.absolute_mem_address;
            }
          },
          Instruction::CLC => {
            self.status.set_carry(0);
          },
          Instruction::CLD => {
            self.status.set_decimal_mode(0);
          },
          Instruction::CLI => {
            self.status.set_irq_disable(0);
          },
          Instruction::CLV => {
            self.status.set_overflow(0);
          },
          Instruction::CMP => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = (self.registers.a as u16).wrapping_sub(operand as u16);
            self.status.set_carry((self.registers.a >= operand) as u8);
            self.status.set_zero(( (result & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((result & 0b10000000 != 0) as u8);
          },
          Instruction::CPX => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = (self.registers.x as u16).wrapping_sub(operand as u16);
            self.status.set_carry((self.registers.x >= operand) as u8);
            self.status.set_zero(( (result & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((result & 0b10000000 != 0) as u8);
          },
          Instruction::CPY => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = (self.registers.y as u16).wrapping_sub(operand as u16);
            self.status.set_carry((self.registers.y >= operand) as u8);
            self.status.set_zero(( (result & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((result & 0b10000000 != 0) as u8);
          },
          Instruction::DEC => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = operand.wrapping_sub(1);
            self.bus.write(self.absolute_mem_address, result).unwrap();
  
            self.status.set_zero(( (result & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((result & 0b10000000 != 0) as u8);
          },
          Instruction::DEX => {
            self.registers.x = self.registers.x.wrapping_sub(1);
  
            self.status.set_zero(( (self.registers.x & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.x & 0b10000000 != 0) as u8);
          },
          Instruction::DEY => {
            self.registers.y = self.registers.y.wrapping_sub(1);
  
            self.status.set_zero(( (self.registers.y & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.y & 0b10000000 != 0) as u8);
          },
          Instruction::EOR => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            self.registers.a ^= operand;
            self.status.set_zero(( (self.registers.a & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.a & 0b10000000 != 0) as u8);
          },
          Instruction::INC => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            let result = operand.wrapping_add(1);
            self.bus.write(self.absolute_mem_address, result).unwrap();
  
            self.status.set_zero(( (result & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((result & 0b10000000 != 0) as u8);
          },
          Instruction::INX => {
            self.registers.x = self.registers.x.wrapping_add(1);
  
            self.status.set_zero(( (self.registers.x & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.x & 0b10000000 != 0) as u8);
          },
          Instruction::INY => {
            self.registers.y = self.registers.y.wrapping_add(1);
  
            self.status.set_zero(( (self.registers.y & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.y & 0b10000000 != 0) as u8);
          },
          Instruction::JMP => {
            self.registers.pc = self.absolute_mem_address;
          },
          Instruction::JSR => {

            // During addressing mode, we increased the pc to the address after the 3 bytes of the instruction, where we want to return to after the subroutine
            // This instruction pushes the address A-1 to the stack, where A is the address to be returned to after the subroutine
            // More info: https://www.masswerk.at/6502/6502_instruction_set.html#stack
            self.registers.pc -= 1;
  
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, (self.registers.pc >> 8) as u8).unwrap();
            self.registers.sp -= 1;
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, (self.registers.pc & 0xFF) as u8).unwrap();
            self.registers.sp -= 1;
  
            self.registers.pc = self.absolute_mem_address;
  
          },
          Instruction::LDA => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            self.registers.a = operand;
  
            self.status.set_zero(( (self.registers.a & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.a & 0b10000000 != 0) as u8);
          },
          Instruction::LDX => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            self.registers.x = operand;
  
            self.status.set_zero(( (self.registers.x & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.x & 0b10000000 != 0) as u8);
          },
          Instruction::LDY => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            self.registers.y = operand;
  
            self.status.set_zero(( (self.registers.y & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.y & 0b10000000 != 0) as u8);
          },
          Instruction::LSR => {
            let operand;
  
            if matches!(addr_mode, AddressingMode::IMP) {
              operand = self.registers.a;
            } else {
              operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            }
            self.status.set_carry((operand & 0x0001 != 0) as u8);
  
            let result: u16 = (operand as u16) >> 1;
  
            self.status.set_zero(( (result & 0xFF) == 0) as u8);
            self.status.set_negative(((result & 0b10000000) != 0) as u8);
            if matches!(addr_mode, AddressingMode::IMP) {
              self.registers.a = (result & 0xFF) as u8;
            } else {
              self.bus.write(self.absolute_mem_address, (result & 0xFF) as u8).unwrap();
            }
          },
          Instruction::NOP => {
            // No Operation
  
            /* Some games use illegal opcodes, which are NOP but have special behaviours.
              These behaviours are nor implemented. The following games are guilty: 
              Beauty and the Beast (E) (1994) uses $80 (a 2-byte NOP).[2]
              Disney's Aladdin (E) (December 1994) uses $07 (SLO). This is Virgin's port of the Game Boy game, itself a port of the Genesis game, not any of the pirate originals.
              Dynowarz: Destruction of Spondylus (April 1990) uses 1-byte NOPs $DA and $FA on the first level when your dino throws his fist.
              F-117A Stealth Fighter uses $89 (a 2-byte NOP).
              Gaau Hok Gwong Cheung (Ch) uses $8B (XAA immediate) as a 2-byte NOP. The game malfunctions after selecting Left from the main menu if that instruction is not emulated.
              Infiltrator uses $89 (a 2-byte NOP).
              Puzznic (all regions) (US release November 1990) uses $89 (a 2-byte NOP).
              Super Cars (U) (February 1991) uses $B3 (LAX).
            */
          },
          Instruction::ORA => { // OR with accum
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            self.registers.a |= operand;
            self.status.set_zero(( (self.registers.a & 0x00FF) == 0x0000 ) as u8);
            self.status.set_negative((self.registers.a & 0b10000000 != 0) as u8);
          },
          Instruction::PHA => {
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, self.registers.a).unwrap();
            self.registers.sp -= 1;
          },
          Instruction::PHP => {
            self.bus.write(STACK_START_ADDR + self.registers.sp as u16, self.status.flags).unwrap();
            self.registers.sp -= 1;
          },
          Instruction::PLA => {
            self.registers.sp += 1;
            self.registers.a = self.bus.read(STACK_START_ADDR + self.registers.sp as u16, false).unwrap();
            self.status.set_zero((self.registers.a == 0) as u8);
            self.status.set_negative((self.registers.a & 0b10000000 != 0) as u8);
          },
          Instruction::PLP => {
            self.registers.sp += 1;
            self.status.flags = self.bus.read(STACK_START_ADDR + self.registers.sp as u16, false).unwrap();
          },
          Instruction::ROL => {
            let operand;
  
            if matches!(addr_mode, AddressingMode::IMP) {
              operand = self.registers.a;
            } else {
              operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            }
  
            let result = ((operand as u16) << 1) | (self.status.get_carry() as u16);
  
            self.status.set_carry(((result & 0xFF00) != 0) as u8);
            self.status.set_zero(( (result & 0xFF) == 0) as u8);
            self.status.set_negative(((result & 0b10000000) != 0) as u8);
  
            if matches!(addr_mode, AddressingMode::IMP) {
              self.registers.a = (result & 0xFF) as u8;
            } else {
              self.bus.write(self.absolute_mem_address, (result & 0xFF) as u8).unwrap();
            }
          },
          Instruction::ROR => {
            let operand;
  
            if matches!(addr_mode, AddressingMode::IMP) {
              operand = self.registers.a;
            } else {
              operand = self.bus.read(self.absolute_mem_address, false).unwrap();
            }
  
            let result = ((self.status.get_carry() as u16) << 7) | ((operand as u16) >> 1);
  
            self.status.set_carry(((result & 0x01) != 0) as u8);
            self.status.set_zero(( (result & 0xFF) == 0) as u8);
            self.status.set_negative(((result & 0b10000000) != 0) as u8);
  
            if matches!(addr_mode, AddressingMode::IMP) {
              self.registers.a = (result & 0xFF) as u8;
            } else {
              self.bus.write(self.absolute_mem_address, (result & 0xFF) as u8).unwrap();
            }
          },
          Instruction::RTI => {
            self.registers.sp += 1;
            self.status.flags = self.bus.read(STACK_START_ADDR + self.registers.sp as u16, false).unwrap();
  
            self.status.set_brk_command(0);
            self.status.set_unused_bit(0);
  
            self.registers.sp += 1;
            self.registers.pc = self.bus.read_word_little_endian(STACK_START_ADDR + self.registers.sp as u16, false).unwrap();
            self.registers.sp += 1;
          },
          Instruction::RTS => {
            self.registers.sp += 1;
            self.registers.pc = self.bus.read_word_little_endian(STACK_START_ADDR + self.registers.sp as u16, false).unwrap();
            self.registers.sp += 1;
            self.registers.pc += 1;
          },
          Instruction::SBC => {
            let operand = self.bus.read(self.absolute_mem_address, false).unwrap();
  
            let inverted_value = operand as u16 ^ 0xFF;
  
            let result = self.registers.a as u16 + inverted_value as u16 + self.status.get_carry() as u16;
            
            self.status.set_carry( (result & 0xFF00 != 0) as u8);
            self.status.set_zero( (result & 0xFF == 0) as u8);
            self.status.set_negative( (result & 0b10000000 != 0) as u8);
            // A beautiful explanation for the following line can be found at https://youtu.be/8XmxKPJDGU0?t=2540
            self.status.set_overflow(( ((self.registers.a as u16 ^ result as u16) & (inverted_value as u16 ^ result as u16) & 0b10000000) != 0) as u8); 
            
            self.registers.a = (result & 0x00FF) as u8;
            // todo!("Might require an additional clock cycle :S");
          },
          Instruction::SEC => {
            self.status.set_carry(1);
          },
          Instruction::SED => {
            self.status.set_decimal_mode(1);
          },
          Instruction::SEI => {
            self.status.set_irq_disable(1);
          },
          Instruction::STA => {
            self.bus.write(self.absolute_mem_address, self.registers.a).unwrap();
          },
          Instruction::STX => {
            self.bus.write(self.absolute_mem_address, self.registers.x).unwrap();
          },
          Instruction::STY => {
            self.bus.write(self.absolute_mem_address, self.registers.y).unwrap();
          },
          Instruction::TAX => {
            self.registers.x = self.registers.a;
            self.status.set_zero((self.registers.x == 0) as u8);
            self.status.set_negative(((self.registers.x & 0b10000000) != 0) as u8);
          },
          Instruction::TAY => {
            self.registers.y = self.registers.a;
            self.status.set_zero((self.registers.y == 0) as u8);
            self.status.set_negative(((self.registers.y & 0b10000000) != 0) as u8);
          },
          Instruction::TSX => {
            self.registers.x = self.registers.sp;
            self.status.set_zero((self.registers.x == 0) as u8);
            self.status.set_negative(((self.registers.x & 0b10000000) != 0) as u8);
          },
          Instruction::TXA => {
            self.registers.a = self.registers.x;
            self.status.set_zero((self.registers.a == 0) as u8);
            self.status.set_negative(((self.registers.a & 0b10000000) != 0) as u8);
          },
          Instruction::TXS => {
            self.registers.sp = self.registers.x;
          },
          Instruction::TYA => {
            self.registers.a = self.registers.y;
            self.status.set_zero((self.registers.a == 0) as u8);
            self.status.set_negative(((self.registers.a & 0b10000000) != 0) as u8);
          },
          Instruction::XXX => {
            // Illegal opcode (no action)
          },
      }
  
    }
  
    pub fn reset(&mut self) {
  
      self.registers.a = 0;
      self.registers.x = 0;
      self.registers.y = 0;
  
      self.registers.sp = SP_RESET_ADDR;
      
      self.status.reset();
  
      // On reset, the cpu goes to a hard-wired address, takes a pointer
      // from that address (2 bytes), and sets the PC to the address specified
      self.registers.pc = self.bus.read_word_little_endian(PROGRAM_START_POINTER_ADDR, false).unwrap();
      self.registers.pc = 0xC000; // THIS LINE USED ONLY FOR NESTEST, TODO: Remove after testing is done :)
  
      self.absolute_mem_address = 0x0;
      self.relative_mem_address = 0x0;
      self.current_instruction_remaining_cycles = 8;
    }
  
    pub fn irq(&mut self) {
  
      if self.status.get_irq_disable() == 1 {
        return;
      }
    
      self.bus.write(STACK_START_ADDR + self.registers.sp as u16, ((self.registers.pc >> 8) & 0xFF) as u8).unwrap();
      self.registers.sp -= 1;
      self.bus.write(STACK_START_ADDR + self.registers.sp as u16, ( self.registers.pc       & 0xFF) as u8).unwrap();
      self.registers.sp -= 1;
  
      self.status.set_brk_command(0);
      self.status.set_unused_bit(1);
      self.status.set_irq_disable(1);
  
      self.bus.write(STACK_START_ADDR + self.registers.sp as u16, self.status.flags).unwrap();
      self.registers.sp -= 1;
  
      // Like on reset, the cpu goes to a hard-wired address, takes a pointer
      // from that address (2 bytes), and sets the PC to the address specified
      self.registers.pc = self.bus.read_word_little_endian(INTERRUPT_START_POINTER_ADDR, false).unwrap();
  
      self.current_instruction_remaining_cycles = 7;
  
    }
  
    pub fn nmi(&mut self) {

      self.bus.write(STACK_START_ADDR + self.registers.sp as u16, ((self.registers.pc >> 8) & 0xFF) as u8).unwrap();
      self.registers.sp -= 1;
      self.bus.write(STACK_START_ADDR + self.registers.sp as u16, ( self.registers.pc       & 0xFF) as u8).unwrap();
      self.registers.sp -= 1;
  
      self.status.set_brk_command(0);
      self.status.set_unused_bit(1);
      self.status.set_irq_disable(1);
  
      self.bus.write(STACK_START_ADDR + self.registers.sp as u16, self.status.flags).unwrap();
      self.registers.sp -= 1;
  
      // Like on reset, the cpu goes to a hard-wired address, takes a pointer
      // from that address (2 bytes), and sets the PC to the address specified
      self.registers.pc = self.bus.read_word_little_endian(NMI_START_POINTER_ADDR, false).unwrap();
  
      self.current_instruction_remaining_cycles = 8;
  
    }
  
    pub fn clock_cycle(&mut self) {
      if self.current_instruction_remaining_cycles == 0 {
        let next_instruction_code = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        let next_instruction_data: &InstructionData = &INSTRUCTION_TABLE[next_instruction_code as usize];
        self.current_instruction_remaining_cycles = next_instruction_data.cycles;
        // self.needs_additional_cycle = false;
        self.set_addressing_mode(&next_instruction_data.addressing_mode);
        self.execute_instruction(&next_instruction_data.instruction, &next_instruction_data.addressing_mode);
        // println!("Executed instruction {:?}", &next_instruction_data.instruction);
        // todo!("We should check if both the set_addressing_mode as well as the execute_instruction functions required more cycles, rather than\
        //       directly increasing the cycle counter inside of those functions. I'm not quite sure how the whole thing works, so I should read up :)");
        // if self.needs_additional_cycle {
        //   self.current_instruction_remaining_cycles += 1;
        // }
      }
      self.current_instruction_remaining_cycles -= 1;
    }
    
  
  }
  
  fn bytes_required_for_address(addressing_mode: &AddressingMode) -> u8 {
    match (addressing_mode) {
      AddressingMode::ACC => {
        0
      },
      AddressingMode::IMM => {
        1
      },
      AddressingMode::ABS => {
        2
      },
      AddressingMode::ZP0 => {
        1
      },
      AddressingMode::ZPX => {
        1
      },
      AddressingMode::ZPY => {
        1
      },
      AddressingMode::ABX => {
        2
      },
      AddressingMode::ABY => {
        2
      },
      AddressingMode::IMP => {
        0
      },
      AddressingMode::REL => {
        1
      },
      AddressingMode::INX => {
        1
      }
      AddressingMode::INY => {
        1
      },
      AddressingMode::IND => {
        2
      }
    }
  }
  
  
  pub fn disassemble(program: &Vec<u8>) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < program.len() {
      let instruction_opcode = *program.get(i).unwrap();
      let instruction_data = &INSTRUCTION_TABLE[instruction_opcode as usize];
      result.push_str(&format!("{:?}", instruction_data.instruction));
  
      let operation_bytes = bytes_required_for_address(&instruction_data.addressing_mode) as usize;
      for j in 0..operation_bytes {
        if program.get(i+(j+1)).is_some() {
          result.push_str(&format!(",{:02X}", program.get(i+(j+1)).unwrap()));
        }
      }
      result.push_str(" - ");
      i += 1 + operation_bytes;
    }
    return result;
  }
}



/*


File: Ben2C02.rs


*/

pub mod Ben2C02 {
  use std::sync::{Arc, Mutex};

  use super::{graphics::Color, Device::Device, bitwise_utils};
  use rand::Rng;

  pub const PPU_MEMORY_BOUNDS: (u16, u16) = (0x2000, 0x3FFF);

  fn create_palette_vis_buffer() -> [Color; 64]{
    let mut buffer= [Color::new(0, 0, 0);64];

    // Original color assignments taken from https://github.com/OneLoneCoder/olcNES/blob/master/Part%20%233%20-%20Buses%2C%20Rams%2C%20Roms%20%26%20Mappers/olc2C02.cpp
    // Author: David Barr, aka javidx9 or OneLoneCoder

    buffer[0x00] = Color::new(84, 84, 84);
    buffer[0x01] = Color::new(0, 30, 116);
    buffer[0x02] = Color::new(8, 16, 144);
    buffer[0x03] = Color::new(48, 0, 136);
    buffer[0x04] = Color::new(68, 0, 100);
    buffer[0x05] = Color::new(92, 0, 48);
    buffer[0x06] = Color::new(84, 4, 0);
    buffer[0x07] = Color::new(60, 24, 0);
    buffer[0x08] = Color::new(32, 42, 0);
    buffer[0x09] = Color::new(8, 58, 0);
    buffer[0x0A] = Color::new(0, 64, 0);
    buffer[0x0B] = Color::new(0, 60, 0);
    buffer[0x0C] = Color::new(0, 50, 60);
    buffer[0x0D] = Color::new(0, 0, 0);
    buffer[0x0E] = Color::new(0, 0, 0);
    buffer[0x0F] = Color::new(0, 0, 0);

    buffer[0x10] = Color::new(152, 150, 152);
    buffer[0x11] = Color::new(8, 76, 196);
    buffer[0x12] = Color::new(48, 50, 236);
    buffer[0x13] = Color::new(92, 30, 228);
    buffer[0x14] = Color::new(136, 20, 176);
    buffer[0x15] = Color::new(160, 20, 100);
    buffer[0x16] = Color::new(152, 34, 32);
    buffer[0x17] = Color::new(120, 60, 0);
    buffer[0x18] = Color::new(84, 90, 0);
    buffer[0x19] = Color::new(40, 114, 0);
    buffer[0x1A] = Color::new(8, 124, 0);
    buffer[0x1B] = Color::new(0, 118, 40);
    buffer[0x1C] = Color::new(0, 102, 120);
    buffer[0x1D] = Color::new(0, 0, 0);
    buffer[0x1E] = Color::new(0, 0, 0);
    buffer[0x1F] = Color::new(0, 0, 0);

    buffer[0x20] = Color::new(236, 238, 236);
    buffer[0x21] = Color::new(76, 154, 236);
    buffer[0x22] = Color::new(120, 124, 236);
    buffer[0x23] = Color::new(176, 98, 236);
    buffer[0x24] = Color::new(228, 84, 236);
    buffer[0x25] = Color::new(236, 88, 180);
    buffer[0x26] = Color::new(236, 106, 100);
    buffer[0x27] = Color::new(212, 136, 32);
    buffer[0x28] = Color::new(160, 170, 0);
    buffer[0x29] = Color::new(116, 196, 0);
    buffer[0x2A] = Color::new(76, 208, 32);
    buffer[0x2B] = Color::new(56, 204, 108);
    buffer[0x2C] = Color::new(56, 180, 204);
    buffer[0x2D] = Color::new(60, 60, 60);
    buffer[0x2E] = Color::new(0, 0, 0);
    buffer[0x2F] = Color::new(0, 0, 0);

    buffer[0x30] = Color::new(236, 238, 236);
    buffer[0x31] = Color::new(168, 204, 236);
    buffer[0x32] = Color::new(188, 188, 236);
    buffer[0x33] = Color::new(212, 178, 236);
    buffer[0x34] = Color::new(236, 174, 236);
    buffer[0x35] = Color::new(236, 174, 212);
    buffer[0x36] = Color::new(236, 180, 176);
    buffer[0x37] = Color::new(228, 196, 144);
    buffer[0x38] = Color::new(204, 210, 120);
    buffer[0x39] = Color::new(180, 222, 120);
    buffer[0x3A] = Color::new(168, 226, 144);
    buffer[0x3B] = Color::new(152, 226, 180);
    buffer[0x3C] = Color::new(160, 214, 228);
    buffer[0x3D] = Color::new(160, 162, 160);
    buffer[0x3E] = Color::new(0, 0, 0);
    buffer[0x3F] = Color::new(0, 0, 0);
    return buffer;
  }

  pub struct StatusRegister {
    flags: u8
  }

  impl StatusRegister {
  
    fn new() -> StatusRegister {
      return StatusRegister {
        flags: 0b00000000
      }
    }
  
    pub fn get_sprite_overflow(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 5);
    }
  
    fn set_sprite_overflow(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 5, value);
    }
  
    pub fn get_sprite_zero_hit(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 6);
    }
  
    fn set_sprite_zero_hit(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 6, value);
    }
  
    pub fn get_vertical_blank(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 7);
    }
  
    fn set_vertical_blank(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 7, value);
    }
  }

  pub struct MaskRegister {
    flags: u8
  }
  
  impl MaskRegister {
  
    fn new() -> MaskRegister {
      return MaskRegister {
        flags: 0b00000000 
      }
    }

    pub fn get_grayscale(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 0);
    }
  
    fn set_grayscale(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 0, value);
    }
  
    pub fn get_render_background_left(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 1);
    }
  
    fn set_render_background_left(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 1, value);
    }
  
    pub fn get_render_sprites_left(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 2);
    }
  
    fn set_render_sprites_left(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 2, value);
    }
  
    pub fn get_render_background(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 3);
    }
  
    fn set_render_background(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 3, value);
    }
  
    pub fn get_render_sprites(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 4);
    }
  
    fn set_render_sprites(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 4, value);
    }
  
    pub fn get_enhance_red(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 5);
    }
  
    fn set_enhance_red(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 5, value);
    }
  
    pub fn get_enhance_green(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 6);
    }
  
    fn set_enhance_green(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 6, value);
    }
  
    pub fn get_enhance_blue(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 7);
    }
  
    fn set_enhance_blue(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 7, value);
    }
  }

  pub struct ControllerRegister {
    flags: u8
  }
  
  impl ControllerRegister {
  
    fn new() -> ControllerRegister {
      return ControllerRegister {
        flags: 0b00000000 
      }
    }

    pub fn get_nametable_x(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 0);
    }
  
    fn set_nametable_x(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 0, value);
    }
  
    pub fn get_nametable_y(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 1);
    }
  
    fn set_nametable_y(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 1, value);
    }
  
    pub fn get_increment_mode(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 2);
    }
  
    fn set_increment_mode(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 2, value);
    }
  
    pub fn get_pattern_sprite(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 3);
    }
  
    fn set_pattern_sprite(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 3, value);
    }
  
    pub fn get_pattern_background(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 4);
    }
  
    fn set_pattern_background(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 4, value);
    }
  
    pub fn get_sprite_size(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 5);
    }
  
    fn set_sprite_size(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 5, value);
    }
  
    pub fn get_slave_mode(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 6);
    }
  
    fn set_slave_mode(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 6, value);
    }
  
    pub fn get_enable_nmi(&self) -> u8 {
      return bitwise_utils::get_bit(self.flags, 7);
    }
  
    fn set_enable_nmi(&mut self, value: u8) {
      bitwise_utils::set_bit(&mut self.flags, 7, value);
    }
  }

  pub struct Ben2C02 {
    memory_bounds: (u16, u16),

    cartridge: Arc<Mutex<dyn Device>>,

    scan_line: i16,
    cycle: i16,
    pub frame_render_complete: bool,
    pub trigger_cpu_nmi: bool,

    controller_reg: ControllerRegister,
    mask_reg: MaskRegister,
    pub status_reg: StatusRegister,
    writing_high_byte_of_addr: bool,
    ppu_addr: u16,
    ppu_data_read_buffer: u8,

    pattern_tables: [[u8; 4096]; 2],
    pattern_tables_mem_bounds: (u16, u16),
    name_tables: [[u8; 1024]; 2],
    name_tables_mem_bounds: (u16, u16),
    pub palette: [u8; 32],
    palette_mem_bounds: (u16, u16),

    
    // These arrays are used for emulator visualization, thus the higher level Color structure
    pub screen_vis_buffer: [[Color; 256]; 240],
    pub pattern_tables_vis_buffer: [[[Color; 128]; 128]; 2],
    name_tables_vis_buffer: [[[Color; 256]; 240]; 2],
    pub palette_vis_bufer: [Color; 64],
  }

  impl Ben2C02 {
    pub fn new(cartridge: Arc<Mutex<dyn Device>>) -> Ben2C02 {
      return Ben2C02 {
        memory_bounds: PPU_MEMORY_BOUNDS,
        cartridge: cartridge,
        
        scan_line: 0,
        cycle: 0,
        frame_render_complete: false,
        trigger_cpu_nmi: false,

        controller_reg: ControllerRegister::new(),
        mask_reg: MaskRegister::new(),
        status_reg: StatusRegister::new(),
        writing_high_byte_of_addr: true,
        ppu_addr: 0,
        ppu_data_read_buffer: 0,

        pattern_tables: [[0; 4096]; 2],
        pattern_tables_mem_bounds: (0x0000, 0x1FFF),
        name_tables: [[0; 1024]; 2],
        name_tables_mem_bounds: (0x2000, 0x3EFF),
        palette: [0; 32],
        palette_mem_bounds: (0x3F00, 0x3FFF),


        palette_vis_bufer: create_palette_vis_buffer(),
        screen_vis_buffer: [[Color::new(0, 0, 0); 256]; 240],
        name_tables_vis_buffer: [[[Color::new(0, 0, 0); 256]; 240]; 2],
        pattern_tables_vis_buffer: [[[Color::new(0, 0, 0); 128]; 128]; 2],
      }
    }

    fn in_pattern_table_memory_bounds(&self, addr: u16) -> bool {
      return addr >= self.pattern_tables_mem_bounds.0 && addr <= self.pattern_tables_mem_bounds.1;
    }

    fn in_name_table_memory_bounds(&self, addr: u16) -> bool {
      return addr >= self.name_tables_mem_bounds.0 && addr <= self.name_tables_mem_bounds.1;
    }

    fn in_palette_memory_bounds(&self, addr: u16) -> bool {
      return addr >= self.palette_mem_bounds.0 && addr <= self.palette_mem_bounds.1;
    }

    pub fn clock_cycle(&mut self) {

      let mut rng = rand::thread_rng();
      if (self.cycle < 256 && self.scan_line < 240 && self.scan_line != -1) {
        self.screen_vis_buffer[self.scan_line as usize][self.cycle as usize] = self.palette_vis_bufer[rng.gen_range(0..(self.palette_vis_bufer.len()-1))]; // Temporary
      }
        
      self.cycle += 1;
      if self.cycle > 340 {
        self.cycle = 0;
        self.scan_line += 1;
        if (self.scan_line > 261) {
          self.scan_line = -1;
          self.frame_render_complete = true;
        }
      }

      if (self.scan_line == 241 && self.cycle == 1) {
        self.status_reg.set_vertical_blank(1);
        if (self.controller_reg.get_enable_nmi() ==  1) {
          self.trigger_cpu_nmi = true;
        }
      }

      if (self.scan_line == -1 && self.cycle == 1) {
        self.status_reg.set_vertical_blank(0);
      }
    }


    // Refer to https://www.nesdev.org/wiki/PPU_programmer_reference#Pattern_tables
    // for a clearer explanation :)
    pub fn update_pattern_tables_vis_buffer(&mut self, palette_id: u8) {
      const PATTERN_TABLE_SIZE: u16 = 4096;
      for pattern_table_id in 0..2 {
        let start_addr = PATTERN_TABLE_SIZE * pattern_table_id;
        for tileIndexRow in 0..16 {
          for tileIndexCol in 0..16 {
            for pixelRow in 0..8 {
              let tile_lsb_data = self.read_from_ppu_bus(start_addr + tileIndexCol * 16 + tileIndexRow * 256 + pixelRow).unwrap();
              let tile_msb_data = self.read_from_ppu_bus(start_addr + tileIndexCol * 16 + tileIndexRow * 256 + pixelRow + 8).unwrap();
              for pixelCol in 0..8 {
                let pixel_value_lsb = bitwise_utils::get_bit(tile_lsb_data, 7 - pixelCol);
                let pixel_value_msb = bitwise_utils::get_bit(tile_msb_data, 7 - pixelCol);
                let pixel_value = (pixel_value_msb << 1) + pixel_value_lsb;
                let pixel_color = self.get_color_from_palette(pixel_value, palette_id);
                self.pattern_tables_vis_buffer[pattern_table_id as usize][(tileIndexCol as u8 * 8 + pixelCol) as usize][(tileIndexRow * 8 + pixelRow) as usize] = pixel_color;
              }
            }
          }
        }
      }
      
    }

    fn get_color_from_palette(&self, pixel_value: u8, palette_id: u8) -> Color {
      let pixel_color_code = self.palette[(palette_id * 4 + pixel_value) as usize];
      return self.palette_vis_bufer[pixel_color_code as usize];
    }

    fn write_to_ppu_memory(&mut self, addr: u16, data: u8) -> Result<(), String>{
      if self.in_pattern_table_memory_bounds(addr) {
		    self.pattern_tables[((addr & 0x1000) > 0) as usize][(addr & 0x0FFF) as usize] = data;
        return Ok(());
      }
      else if self.in_name_table_memory_bounds(addr) {
        // TODO: implement
        return Ok(());
      }
      else if self.in_palette_memory_bounds(addr) {
        // Address space is $3F00-$3F1F, mirrored in the range $3F00-$3FFF
        self.palette[((addr & 0x0FF) % 32) as usize] = data;
        return Ok(());
      }
      else {
        return  Err(format!("Tried writing to PPU memory, but provided address wasn't within pattern_table,
                  name_table, or palette memory bounds!. Provided address was 0x{:X}", addr));
      }
    }

    fn read_from_ppu_memory(&self, addr: u16) -> Result<u8, String>{
      if self.in_pattern_table_memory_bounds(addr) {
		    let data = self.pattern_tables[((addr & 0x1000) > 0) as usize][(addr & 0x0FFF) as usize];
        return Ok(data);
      }
      else if self.in_name_table_memory_bounds(addr) {
        // TODO: implement
        return Ok(0);
      }
      else if self.in_palette_memory_bounds(addr) {
        let data = self.palette[((addr & 0x0FF) % 32) as usize];
        return Ok(data);
      }
      else {
        return  Err(format!("Tried reading from PPU memory, but provided address wasn't within pattern_table,
                  name_table, or palette memory bounds!. Provided address was 0x{:X}", addr));
      }
    }

    fn read_from_ppu_bus(&self, addr: u16) -> Result<u8, String> {
      let read_from_cartridge = self.cartridge.lock().unwrap().read(addr);
      match read_from_cartridge {
        Ok(retrieved_data) => {
          return Ok(retrieved_data);
        },
        Err(message) => {
          println!("Tried to read from cartridge, but failed with error: {}. Reading from PPU internal memory instead :)" , message);
          return Ok(self.read_from_ppu_memory(self.ppu_addr).unwrap());
        }
      }
    }

    fn write_to_ppu_bus(&mut self, addr: u16, data: u8) -> Result<(), String> {
      let write_to_cartridge = self.cartridge.lock().unwrap().write(addr, data);
      match write_to_cartridge {
        Ok(()) => {
          return Ok(());
        },
        Err(message) => {
          println!("Tried to write to cartridge, but failed with error: {}. Writing to PPU internal memory instead :)" , message);
          return Ok(self.write_to_ppu_memory(addr, data).unwrap());
        }
      }
    }
  
  }

  impl Device for Ben2C02 {

    fn in_memory_bounds(&self, addr: u16)-> bool {
      return  addr >= self.memory_bounds.0 && addr <= self.memory_bounds.1;
    }

    fn write(&mut self, addr: u16, data: u8) -> Result<(), String> {
      if self.in_memory_bounds(addr) {
        let mirrored_addr = addr & 0x0007;
        match mirrored_addr {
          0x0 => { // Control
            self.controller_reg.flags = data;
          },
          0x1 => { // Mask
            self.mask_reg.flags = data;
          },
          0x2 => { // Status

          },
          0x3 => { // OAM Address

          },
          0x4 => { // OAM Data

          },
          0x5 => { // Scroll

          },
          0x6 => { // PPU Address
            if self.writing_high_byte_of_addr {
              self.ppu_addr &= 0xFF;
              self.ppu_addr += (data as u16) << 8; 
            } else {
              self.ppu_addr &= 0xFF00;
              self.ppu_addr += (data as u16); 
            }
            self.writing_high_byte_of_addr = !self.writing_high_byte_of_addr;
          },
          0x7 => { // PPU data
            self.write_to_ppu_bus(self.ppu_addr, data).unwrap();
            self.ppu_addr += 1; // TODO: depending on increment mode, we might want to add 32 bytes instead of 1
            return Ok(());
          },
          _ => return Err(String::from("Error while mirroring address in PPU write() function!"))
        }
        return Ok(());
      } else {
        return Err(String::from("Tried to write outside PPU bounds!"));
      }
    }

    fn read(&mut self, addr: u16) -> Result<u8, String> {
      if self.in_memory_bounds(addr) {
        let mirrored_addr = addr & 0x0007;
        match mirrored_addr {
          0x0 => { // Control
            return Ok(self.controller_reg.flags);
          },
          0x1 => { // Mask
            return Ok(self.mask_reg.flags);

          },
          0x2 => { // Status
            // We use the 3 most significant bits of the status register
            // and the 5 least sifgnificant bits of the data buffer
            let result = (self.status_reg.flags & 0xE0) + (self.ppu_data_read_buffer & 0x1F);
            self.status_reg.set_vertical_blank(0);
            self.writing_high_byte_of_addr = true;
            return Ok(result);
          },
          0x3 => { // OAM Address
            return Ok(0);
          },
          0x4 => { // OAM Data
            return Ok(0);
          },
          0x5 => { // Scroll
            return Ok(0);
          },
          0x6 => { // PPP Address
            return Ok(0);
          },
          0x7 => { // PPU data
            let read_result = self.read_from_ppu_bus(self.ppu_addr).unwrap();

            let return_value : u8;
            // Unless reading from palette memory, we return the value that is currently 
            // stored on the read buffer, and then update the buffer with the 
            // data located at self.ppu_addr
            // Essentially, most read() operations are delayed one cycle.
            if self.in_palette_memory_bounds(self.ppu_addr) {
              self.ppu_data_read_buffer = read_result;
              return_value = read_result;
            } else {
              return_value = self.ppu_data_read_buffer;
              self.ppu_data_read_buffer = read_result;
            }
            self.ppu_addr += 1; // TODO: depending on increment mode, we might want to add 32 bytes instead of 1
            return Ok(return_value);

          },
          _ => return Err(String::from("Error while mirroring address in PPU write() function!"))
        }
      } else {
        return Err(String::from("Tried to read outside PPU bounds!"));
      }
    }
  }
}




/*


mapper.rs


*/

pub mod Mapper {
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
}


/*


cartridge.rs


*/

pub mod Cartridge {
  use std::{fs, rc::Rc, sync::{Mutex, Arc}};

  use super::{Mapper::{Mapper, Mapper000}, Device::Device};

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

    let mapper = create_mapper_from_number((header.mapper2 << 4) & header.mapper1, prg_chunks, chr_chunks).unwrap();

    let mut cartridge = Cartridge::new(header, mapper);

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
    mapper: Box<dyn Mapper>
  }

  impl Cartridge {
    fn new(rom_header: RomHeader, mapper: Box<dyn Mapper>) -> Cartridge {
      return Cartridge {
        cpu_memory_bounds: (0x8000, 0xFFFF),
        ppu_memory_bounds: (0x0000, 0x1FFF),
        rom_header,
        PRG_data: vec![],
        CHR_data: vec![],
        mapper
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
            let data = self.CHR_data.get(mapped_addr as usize).unwrap();
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
}





/*


File: bus.rs


*/

pub mod Bus16Bit {
  use std::sync::{Arc, Mutex};

  use super::{Device::Device, Ben2C02::Ben2C02, hex_utils, Cartridge::create_cartridge_from_ines_file, Ram::Ram2K};

  pub struct Bus16Bit {
    pub devices: Vec<Arc<Mutex<dyn Device>>>,
    pub PPU: Arc<Mutex<Ben2C02>>
  }
  
  // Assumed to be a 16-bit bus
  impl Bus16Bit {
  
    pub fn new(rom_file_path: &str) -> Bus16Bit {
      let ram = Arc::new(Mutex::new(Ram2K::new((0x0000, 0x1FFF))));
      let apu_mock = Arc::new(Mutex::new(Ram2K::new((0x4000, 0x4017))));
      let cartridge = Arc::new(Mutex::new(create_cartridge_from_ines_file(rom_file_path).unwrap()));
      let PPU = Arc::new(Mutex::new(Ben2C02::new(cartridge.clone())));
  
      let mut devices: Vec<Arc<Mutex<dyn Device>>> = vec![];
      devices.push(ram.clone());
      devices.push(apu_mock.clone());
      devices.push(PPU.clone());
      devices.push(cartridge.clone());
      return Bus16Bit {
        devices,
        PPU
      }
    }
  
    pub fn read(&mut self, addr: u16, readOnly: bool) -> Result<u8, String> {
      for device_mutex in self.devices.iter() {
        let mut device = device_mutex.lock().unwrap();
        if device.in_memory_bounds(addr) {
          return device.read(addr);
        }
      }
      return Ok(0);
      // return Err(String::from(format!("Error reading from memory bus (No device found in given address: 0x{:x}).", addr)));
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
      for device_mutex in self.devices.iter_mut() {
        let mut device = device_mutex.lock().unwrap();
        if device.in_memory_bounds(addr) {
          return device.write(addr, content);
        }
      }
      return Ok(());
      // return Err(format!("Error writing to memory bus (No device found in given address: 0x{:X}", addr));
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
  
    pub fn get_PPU(&mut self) -> Arc<Mutex<Ben2C02>> {
      return self.PPU.clone();
    }
  }
  
  
  #[cfg(test)]
  mod bus_tests {
    use super::Bus16Bit;
  
    // #[test]
    // fn test_get_memory_content_as_string() {
    //   let bus = Bus16Bit::new("hey_mona.nes");
    //   println!("{}", bus.get_memory_content_as_string(0, 100));
    // }
  
  }
}