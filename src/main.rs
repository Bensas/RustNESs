mod emulation;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Mutex, Arc};

use emulation::{ Bus16Bit, Ben6502, hex_utils, Ben2C02, Ram2K, Cartridge, Device};


use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Sandbox, Settings, Renderer, event, Application, Subscription, executor, Theme, Command};

use iced::keyboard::{self, KeyCode, Modifiers};

use iced_native::Event;
use iced_native::Length;


fn main() {
  RustNESs::run(Settings::default());
}


struct RustNESs {
  cpu: Ben6502,
  current_cycle: u16,

  paused: bool,
}

impl RustNESs {

  fn clock_cycle(&mut self) {
    self.cpu.clock_cycle();
    if self.current_cycle % 3 == 0 {
      let ppu_mutex = self.cpu.bus.get_PPU();
      let mut ppu_mutex_guard = ppu_mutex.lock().unwrap();
      ppu_mutex_guard.clock_cycle();
    }
    self.current_cycle += 1;
  }

}

#[derive(Debug, Clone)]
enum EmulatorMessage {
  ResumeEmulation,
  PauseEmulation,
  NextCPUInstruction,
  NextFrame,
  EventOccurred(iced_native::Event),
}

impl Application for RustNESs {
  type Message = EmulatorMessage;
  type Executor = executor::Default;

  type Theme = Theme;
  
  type Flags = ();

  fn new(flags: Self::Flags) -> (RustNESs, iced::Command<EmulatorMessage>) {
    let rom_file_path = "src/test_roms/nestest.nes";


    let mut cpu_bus = Bus16Bit::new(rom_file_path);

    cpu_bus.write(emulation::PROGRAM_START_POINTER_ADDR, 0x00).unwrap();
    cpu_bus.write(emulation::PROGRAM_START_POINTER_ADDR + 1, 0x80).unwrap();
    
    let cpu: Ben6502 = Ben6502::new(cpu_bus);
    return (Self { 
              cpu,
              current_cycle: 0,
              paused: true
            },
            Command::none()
    );
  }

  fn title(&self) -> String {
    return String::from("RustNESs NES Emulator of whimsy!");
  }

  fn update(&mut self, message: Self::Message) -> iced::Command<EmulatorMessage> {
    match message {
        EmulatorMessage::ResumeEmulation => {
          self.paused = false;
        },
        EmulatorMessage::PauseEmulation => {
          self.paused = true;
        },
        EmulatorMessage::NextCPUInstruction => {
          self.clock_cycle();
          while (self.cpu.current_instruction_remaining_cycles > 0){
            self.clock_cycle();
          }

          // TODO: verify that this is how the cycles should be executed
          self.clock_cycle();
          while (self.cpu.current_instruction_remaining_cycles == 0) {
            self.clock_cycle();
          }
          
        },
        EmulatorMessage::NextFrame => {
          self.clock_cycle();
          let ppu_mutex = self.cpu.bus.get_PPU();
          let ppu_mutex_guard = ppu_mutex.lock().unwrap();
          while (!ppu_mutex_guard.frame_render_complete){
            self.clock_cycle();
          }
          
        },
        EmulatorMessage::EventOccurred(event) => {
          if let Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::Space, modifiers }) = event {
              println!("Spacebar pressed!");
              self.update(EmulatorMessage::NextCPUInstruction);
          } else {
            // println!("Spacebar pressed!!");
          }
      }
    }

    Command::none()
    
  }

  fn view(&self) -> Element<'_, Self::Message> {
    let ram_start_addr = 0x8000;
    let ram_end_addr = 0x8010;

    let stack_start_addr = 0x100 + emulation::SP_RESET_ADDR as u16 - 100;
    let stack_end_addr = 0x100 + emulation::SP_RESET_ADDR as u16;


    column![
      // Contains screen visualizer and PPU buffer visualizers
      row![

      ],

      // Contains Memory visualizer and CPU+PPU status visualizers  
      row![

        // MemoryVisualizer
        column![
          text(format!("RAM contents (Addr 0x{:x} - 0x{:x}):", 0x00, 0x50)),
          text(self.cpu.bus.get_memory_content_as_string(0x00, 0x50)).size(20),
          text(format!("RAM contents (Addr 0x{:x} - 0x{:x}):", ram_start_addr, ram_end_addr)),
          text(self.cpu.bus.get_memory_content_as_string(ram_start_addr, ram_end_addr)).size(20),
          text(format!("Stack contents (Addr 0x{:x} - 0x{:x}):", stack_start_addr, stack_end_addr)),
          text(self.cpu.bus.get_memory_content_as_string(stack_start_addr, stack_end_addr)).size(20)
        ].max_width(400),

        // StatusVisualizer
        column![
          row![
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
          row![
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
        ]
      ]
    ]
    .padding(20)
    .align_items(Alignment::Center)
    .into()
  }

  fn subscription(&self) -> Subscription<EmulatorMessage> {
    iced_native::subscription::events().map(EmulatorMessage::EventOccurred)
  }
}