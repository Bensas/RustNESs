mod emulation;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Mutex, Arc, MutexGuard};

use emulation::{ Bus16Bit::Bus16Bit, Ben6502::Ben6502, hex_utils, Ben2C02::Ben2C02, Ram::Ram2K, Cartridge::Cartridge, Device::Device};


use iced::widget::{button, column, row, text};
use iced::{Alignment, Element, Sandbox, Settings, Renderer, event, Application, Subscription, executor, Theme, Command, Rectangle, time, Point, Size};

use iced::keyboard::{self, KeyCode, Modifiers};

use iced_native::{Event, Program};
use iced_native::Length;
use iced_native::Color;

use std::time::{Duration, Instant};

use iced::widget::canvas;
use iced::widget::canvas::{
  Cache, Canvas, Cursor, Frame, Geometry, Path, Text,
};


fn main() {
  RustNESs::run(Settings::default());
}

const EMULATOR_CYCLES_PER_SECOND: u64 = 10;
const SCREEN_PIXEL_HEIGHT: f32 = 10.0;
const SCREEN_HEIGHT: u16 = 300;

struct RustNESs {
  cpu: Ben6502,
  current_cycle: u64,

  paused: bool,
  cycles_per_second: u64,

  ppu_screen_buffer_visualizer: PPUScreenBufferVisualizer,
  mem_visualizer: MemoryVisualizer
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

    // cpu_bus.write(emulation::PROGRAM_START_POINTER_ADDR, 0x00).unwrap();
    // cpu_bus.write(emulation::PROGRAM_START_POINTER_ADDR + 1, 0x80).unwrap();
    
    let cpu: Ben6502 = Ben6502::new(cpu_bus);
    return (Self { 
              cpu,
              current_cycle: 0,
              paused: true,
              cycles_per_second: EMULATOR_CYCLES_PER_SECOND,
              ppu_screen_buffer_visualizer: PPUScreenBufferVisualizer {
                screen_vis_buffer: [[emulation::graphics::Color::new(0, 0, 0); 256]; 240],
                canvas_cache: Cache::default(),
                pixel_height: SCREEN_PIXEL_HEIGHT
              },
              mem_visualizer: MemoryVisualizer {
                ram_start_addr: 0x8000,
                ram_end_addr: 0x8010,
                stack_start_addr: 0x100 + emulation::Ben6502::SP_RESET_ADDR as u16 - 100,
                stack_end_addr: 0x100 + emulation::Ben6502::SP_RESET_ADDR as u16
              }
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
          
          // // TODO: verify that this is how the cycles should be executed
          // self.clock_cycle();
          // while (self.cpu.current_instruction_remaining_cycles == 0) {
          //   self.clock_cycle();
          // }
          
        },
        EmulatorMessage::NextFrame => {
          self.clock_cycle();
          let ppu_mutex = self.cpu.bus.get_PPU();
          let ppu_mutex_guard = ppu_mutex.lock().unwrap();
          let mut frame_render_complete = ppu_mutex_guard.frame_render_complete;
          drop(ppu_mutex_guard);
          drop(ppu_mutex);
          while (!frame_render_complete){
            self.clock_cycle();
            let ppu_mutex = self.cpu.bus.get_PPU();
            let ppu_mutex_guard = ppu_mutex.lock().unwrap();
            frame_render_complete = ppu_mutex_guard.frame_render_complete;
            drop(ppu_mutex_guard);
            drop(ppu_mutex);
          }
          let ppu_mutex = self.cpu.bus.get_PPU();
          let mut ppu_mutex_guard = ppu_mutex.lock().unwrap();
          ppu_mutex_guard.frame_render_complete = false;
          drop(ppu_mutex_guard);
          drop(ppu_mutex);
        },
        EmulatorMessage::EventOccurred(event) => {
          match event {
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::Space, modifiers }) => {
              println!("Spacebar pressed!");
              self.update(EmulatorMessage::NextCPUInstruction);
            },
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::F, modifiers }) => {
              println!("F(For next Frame) pressed!");
              self.update(EmulatorMessage::NextFrame);
            },
            _ => {

            }
          }
      }
    }
    self.mem_visualizer.update(&self.cpu);
    self.ppu_screen_buffer_visualizer.update_data(&self.cpu.bus.PPU.lock().unwrap());
    Command::none()
    
  }

  fn view(&self) -> Element<'_, Self::Message> {
    column![
      // Contains screen visualizer and PPU buffer visualizers
      row![

      self.ppu_screen_buffer_visualizer.view(),

      ],

      // Contains Memory visualizer and CPU+PPU status visualizers  
      row![

        // MemoryVisualizer
        self.mem_visualizer.view(&self.cpu.bus),

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
    // if !self.paused {
    //   return iced::time::every(time::Duration::from_millis(1000 / self.cycles_per_second)).map(EmulatorMessage::Tick);
    // }
  }
}


