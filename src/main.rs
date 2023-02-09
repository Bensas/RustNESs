#![allow(unused_parens)]
mod emulation;
use std::cell::RefCell;
use std::env;
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
  env::set_var("RUST_BACKTRACE", "1");
  RustNESs::run(Settings::default());
}

const EMULATOR_CYCLES_PER_SECOND: u64 = 5;
const SCREEN_HEIGHT: u16 = 500;
const PATTERN_TABLE_VIS_HEIGHT: u16 = 300;
const PALETTE_VIS_HEIGHT: u16 = 30;
const PALETTE_VIS_WIDTH: u16 = 240;

struct RustNESs {
  cpu: Ben6502,
  current_cycle: u64,

  paused: bool,
  cycles_per_second: u64,

  input_handler: NESInputHandler,

  ppu_screen_buffer_visualizer: PPUScreenBufferVisualizer,
  ppu_pattern_tables_buffer_visualizer: PPUPatternTableBufferVisualizer,
  ppu_palette_visualizer: PPUPaletteVisualizer,

  mem_visualizer: MemoryVisualizer
}

impl RustNESs {

  fn clock_cycle(&mut self) {
    self.cpu.clock_cycle();
    if self.current_cycle % 3 == 0 {
      self.cpu.bus.PPU.borrow_mut().clock_cycle();
      if (self.cpu.bus.PPU.borrow().trigger_cpu_nmi) {
        // println!("PPU triggered CPU nmi!");
        self.cpu.bus.PPU.borrow_mut().trigger_cpu_nmi = false;
        self.cpu.nmi();
      }
    }
    self.current_cycle += 1;
  }

}

#[derive(Debug, Clone)]
enum EmulatorMessage {
  TogglePauseEmulation,
  NextCPUInstruction,
  NextFrame,
  Run50CPUInstructions,

