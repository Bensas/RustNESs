mod emulation;
use emulation::{ Bus16Bit, Ben6502};


use iced::widget::{button, column, text};
use iced::{Alignment, Element, Sandbox, Settings};

fn main() {
  println!("Hello, world!");
  let mem_bus = Bus16Bit::new();
  let cpu: Ben6502 = Ben6502::new(mem_bus);

  Counter::run(Settings::default());
}

struct Counter {
  cpu: Ben6502
}

#[derive(Debug, Clone, Copy)]
enum Message {
  NextInstruction,
}

impl Sandbox for Counter {
  type Message = Message;

  fn new() -> Self {
    let bus = Bus16Bit::new();
      Self { 
        cpu: Ben6502::new(bus)
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
          text(self.cpu.bus.get_memory_content_as_string(0, 100)).size(50),
          button("Next Instruction").on_press(Message::NextInstruction),
      ]
      .padding(20)
      .align_items(Alignment::Center)
      .into()
  }
}