struct MemoryVisualizer {
  ram_start_addr: u16,
  ram_end_addr: u16,
  stack_start_addr: u16,
  stack_end_addr: u16,
}

impl MemoryVisualizer {
  fn update(&mut self, cpu: &Ben6502) {
    self.ram_start_addr = cpu.registers.pc;
    self.ram_end_addr = cpu.registers.pc + 16;

    self.stack_start_addr = emulation::Ben6502::STACK_START_ADDR + cpu.registers.sp as u16 - 40;
    self.stack_end_addr = emulation::Ben6502::STACK_START_ADDR + cpu.registers.sp as u16 + 1;
  }

  fn view<'a>(&self, cpu_bus: &Bus16Bit) -> Element<'a, EmulatorMessage> {
  
    column![
      text(format!("RAM contents (Addr 0x{:x} - 0x{:x}):", 0x00, 0x50)),
      text(cpu_bus.get_memory_content_as_string(0x00, 0x50)).size(20),
      text(format!("RAM contents  at PC (Addr 0x{:x} - 0x{:x}):", self.ram_start_addr, self.ram_end_addr)),
      text(cpu_bus.get_memory_content_as_string(self.ram_start_addr, self.ram_end_addr)).size(20),
      text(emulation::Ben6502::disassemble(cpu_bus.get_memory_content_as_vec(self.ram_start_addr, self.ram_end_addr))).size(18).style(Color::from([0.0, 0.0, 1.0])),
      text(format!("Stack contents (Addr 0x{:x} - 0x{:x}):", self.stack_start_addr, self.stack_end_addr)),
      text(cpu_bus.get_memory_content_as_string(self.stack_start_addr, self.stack_end_addr)).size(20)
    ]
    .max_width(500)
    .into()
  }
}


struct PPUScreenBufferVisualizer {
  screen_vis_buffer: [[emulation::graphics::Color; 256]; 240],
  canvas_cache: Cache,
  pixel_height: f32
}

impl PPUScreenBufferVisualizer {
  pub fn view(&self) -> Element<EmulatorMessage> {
    Canvas::new(self)
        .width(Length::Units(SCREEN_HEIGHT))
        .height(Length::Units(SCREEN_HEIGHT))
        .into()
  }

  pub fn update_data(&mut self, ppu: &Ben2C02) {
    // Every time we update, I'm copying the contents of the PPU buffer
    // onto the buffer of the Screen Visualizer. This is awful, but I can't 
    // figure out lifetimes well enough to directly reference the PPU buffer :/
    // TODO: Reference PPU buffer directly
    for i in 0..ppu.screen_vis_buffer.len() {
      for j in 0..ppu.screen_vis_buffer[0].len() {
        self.screen_vis_buffer[i][j] = ppu.screen_vis_buffer[i][j];
      }
    }
    self.canvas_cache.clear();
  }
}


#[derive(Debug, Clone)]
    pub enum Message {
        Hello(u8)
    }

impl canvas::Program<EmulatorMessage> for PPUScreenBufferVisualizer {
  type State = ();

  fn draw(
      &self,
      _state: &Self::State,
      _theme: &Theme,
      bounds: Rectangle,
      cursor: Cursor,
  ) -> Vec<Geometry> {

    let pixel_grid = self.canvas_cache.draw(bounds.size(), |frame| {
      for i in 0..self.screen_vis_buffer.len() {
        for j in 0..self.screen_vis_buffer[0].len() {
          let pixel_color = self.screen_vis_buffer[i][j];

          frame.fill_rectangle(
              Point::new((i as f32) * self.pixel_height as f32, (j as f32) * self.pixel_height as f32),
              Size::new(self.pixel_height, self.pixel_height),
              pixel_color.to_iced_color(),
          );
        }
      }
    });
    vec![pixel_grid]
  }
}