  PatternTablePaletteCycle,
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
              input_handler: NESInputHandler::new(),
              ppu_screen_buffer_visualizer: PPUScreenBufferVisualizer {
                screen_vis_buffer: [[emulation::graphics::Color::new(0, 0, 0); 256]; 240],
                canvas_cache: Cache::default(),
                pixel_height: f32::from(SCREEN_HEIGHT) / 240.0
              },
              ppu_pattern_tables_buffer_visualizer: PPUPatternTableBufferVisualizer {
                pattern_tables_vis_buffer: [[[emulation::graphics::Color::new(0, 0, 0); 128]; 128]; 2],
                pattern_table_vis_palette_id: 0,
                canvas_cache: Cache::default(),
                pixel_height: f32::from(PATTERN_TABLE_VIS_HEIGHT) / 128.0
              },
              ppu_palette_visualizer: PPUPaletteVisualizer {
                palette: [emulation::graphics::Color::new(0, 0, 0); 32],
                canvas_cache: Cache::default(),
                pixel_height: f32::from(PALETTE_VIS_WIDTH) / 32.0
              },
              mem_visualizer: MemoryVisualizer {
                ram_start_addr: 0x00, //0xC0,
                ram_end_addr: 0x100,
                pc_start_addr:0x8000,
                pc_end_addr: 0x8010,
                stack_start_addr: 0x100 + emulation::Ben6502::SP_RESET_ADDR as u16 - 100,
                stack_end_addr: 0x100 + emulation::Ben6502::SP_RESET_ADDR as u16,

                ram_content_str: String::from(""),
                program_content_str: String::from(""),
                program_content: vec![],
                stack_content_str: String::from(""),
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
        EmulatorMessage::TogglePauseEmulation => {
          self.paused = !self.paused;
        },
        EmulatorMessage::NextCPUInstruction => {
          self.clock_cycle();
          while (self.cpu.current_instruction_remaining_cycles > 0){
            self.clock_cycle();
          }
        },

        EmulatorMessage::Run50CPUInstructions => {
          for i in 0..50 {
            self.clock_cycle();
            while (self.cpu.current_instruction_remaining_cycles > 0){
              self.clock_cycle();
            }
          }
        },
        EmulatorMessage::NextFrame => {
          let input_byte = self.input_handler.get_input_byte();
          self.cpu.bus.controller.borrow_mut().emulator_input[0] = input_byte;

          let start_render_time = Instant::now();

          self.clock_cycle();
          let mut frame_render_complete = self.cpu.bus.PPU.borrow().frame_render_complete;
          while (!frame_render_complete){
            self.clock_cycle();
            frame_render_complete = self.cpu.bus.PPU.borrow().frame_render_complete;
          }

          println!("Frame render took {}ms", start_render_time.elapsed().as_millis());
          self.cpu.bus.PPU.borrow_mut().frame_render_complete = false;
          self.cpu.bus.PPU.borrow_mut().update_pattern_tables_vis_buffer(self.ppu_pattern_tables_buffer_visualizer.pattern_table_vis_palette_id);

        },
        EmulatorMessage::PatternTablePaletteCycle => {
          self.ppu_pattern_tables_buffer_visualizer.pattern_table_vis_palette_id += 1;
          if self.ppu_pattern_tables_buffer_visualizer.pattern_table_vis_palette_id > 7 {
            self.ppu_pattern_tables_buffer_visualizer.pattern_table_vis_palette_id = 0;
          }
        },

        EmulatorMessage::EventOccurred(event) => {
          match event {
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::Space, modifiers }) => {
              // println!("Spacebar (For run 1 cpu instruction) pressed!");
              self.update(EmulatorMessage::NextCPUInstruction);
            },
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::Key5, modifiers }) => {
              // println!("Key5(For run 50 cpu instructions) pressed!");
              self.update(EmulatorMessage::Run50CPUInstructions);
            },
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::F, modifiers }) => {
              println!("F(For next Frame) pressed!");
              self.update(EmulatorMessage::NextFrame);
            },
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::P, modifiers }) => {
              println!("P(cycle palette color) pressed!");
              self.update(EmulatorMessage::PatternTablePaletteCycle);
            },
            Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::Enter, modifiers }) => {
              println!("Enter(play/pause emulation) pressed!");
              self.update(EmulatorMessage::TogglePauseEmulation);
            },
            _ => {
              self.input_handler.handle_keyboard_input(event);
            }
          }
      }
    }
    self.mem_visualizer.update(&mut self.cpu);

    self.cpu.bus.PPU.borrow_mut().update_pattern_tables_vis_buffer(self.ppu_pattern_tables_buffer_visualizer.pattern_table_vis_palette_id);
    self.ppu_screen_buffer_visualizer.update_data(&self.cpu.bus.PPU.borrow_mut());
    self.ppu_pattern_tables_buffer_visualizer.update_data(&self.cpu.bus.PPU.borrow_mut());
    self.ppu_palette_visualizer.update_data(&self.cpu.bus.PPU.borrow_mut());
    Command::none()
    
  }

  fn view(&self) -> Element<'_, Self::Message> {
    column![
      // Contains screen visualizer and PPU buffer visualizers
      row![

      self.ppu_screen_buffer_visualizer.view(),
      self.ppu_pattern_tables_buffer_visualizer.view(),
      self.ppu_palette_visualizer.view(),
      ],

      // Contains Memory visualizer and CPU+PPU status visualizers  
      row![


        // MemoryVisualizer
        self.mem_visualizer.view(),

        // StatusVisualizer
        column![
          row![
            text("Cpu registers:").size(20),
            text(format!(" A: 0x{:X}", self.cpu.registers.a)),
            text(format!(" X: 0x{:X}", self.cpu.registers.x)),
            text(format!(" Y: 0x{:X}", self.cpu.registers.y)),
            text(format!(" PC: 0x{:X}", self.cpu.registers.pc)),
            text(format!("SP: 0x{:X}", self.cpu.registers.sp)),
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

          row![
            text("PPU flags:").size(20),
            text("Vertical Blank: "),
            text(self.cpu.bus.PPU.borrow().status_reg.get_vertical_blank().to_string()),
          ],
        ]
      ]
    ]
    .padding(20)
    .align_items(Alignment::Center)
    .into()
  }

  fn subscription(&self) -> Subscription<EmulatorMessage> {
    let mut subs = vec![];
    subs.push(iced_native::subscription::events().map(EmulatorMessage::EventOccurred));
    if !self.paused {
      subs.push(iced::time::every(time::Duration::from_millis(1000 / self.cycles_per_second)).map(|em| {EmulatorMessage::NextFrame}));
    }
    return Subscription::batch(subs);
  }
}


struct MemoryVisualizer {
  ram_start_addr: u16,
  ram_end_addr: u16,
  pc_start_addr: u16,
  pc_end_addr: u16,
  stack_start_addr: u16,
  stack_end_addr: u16,

  ram_content_str: String,
  program_content_str: String,
  program_content: Vec<u8>,
  stack_content_str: String
}

