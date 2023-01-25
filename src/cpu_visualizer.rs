/*


cpu-visualizer.rs


*/

struct CPUVisualizer {
  cpu: Ben6502
}

impl CPUVisualizer {
  fn load_program_to_ram(&mut self, program_str: &str, start_addr: u16) {
    let hex_strings = program_str.split(" ");

    let mut curr_addr = start_addr;
    for hex_str in hex_strings.into_iter() {
      let value: u8 = u8::from_str_radix(hex_str, 16).unwrap();
      self.cpu.bus.write(curr_addr, value).unwrap();
      curr_addr += 1;
    }
  }
}

#[derive(Debug, Clone, Copy)]
enum Message {
  NextInstruction,
  LoadProgram
}

impl Sandbox for CPUVisualizer {
  type Message = Message;

  fn new() -> Self {
    let mut mem_bus = Bus16Bit::new();
    mem_bus.write(emulation::PROGRAM_START_POINTER_ADDR, 0x00).unwrap();
    mem_bus.write(emulation::PROGRAM_START_POINTER_ADDR + 1, 0x80).unwrap();
    let cpu: Ben6502 = Ben6502::new(mem_bus);
    Self { 
      cpu: cpu
    }
  }

  fn title(&self) -> String {
      String::from("6502 Emulation :)")
  }

  fn update(&mut self, message: Message) {
      match message {
          Message::NextInstruction => {
              self.cpu.clock_cycle();
          },
          Message::LoadProgram => {
            self.load_program_to_ram("A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA", 0x8000);
        }
      }
  }

  fn view(&self) -> Element<Message> {
    let ram_start_addr = 0x8000;
    let ram_end_addr = 0x8010;

    let stack_start_addr = 0x100 + emulation::SP_RESET_ADDR as u16 - 100;
    let stack_end_addr = 0x100 + emulation::SP_RESET_ADDR as u16;
    column![
        text(format!("RAM contents (Addr 0x{:x} - 0x{:x}):", 0x00, 0x50)),
        text(self.cpu.bus.get_memory_content_as_string(0x00, 0x50)).size(20),
        text(format!("RAM contents (Addr 0x{:x} - 0x{:x}):", ram_start_addr, ram_end_addr)),
        text(self.cpu.bus.get_memory_content_as_string(ram_start_addr, ram_end_addr)).size(20),
        text(format!("Stack contents (Addr 0x{:x} - 0x{:x}):", stack_start_addr, stack_end_addr)),
        text(self.cpu.bus.get_memory_content_as_string(stack_start_addr, stack_end_addr)).size(20),
        button("Next Clock Cycle").on_press(Message::NextInstruction),
        button("Load Program").on_press(Message::LoadProgram),
        row![
          column![
            text("Cpu registers:").size(20),
            text("A: "),
            text(self.cpu.registers.a.to_string()),
            text("X: "),
            text(self.cpu.registers.x.to_string()),
            text("Y: "),
            text(self.cpu.registers.y.to_string()),
            text("PC(hex): "),
            text(hex_utils::decimal_word_to_hex_str(self.cpu.registers.pc)),
            text("SP(hex): "),
            text(hex_utils::decimal_byte_to_hex_str(self.cpu.registers.sp))
          ],
          column![
            text("Cpu flags:").size(20),
            text("Carry: "),
            text(self.cpu.status.get_carry().to_string()),
            text("Zero: "),
            text(self.cpu.status.get_zero().to_string()),
            text("Negative: "),
            text(self.cpu.status.get_negative().to_string()),
            text("overflow: "),
            text(self.cpu.status.get_overflow().to_string()),
            text("Decimal mode: "),
            text(self.cpu.status.get_decimal_mode().to_string()),
            text("BRK command: "),
            text(self.cpu.status.get_brk_command().to_string()),
            text("IRQ Disable: "),
            text(self.cpu.status.get_irq_disable().to_string())
          ],
          column![
            text(format!("Remaining cycles in curr instruction: {}", self.cpu.current_instruction_remaining_cycles)).size(15),
          ]
        ]
    ]
    .padding(20)
    .align_items(Alignment::Center)
    .into()
  }
}