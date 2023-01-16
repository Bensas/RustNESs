mod emulation;
use emulation::{ Bus16Bit, Ben6502, hex_utils};


use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Sandbox, Settings};

fn main() {
  CPUVisualizer::run(Settings::default());
}

struct CPUVisualizer {
  cpu: Ben6502
}

#[derive(Debug, Clone, Copy)]
enum Message {
  NextInstruction,
}

impl Sandbox for CPUVisualizer {
  type Message = Message;

  fn new() -> Self {
    let mut mem_bus = Bus16Bit::new();
    mem_bus.write(0xFFFC, 0x00).unwrap();
    mem_bus.write(0xFFFD, 0x80).unwrap();

    mem_bus.write(0x8000, 0x38).unwrap();


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
          }
      }
  }

  fn view(&self) -> Element<Message> {
      column![
          text("RAM contents (Addr 0xFFFA - 0xFFFF):"),
          text(self.cpu.bus.get_memory_content_as_string(0x8000, 0x8010)).size(20),
          button("Next Instruction").on_press(Message::NextInstruction),
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
            ]
          ]
      ]
      .padding(20)
      .align_items(Alignment::Center)
      .into()
  }
}