impl MemoryVisualizer {
  fn update(&mut self, cpu: &mut Ben6502) {

    self.pc_start_addr = cpu.registers.pc;
    if ((cpu.registers.pc as u32 + 16) <= u16::MAX.into()) {
      self.pc_end_addr = cpu.registers.pc + 16;
    } else {
      self.pc_end_addr = self.pc_start_addr;
    }

    self.stack_start_addr = emulation::Ben6502::STACK_START_ADDR + cpu.registers.sp as u16 - 40;
    self.stack_end_addr = emulation::Ben6502::STACK_START_ADDR + cpu.registers.sp as u16 + 4;


    if ((self.pc_start_addr >= emulation::Ben2C02::PPU_MEMORY_BOUNDS.0 && self.pc_start_addr <= emulation::Ben2C02::PPU_MEMORY_BOUNDS.1) ||
        (self.pc_end_addr >= emulation::Ben2C02::PPU_MEMORY_BOUNDS.0 && self.pc_end_addr <= emulation::Ben2C02::PPU_MEMORY_BOUNDS.1) ||
        (self.stack_start_addr >= emulation::Ben2C02::PPU_MEMORY_BOUNDS.0 && self.stack_start_addr <= emulation::Ben2C02::PPU_MEMORY_BOUNDS.1) ||
        (self.stack_end_addr >= emulation::Ben2C02::PPU_MEMORY_BOUNDS.0 && self.stack_end_addr <= emulation::Ben2C02::PPU_MEMORY_BOUNDS.1) ||
        (self.ram_start_addr >= emulation::Ben2C02::PPU_MEMORY_BOUNDS.0 && self.ram_start_addr <= emulation::Ben2C02::PPU_MEMORY_BOUNDS.1) ||
        (self.ram_end_addr >= emulation::Ben2C02::PPU_MEMORY_BOUNDS.0 && self.ram_end_addr <= emulation::Ben2C02::PPU_MEMORY_BOUNDS.1)) {
          panic!("Memory visualizer is reading from PPU memory bounds, which might alter the state of the emulation!");
        }

    self.ram_content_str = cpu.bus.get_memory_content_as_string(self.ram_start_addr, self.ram_end_addr);
    self.program_content_str = cpu.bus.get_memory_content_as_string(self.pc_start_addr, self.pc_end_addr);
    self.program_content = cpu.bus.get_memory_content_as_vec(self.pc_start_addr, self.pc_end_addr);
    self.stack_content_str = cpu.bus.get_memory_content_as_string(self.stack_start_addr, self.stack_end_addr);    

  }

  fn view<'a>(&self) -> Element<'a, EmulatorMessage> {
  
    column![
      text(format!("RAM contents (Addr 0x{:x} - 0x{:x}):", self.ram_start_addr, self.ram_end_addr-1)),
      text(&self.ram_content_str).size(20),
      text(format!("RAM contents  at PC (Addr 0x{:x} - 0x{:x}):", self.pc_start_addr, self.pc_end_addr-1)),
      text(&self.program_content_str).size(20),
      text(emulation::Ben6502::disassemble(&self.program_content)).size(18).style(Color::from([0.0, 0.0, 1.0])),
      text(format!("Stack contents (Addr 0x{:x} - 0x{:x}):", self.stack_start_addr, self.stack_end_addr-1)),
      text(&self.stack_content_str).size(20)
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
              Point::new( (j as f32) * self.pixel_height as f32, (i as f32) * self.pixel_height as f32),
              Size::new(self.pixel_height, self.pixel_height),
              pixel_color.to_iced_color(),
          );
        }
      }
    });
    vec![pixel_grid]
  }
}

struct PPUPaletteVisualizer {
  palette: [emulation::graphics::Color; 32],
  canvas_cache: Cache,
  pixel_height: f32
}

impl PPUPaletteVisualizer {
  pub fn view(&self) -> Element<EmulatorMessage> {
    Canvas::new(self)
        .width(Length::Units(PALETTE_VIS_WIDTH))
        .height(Length::Units(PALETTE_VIS_HEIGHT))
        .into()
  }

  pub fn update_data(&mut self, ppu: &Ben2C02) {
    // Every time we update, I'm copying the contents of the PPU buffer
    // onto the buffer of the Visualizer. This is awful, but I can't 
    // figure out lifetimes well enough to directly reference the PPU buffer :/
    // TODO: Reference PPU buffer directly
    for i in 0..ppu.palette.len() {
      self.palette[i] = ppu.palette_vis_bufer[ppu.palette[i] as usize];
    }
    self.canvas_cache.clear();
  }
}


impl canvas::Program<EmulatorMessage> for PPUPaletteVisualizer {
  type State = ();

  fn draw(
      &self,
      _state: &Self::State,
      _theme: &Theme,
      bounds: Rectangle,
      cursor: Cursor,
  ) -> Vec<Geometry> {

    let pixel_grid = self.canvas_cache.draw(bounds.size(), |frame| {
      for i in 0..self.palette.len() {
        let pixel_color = self.palette[i];

        frame.fill_rectangle(
            Point::new((i as f32) * self.pixel_height as f32, 0.0),
            Size::new(self.pixel_height, self.pixel_height),
            pixel_color.to_iced_color(),
        );
      }
    });
    vec![pixel_grid]
  }
}



