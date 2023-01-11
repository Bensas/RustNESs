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


File: utils.rs


*/

mod bitwise_utils {
  pub fn get_bit(source: u8, bit_pos: u8) -> u8{ // bit_pos counted from least significant to most significant
    return source & (1 << bit_pos);
  }

  pub fn set_bit(target: &mut u8, bit_pos: u8, new_value: u8) {
    match new_value {
      0 => *target &= !(1 << bit_pos),
      1 => *target |= (1 << bit_pos),
      _ => panic!("Tried to set_bit with a value other than 0 or 1")
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
  pc: u16
}

struct Status {
  flags: u8
}

impl Status {

  fn get_carry(&self) -> u8 {
    return bitwise_utils::get_bit(self.flags, 0);
  }

  fn set_carry(&mut self, value: u8) {
    bitwise_utils::set_bit(&mut self.flags, 0, value);
  }

  fn get_zero(&self) -> u8 {
    return bitwise_utils::get_bit(self.flags, 1);
  }

  fn set_zero(&mut self, value: u8) {
    bitwise_utils::set_bit(&mut self.flags, 1, value);
  }

  fn get_irq_disable(&self) -> u8 {
    return bitwise_utils::get_bit(self.flags, 2);
  }

  fn set_irq_disable(&mut self, value: u8) {
    bitwise_utils::set_bit(&mut self.flags, 2, value);
  }

  fn get_decimal_mode(&self) -> u8 {
    return bitwise_utils::get_bit(self.flags, 3);
  }

  fn set_decimal_mode(&mut self, value: u8) {
    bitwise_utils::set_bit(&mut self.flags, 3, value);
  }

  fn get_brk_command(&self) -> u8 {
    return bitwise_utils::get_bit(self.flags, 4);
  }

  fn set_brk_command(&mut self, value: u8) {
    bitwise_utils::set_bit(&mut self.flags, 4, value);
  }

  fn get_overflow(&self) -> u8 {
    return bitwise_utils::get_bit(self.flags, 6);
  }

  fn set_overflow(&mut self, value: u8) {
    bitwise_utils::set_bit(&mut self.flags, 6, value);
  }

  fn get_negative(&self) -> u8 {
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
// Thank you so much dude
const INSTRUCTION_TABLE: [InstructionData; 256] = 
[
	InstructionData{instruction: Instruction::BRK, addressing_mode: AddressingMode::IMM, cycles: 7 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::INX, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 8 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ZP0, cycles: 3 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ZP0, cycles: 5 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 5 },InstructionData{instruction: Instruction::PHP, addressing_mode: AddressingMode::IMP, cycles: 3 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::IMM, cycles: 2 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ACC, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 2 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 4 },InstructionData{instruction: Instruction::ORA, addressing_mode: AddressingMode::ABS, cycles: 4 },InstructionData{instruction: Instruction::ASL, addressing_mode: AddressingMode::ABS, cycles: 6 },InstructionData{instruction: Instruction::XXX, addressing_mode: AddressingMode::IMP, cycles: 6 },
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


pub struct Ben6502 {
  bus: Bus,

  status: Status,
  registers: Registers,


  current_instruction_remaining_cycles: u8,
  needs_additional_cycle: bool,
  fetched_data: u8,
  absolute_mem_address: u16,
  relative_mem_address: u8,

  

}

impl Ben6502 {
  fn new(mem_bus: Bus) -> Ben6502 {
    return Ben6502 {
      bus: mem_bus,
      status: Status { flags: 0 },
      registers: Registers { a: 0, x: 0, y: 0, sp: 0, pc: 0 },
      current_instruction_remaining_cycles: 0,
      needs_additional_cycle: false,
      fetched_data: 0,
      absolute_mem_address: 0,
      relative_mem_address: 0
    };
  }

  fn set_addressing_mode(&mut self, mode: &AddressingMode) {
    match mode {
      AddressingMode::ACC => {
        self.fetched_data = self.registers.a;
      },
      AddressingMode::IMM => {
        self.absolute_mem_address = self.registers.pc;
        self.registers.pc += 1;
      },
      AddressingMode::ABS => {
        let addr_low= self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        let addr_high = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        self.absolute_mem_address = (addr_high as u16) << 8 + (addr_low as u16);
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
        let addr_low = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        let addr_high = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        self.absolute_mem_address = ((addr_high as u16) << 8 + (addr_low as u16)) + self.registers.x as u16;

        if (self.absolute_mem_address > (addr_high as u16) << 8) { // We crossed a page boundary after adding X to the address
          self.current_instruction_remaining_cycles += 1;
        }
      },
      AddressingMode::ABY => {
        let addr_low = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        let addr_high = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        self.absolute_mem_address = ((addr_high as u16) << 8 + (addr_low as u16)) + self.registers.y as u16;

        if (self.absolute_mem_address > (addr_high as u16) << 8) { // We crossed a page boundary after adding X to the address
          self.current_instruction_remaining_cycles += 1;
        }
      },
      AddressingMode::IMP => {
        // Implied addressing means that no address is required to execute the instruction
      },
      AddressingMode::REL => {
        self.relative_mem_address = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
      },
      AddressingMode::INX => {
        let instruction_addr = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;

        let abs_address_of_low_byte = (instruction_addr as u16 + self.registers.x as u16) & 0x00FF;

        let low_byte = self.bus.read(abs_address_of_low_byte, false).unwrap();
        let high_byte = self.bus.read(abs_address_of_low_byte + 1, false).unwrap();

        self.absolute_mem_address = (high_byte as u16) << 8 + (low_byte as u16);
        
      }
      AddressingMode::INY => {
        let instruction_addr = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;

        let abs_address_of_low_byte = (instruction_addr as u16 + self.registers.y as u16) & 0x00FF;

        let low_byte = self.bus.read(abs_address_of_low_byte, false).unwrap();
        let high_byte = self.bus.read(abs_address_of_low_byte + 1, false).unwrap();

        self.absolute_mem_address = (high_byte as u16) << 8 + (low_byte as u16);
      },
      AddressingMode::IND => {
        let addr_low = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;
        let addr_high = self.bus.read(self.registers.pc, false).unwrap();
        self.registers.pc += 1;

        let abs_address_of_low_byte = (addr_high as u16) << 8 + (addr_low as u16);;
        
        let low_byte = self.bus.read(abs_address_of_low_byte, false).unwrap();
        let high_byte: u8;

        if (addr_low == 0x00FF) { // We must do this weird thing to simulate a hardware bug in the CPU with page boundaries. https://www.nesdev.org/6502bugs.txt
          high_byte = self.bus.read(abs_address_of_low_byte & 0xFF00, false).unwrap();
        } else {
          high_byte = self.bus.read(abs_address_of_low_byte + 1, false).unwrap();
        }

        self.absolute_mem_address = (high_byte as u16) << 8 + (low_byte as u16);
      },
      _ => return
      
    }
  }


  fn execute_instruction(&mut self, instruction: &Instruction) {

  }

  fn reset() {

  }

  fn irq() {

  }

  fn nmi() {

  }

  fn clock_cycle(&mut self) {
    if self.current_instruction_remaining_cycles == 0 {
      let next_instruction_code = self.bus.read(self.registers.pc, false).unwrap();
      self.registers.pc += 1;
      let next_instruction_data: &InstructionData = &INSTRUCTION_TABLE[next_instruction_code as usize];
      self.current_instruction_remaining_cycles = next_instruction_data.cycles;
      // self.needs_additional_cycle = false;
      self.set_addressing_mode(&next_instruction_data.addressing_mode);
      self.execute_instruction(&next_instruction_data.instruction);
      // if self.needs_additional_cycle {
      //   self.current_instruction_remaining_cycles += 1;
      // }
    }
    self.current_instruction_remaining_cycles -= 1;
  }
  

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
  let cpu: Ben6502 = Ben6502::new(mem_bus);
}