struct PPUPatternTableBufferVisualizer {
  pattern_tables_vis_buffer: [[[emulation::graphics::Color; 128]; 128]; 2],
  canvas_cache: Cache,
  pixel_height: f32,
  pattern_table_vis_palette_id: u8
}

impl PPUPatternTableBufferVisualizer {
  pub fn view(&self) -> Element<EmulatorMessage> {
    Canvas::new(self)
        .width(Length::Units(PATTERN_TABLE_VIS_HEIGHT * 2))
        .height(Length::Units(PATTERN_TABLE_VIS_HEIGHT))
        .into()
  }

  pub fn update_data(&mut self, ppu: &Ben2C02) {
    // Every time we update, I'm copying the contents of the PPU buffer
    // onto the buffer of the Visualizer. This is awful, but I can't 
    // figure out lifetimes well enough to directly reference the PPU buffer :/
    // TODO: Reference PPU buffer directly
    for tableIndex in 0..2 {
      for i in 0..ppu.pattern_tables_vis_buffer[0].len() {
        for j in 0..ppu.pattern_tables_vis_buffer[0][0].len() {
          self.pattern_tables_vis_buffer[tableIndex][i][j] = ppu.pattern_tables_vis_buffer[tableIndex][i][j];
        }
      }
    }
    self.canvas_cache.clear();
  }
}

impl canvas::Program<EmulatorMessage> for PPUPatternTableBufferVisualizer {
  type State = ();

  fn draw(
      &self,
      _state: &Self::State,
      _theme: &Theme,
      bounds: Rectangle,
      cursor: Cursor,
  ) -> Vec<Geometry> {

    let pixel_grid = self.canvas_cache.draw(bounds.size(), |frame| {
      for tableIndex in 0..2 {
        for i in 0..self.pattern_tables_vis_buffer[0].len() {
          for j in 0..self.pattern_tables_vis_buffer[0][0].len() {
            let pixel_color = self.pattern_tables_vis_buffer[tableIndex][i][j];
  
            frame.fill_rectangle(
                Point::new(
                          (tableIndex as f32) * self.pixel_height * (self.pattern_tables_vis_buffer[0].len() as f32)  + (i as f32) * self.pixel_height as f32,
                          (j as f32) * self.pixel_height as f32
                ),
                Size::new(self.pixel_height, self.pixel_height),
                pixel_color.to_iced_color(),
            );
          }
        }
      }
    });
    vec![pixel_grid]
  }
}

struct NESInputHandler {
  a_pressed: bool,
  b_pressed: bool,
  start_pressed: bool,
  select_pressed: bool,
  up_pressed: bool,
  down_pressed: bool,
  left_pressed: bool,
  right_pressed: bool,
}

impl NESInputHandler {
  fn new() -> Self {
    return NESInputHandler {
      a_pressed: false,
      b_pressed: false,
      start_pressed: false,
      select_pressed: false,
      up_pressed: false,
      down_pressed: false,
      left_pressed: false,
      right_pressed: false
    }
  }

  fn handle_keyboard_input(&mut self, event: Event) {
    match event {
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::W, modifiers }) => {
        self.up_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::W, modifiers }) => {
        self.up_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::A, modifiers }) => {
        self.left_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::A, modifiers }) => {
        self.left_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::S, modifiers }) => {
        self.down_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::S, modifiers }) => {
        self.down_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::D, modifiers }) => {
        self.right_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::D, modifiers }) => {
        self.right_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::M, modifiers }) => {
        self.b_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::M, modifiers }) => {
        self.b_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::N, modifiers }) => {
        self.a_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::N, modifiers }) => {
        self.a_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::J, modifiers }) => {
        self.start_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::J, modifiers }) => {
        self.start_pressed = false;
      },
      Event::Keyboard(keyboard::Event::KeyPressed { key_code: KeyCode::H, modifiers }) => {
        self.select_pressed = true;
      },
      Event::Keyboard(keyboard::Event::KeyReleased { key_code: KeyCode::H, modifiers }) => {
        self.select_pressed = false;
      },
      _ => {

      }
    }
  }

  fn get_input_byte(&self) -> u8 {
    let mut result = 0x0;
    if self.a_pressed {
      result |= 0b10000000;
    }
    if self.b_pressed {
      result |= 0b01000000;
    }
    if self.select_pressed {
      result |= 0b00100000;
    }
    if self.start_pressed {
      result |= 0b00010000;
    }
    if self.up_pressed {
      result |= 0b00001000;
    }
    if self.down_pressed {
      result |= 0b00000100;
    }
    if self.left_pressed {
      result |= 0b00000010;
    }
    if self.right_pressed {
      result |= 0b00000001;
    }
    return result;
  }
}