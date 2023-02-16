use std::{sync::{Arc, Mutex}, cell::RefCell, rc::Rc};

  use crate::{graphics::Color, device::Device, utils::bitwise_utils, cartridge::{Cartridge, MirroringMode}};
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

  #[derive(Clone, Copy)]
  pub struct VramRegister {
    flags: u16
  }
  
  impl VramRegister {
  
    fn new() -> VramRegister {
      return VramRegister {
        flags: 0b00000000 
      }
    }

    pub fn get_coarse_x(&self) -> u8 {
      return (bitwise_utils::get_bits_16(self.flags, 0, 4) & 0xFF) as u8;
    }
  
    fn set_coarse_x(&mut self, value: u8) {
      if (value > 0b11111) {
        panic!("VramRegister::set_coarse_x() was given a value larger than 5 bits!");
      };
      bitwise_utils::set_bits_16(&mut self.flags, 0, 4, value as u16);
    }
  
    pub fn get_coarse_y(&self) -> u8 {
      return (bitwise_utils::get_bits_16(self.flags, 5, 9) & 0xFF) as u8;
    }
  
    fn set_coarse_y(&mut self, value: u8) {
      if (value > 0b11111) {
        panic!("VramRegister::set_coarse_y() was given a value larger than 5 bits!");
      };
      bitwise_utils::set_bits_16(&mut self.flags, 5, 9, value as u16);
    }

    pub fn get_nametable_x(&self) -> u8 {
      return bitwise_utils::get_bit_16(self.flags, 10);
    }
  
    fn set_nametable_x(&mut self, value: u8) {
      bitwise_utils::set_bit_16(&mut self.flags, 10, value);
    }
  
    pub fn get_nametable_y(&self) -> u8 {
      return bitwise_utils::get_bit_16(self.flags, 11);
    }
  
    fn set_nametable_y(&mut self, value: u8) {
      bitwise_utils::set_bit_16(&mut self.flags, 11, value);
    }

    pub fn get_fine_y(&self) -> u8 {
      return (bitwise_utils::get_bits_16(self.flags, 12, 14) & 0xFF) as u8;
    }
  
    fn set_fine_y(&mut self, value: u8) {
      if (value > 0b111) {
        panic!("VramRegister::set_fine_y() was given a value larger than 3 bits!");
      };
      bitwise_utils::set_bits_16(&mut self.flags, 12, 14, value as u16);
    }
  }

#[derive(Default, Clone, Copy, Debug)]
  pub struct SpriteObj {
    y: u8,
    tile_id: u8,
    attributes: u8,
    x: u8
  }

  pub struct Ben2C02 {
    memory_bounds: (u16, u16),

    cartridge: Rc<RefCell<Cartridge>>,

    scan_line: i16,
    cycle: i16,
    pub frame_render_complete: bool,
    odd_frame: bool,
    pub trigger_cpu_nmi: bool,

    controller_reg: ControllerRegister,
    mask_reg: MaskRegister,
    pub status_reg: StatusRegister,
    writing_high_byte_of_addr: bool,
    ppu_data_read_buffer: u8,
    oam_data_addr: u8,

    vram_reg: VramRegister,
    temp_vram_reg: VramRegister,
    fine_x: u8,

    // Scroll-related variables
    bg_next_tile_id: u8,
    bg_next_tile_attribute: u8,
    bg_next_tile_lsb: u8,
    bg_next_tile_msb: u8,

    // Shift registers
    bg_shifter_pattern_lo: u16,
		bg_shifter_pattern_hi: u16,
		bg_shifter_attrib_lo: u16,
		bg_shifter_attrib_hi: u16,


    // Sprite rendering variables
    sprites_on_curr_scanline: Vec<SpriteObj>,
    sprites_on_curr_scanline_pattern_lsb: Vec<u8>,
    sprites_on_curr_scanline_pattern_msb: Vec<u8>,

    sprite_zero_hit_possible: bool,
    sprite_zero_being_rendered: bool,

    pattern_tables: [[u8; 4096]; 2],
    pattern_tables_mem_bounds: (u16, u16),
    name_tables: [[u8; 1024]; 2],
    name_tables_mem_bounds: (u16, u16),
    pub palette: [u8; 32],
    palette_mem_bounds: (u16, u16),
    pub oam_memory: [SpriteObj; 64],

    
    // These arrays are used for emulator visualization, thus the higher level Color structure
    pub screen_vis_buffer: [[Color; 256]; 240],
    pub pattern_tables_vis_buffer: [[[Color; 128]; 128]; 2],
    name_tables_vis_buffer: [[[Color; 256]; 240]; 2],
    pub palette_vis_bufer: [Color; 64],
  }

  impl Ben2C02 {
    pub fn new(cartridge: Rc<RefCell<Cartridge>>) -> Ben2C02 {
      return Ben2C02 {
        memory_bounds: PPU_MEMORY_BOUNDS,
        cartridge: cartridge,
        
        scan_line: 0,
        cycle: 0,
        frame_render_complete: false,
        odd_frame: false,
        trigger_cpu_nmi: false,

        controller_reg: ControllerRegister::new(),
        mask_reg: MaskRegister::new(),
        status_reg: StatusRegister::new(),
        writing_high_byte_of_addr: true,
        ppu_data_read_buffer: 0,
        oam_data_addr: 0,

        vram_reg: VramRegister::new(),
        temp_vram_reg: VramRegister::new(),
        fine_x: 0,

        bg_next_tile_id: 0,
			  bg_next_tile_attribute: 0,
			  bg_next_tile_lsb: 0,
			  bg_next_tile_msb: 0,

        bg_shifter_pattern_lo: 0,
        bg_shifter_pattern_hi: 0,
        bg_shifter_attrib_lo: 0,
        bg_shifter_attrib_hi: 0,

        sprites_on_curr_scanline: vec![],
        sprites_on_curr_scanline_pattern_lsb: vec![],
        sprites_on_curr_scanline_pattern_msb: vec![],

        sprite_zero_hit_possible: false,
        sprite_zero_being_rendered: false,

        pattern_tables: [[0; 4096]; 2],
        pattern_tables_mem_bounds: (0x0000, 0x1FFF),
        name_tables: [[0; 1024]; 2],
        name_tables_mem_bounds: (0x2000, 0x3EFF),
        palette: [0; 32],
        palette_mem_bounds: (0x3F00, 0x3FFF),
        oam_memory: [SpriteObj::default(); 64],


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

      // This cycle stravaganza is very concisely explained here: https://www.nesdev.org/w/images/default/4/4f/Ppu.svg
      if (self.scan_line >= -1 && self.scan_line < 240) {

        if (self.scan_line == 0 && self.cycle == 0 && self.odd_frame && (self.mask_reg.get_render_background() != 0 || self.mask_reg.get_render_sprites() != 0)) {
          // "Odd Frame" cycle skip
          self.cycle = 1;
        }

        if (self.scan_line == -1 && self.cycle == 1) {
          self.status_reg.set_vertical_blank(0);
          self.status_reg.set_sprite_overflow(0);
          self.status_reg.set_sprite_zero_hit(0);
          self.sprites_on_curr_scanline_pattern_lsb = vec![];
          self.sprites_on_curr_scanline_pattern_msb = vec![];
        }

        if ((self.cycle >= 2 && self.cycle < 258) || (self.cycle >= 321 && self.cycle < 338)) {
          if (self.mask_reg.get_render_background() != 0) {
            self.update_background_shift_registers();
          }
          if (self.mask_reg.get_render_sprites() != 0 && self.cycle >= 1 && self.cycle < 258) {
            self.update_foreground_shift_registers();
          }
          match ((self.cycle - 1) % 8) {
            0 => {
              self.load_background_shift_registers_with_next_tile();
              self.bg_next_tile_id = self.read_from_ppu_bus(0x2000 | (self.vram_reg.flags & 0xFFF)).unwrap();
            },
            1 => {

            },
            2 => {
              self.bg_next_tile_attribute = self.read_from_ppu_bus(
                                                  0x23C0 |
                                                  ((self.vram_reg.get_nametable_y() as u16) << 11) |
                                                  ((self.vram_reg.get_nametable_x() as u16) << 10) |
                                                  (((self.vram_reg.get_coarse_y() as u16) >> 2) << 3) |
                                                  ((self.vram_reg.get_coarse_x() as u16) >> 2)).unwrap();
              if ((self.vram_reg.get_coarse_y() & 0x02) != 0) {
                self.bg_next_tile_attribute >>= 4;
              }
              if ((self.vram_reg.get_coarse_x() & 0x02) != 0) {
                self.bg_next_tile_attribute >>= 2;
              }
              self.bg_next_tile_attribute &= 0x03;
            },
            3 => {

            },
            4 => {
              self.bg_next_tile_lsb = self.read_from_ppu_bus(
                                            ((self.controller_reg.get_pattern_background() as u16) << 12) +
                                                  ((self.bg_next_tile_id as u16) * 16) +
                                                  (self.vram_reg.get_fine_y() as u16)).unwrap();
            },
            5 => {

            },
            6 => {
              self.bg_next_tile_msb = self.read_from_ppu_bus(
                                            ((self.controller_reg.get_pattern_background() as u16) << 12) +
                                                  ((self.bg_next_tile_id as u16) * 16) +
                                                  (self.vram_reg.get_fine_y() as u16) + 8).unwrap();
            },
            7 => {
              if self.mask_reg.get_render_background() != 0 || self.mask_reg.get_render_sprites() != 0 {
                self.increment_scroll_x();
              }
            },
            _ => {}
          }
        }

        if (self.cycle == 256) {
          if self.mask_reg.get_render_background() != 0 || self.mask_reg.get_render_sprites() != 0 {
            self.increment_scroll_y();
          }
        }

        if (self.cycle == 257) {
          self.load_background_shift_registers_with_next_tile();
          if self.mask_reg.get_render_background() != 0 || self.mask_reg.get_render_sprites() != 0 {
            self.transfer_temp_vram_x();
          }
        }

        if (self.scan_line == -1 && self.cycle >= 280 && self.cycle < 305) {
          if self.mask_reg.get_render_background() != 0 || self.mask_reg.get_render_sprites() != 0 {
            self.transfer_temp_vram_y();
          }
        }

        if (self.scan_line >= 0 && self.cycle == 257) { // End of the visible scanline

          // We check which sprites in the OAM memory should be rendered in the current scanline (up to 8)
          // And add them to the sprites_on_curr_scanline vector
          self.sprites_on_curr_scanline = vec![];
          self.sprites_on_curr_scanline_pattern_lsb = vec![];
          self.sprites_on_curr_scanline_pattern_msb = vec![];

          self.sprite_zero_hit_possible = false;

          for i in 0..self.oam_memory.len() {
            let sprite = self.oam_memory.get(i).unwrap();
            let y_pos_diff = self.scan_line - sprite.y as i16;
            let sprite_size = if (self.controller_reg.get_sprite_size() != 0) { 16 } else { 8 };
            if (y_pos_diff >= 0 && y_pos_diff < sprite_size) {
              if (i == 0) {
                self.sprite_zero_hit_possible = true;
              }
              if (self.sprites_on_curr_scanline.len() < 8) {
                self.sprites_on_curr_scanline.push(sprite.clone());
              }
            }
          }
          if self.sprites_on_curr_scanline.len() >= 8 {
            self.status_reg.set_sprite_overflow(1);
          }
        }
        
        if (self.cycle == 340) {
          // For each of the sprites in the render list for this scanline, we calculate the address of its tile row
          // that corresponds to the current scanline, and then fetch the information for that row, flipping it if necessary.
          for i in 0..self.sprites_on_curr_scanline.len() {
            let sprite = self.sprites_on_curr_scanline.get(i).unwrap();
            let y_pos_diff = self.scan_line - sprite.y as i16;
            let sprite_color_value_lsb_addr: u16;
            let sprite_color_value_msb_addr: u16;
            if (self.controller_reg.get_sprite_size() == 0) { // Sprites are 8x8
              let start_addr = self.pattern_tables_mem_bounds.0 + if (self.controller_reg.get_pattern_sprite() != 0) { 4096 } else { 0 };
              if ((sprite.attributes & 0x80) != 0) { // Sprite is flipped vertically
                sprite_color_value_lsb_addr = start_addr + (sprite.tile_id as u16) * 16 + (7 - y_pos_diff) as u16;
              } else {
                sprite_color_value_lsb_addr = start_addr + (sprite.tile_id as u16) * 16 + y_pos_diff as u16;
              }
            } else { // Sprites are 8x16
              let start_addr = self.pattern_tables_mem_bounds.0 + ((sprite.tile_id & 0x01) as u16) * 4096;
              if ((sprite.attributes & 0x80) != 0) { // Sprite is flipped vertically
                if ( y_pos_diff < 8 ) { // We're rendering the top half of the rendered sprite (which is the bottom half of the original sprite)
                  sprite_color_value_lsb_addr = start_addr + (((sprite.tile_id & 0b11111110) + 1) as u16) * 16 + (7 - (y_pos_diff % 8)) as u16;
                } else {
                  sprite_color_value_lsb_addr = start_addr + (( sprite.tile_id & 0b11111110)       as u16) * 16 + (7 - (y_pos_diff % 8)) as u16;
                }
              } else {
                if ( y_pos_diff < 8 ) { // We're rendering the top half of the sprite
                  sprite_color_value_lsb_addr = start_addr + (( sprite.tile_id & 0b11111110)       as u16) * 16 + (y_pos_diff % 8) as u16;
                } else {
                  sprite_color_value_lsb_addr = start_addr + (((sprite.tile_id & 0b11111110) + 1) as u16) * 16 + (y_pos_diff % 8) as u16;
                }
              }
            }
            sprite_color_value_msb_addr = sprite_color_value_lsb_addr + 8;

            let mut sprite_color_value_lsb = self.read_from_ppu_bus(sprite_color_value_lsb_addr).unwrap();
            let mut sprite_color_value_msb = self.read_from_ppu_bus(sprite_color_value_msb_addr).unwrap();

            if ((sprite.attributes & 0x40) != 0) { // Sprite is flipped horizontally
              sprite_color_value_lsb = sprite_color_value_lsb.reverse_bits();
              sprite_color_value_msb = sprite_color_value_msb.reverse_bits();
            }
            self.sprites_on_curr_scanline_pattern_lsb.push(sprite_color_value_lsb);
            self.sprites_on_curr_scanline_pattern_msb.push(sprite_color_value_msb);
          }
        }
        
      }

      if (self.scan_line == 240) {
        
      }

      if (self.scan_line == 241 && self.cycle == 1) {
        self.status_reg.set_vertical_blank(1);
        if (self.controller_reg.get_enable_nmi() ==  1) {
          self.trigger_cpu_nmi = true;
        }
      }

      let mut bg_pixel_value: u8 = 0;
      let mut bg_palette_id: u8 = 0;

      if (self.mask_reg.get_render_background() != 0) {
        let bit_mux: u16 = 0x8000 >> self.fine_x;
        
        let bg_pixel0 = ((self.bg_shifter_pattern_lo & bit_mux) > 0) as u8;
        let bg_pixel1 = ((self.bg_shifter_pattern_hi & bit_mux) > 0) as u8;
        bg_pixel_value = bg_pixel1 << 1 | bg_pixel0;

        let bg_palette0 = ((self.bg_shifter_attrib_lo & bit_mux) > 0) as u8;
        let bg_palette1 = ((self.bg_shifter_attrib_hi & bit_mux) > 0) as u8;
        bg_palette_id = bg_palette1 << 1 | bg_palette0;
      }

      let mut fg_pixel_value: u8 = 0x0;
      let mut fg_palette_id: u8 = 0x0;
      let mut fg_priority: bool = false;

      if (self.mask_reg.get_render_sprites() != 0) {

        if ( (self.mask_reg.get_render_sprites_left() != 0) || (self.cycle >= 9)) {
          self.sprite_zero_being_rendered = false;
          for i in 0..self.sprites_on_curr_scanline.len() {
            let sprite_obj = self.sprites_on_curr_scanline.get(i).unwrap();
            if self.cycle >= (sprite_obj.x as i16) && self.cycle < (sprite_obj.x as i16 + 8) {
              let fg_pixel_lo = (self.sprites_on_curr_scanline_pattern_lsb.get(i).unwrap_or(&0) & 0b10000000 != 0) as u8;
              let fg_pixel_hi = (self.sprites_on_curr_scanline_pattern_msb.get(i).unwrap_or(&0) & 0b10000000 != 0) as u8;
              fg_pixel_value = (fg_pixel_hi << 1) | fg_pixel_lo;
  
              fg_palette_id = (sprite_obj.attributes & 0b11) + 0x04;
              fg_priority = (sprite_obj.attributes & 0b00100000) == 0;
  
              if (fg_pixel_value != 0) {
                if (i == 0) {
                  self.sprite_zero_being_rendered = true;
                }
                break;
              }
  
            }
          }
        }
      }

      let mut result_pixel_value: u8 = 0x00;
      let mut result_palette_id: u8 = 0x00;

      if (bg_pixel_value == 0 && fg_pixel_value == 0) {
        result_pixel_value = 0;
        result_palette_id = 0;
      } else if (bg_pixel_value == 0 && fg_pixel_value != 0) {
        result_pixel_value = fg_pixel_value;
        result_palette_id = fg_palette_id;
      } else if (bg_pixel_value != 0 && fg_pixel_value == 0) {
        result_pixel_value = bg_pixel_value;
        result_palette_id = bg_palette_id;
      } else if (bg_pixel_value != 0 && fg_pixel_value != 0) {
        if (fg_priority) {
          result_pixel_value = fg_pixel_value;
          result_palette_id = fg_palette_id;
        } else {
          result_pixel_value = bg_pixel_value;
          result_palette_id = bg_palette_id;
        }

        if (self.sprite_zero_being_rendered
            && self.sprite_zero_hit_possible
            && self.mask_reg.get_render_background() != 0
            && self.mask_reg.get_render_sprites() != 0   ) {

              if (self.mask_reg.get_render_background_left() == 0
                  && self.mask_reg.get_render_sprites_left() == 0) {

                  if (self.cycle >= 9 && self.cycle < 258) {
                    self.status_reg.set_sprite_zero_hit(1);
                  }

              } else if (self.cycle >= 1 && self.cycle < 258){
                self.status_reg.set_sprite_zero_hit(1);
              }
        }
      }

      if (self.cycle < 256 && self.scan_line < 240 && self.scan_line != -1) {
        self.screen_vis_buffer[self.scan_line as usize][self.cycle as usize] = self.get_color_from_palette(result_pixel_value, result_palette_id);
      }

      self.cycle += 1;
      if self.cycle > 340 {
        self.cycle = 0;
        self.scan_line += 1;
        if (self.scan_line > 260) {
          self.scan_line = -1;
          self.frame_render_complete = true;
          self.odd_frame = !self.odd_frame;
        }
      }

    }

    fn increment_scroll_x(&mut self) {
      if (self.vram_reg.get_coarse_x() == 31) {
        self.vram_reg.set_nametable_x((self.vram_reg.get_nametable_x() == 0) as u8);
        self.vram_reg.set_coarse_x(0);
      } else {
        self.vram_reg.set_coarse_x(self.vram_reg.get_coarse_x() + 1);
      }
    }

    fn increment_scroll_y(&mut self) {
			if (self.vram_reg.get_fine_y() < 7)
			{
				self.vram_reg.set_fine_y(self.vram_reg.get_fine_y() + 1);
			}
			else
			{
        self.vram_reg.set_fine_y(0);

				if (self.vram_reg.get_coarse_y() == 29) {
          self.vram_reg.set_nametable_y((self.vram_reg.get_nametable_y() == 0) as u8);
          self.vram_reg.set_coarse_y(0)
        }
				else if (self.vram_reg.get_coarse_y() == 31)
				{
					self.vram_reg.set_coarse_y(0)
				}
				else
				{
					self.vram_reg.set_coarse_y(self.vram_reg.get_coarse_y() + 1);
				}
			}
    }

    fn transfer_temp_vram_x(&mut self) {
      self.vram_reg.set_nametable_x(self.temp_vram_reg.get_nametable_x());
      self.vram_reg.set_coarse_x(self.temp_vram_reg.get_coarse_x());
    }

    fn transfer_temp_vram_y(&mut self) {
      self.vram_reg.set_nametable_y(self.temp_vram_reg.get_nametable_y());
      self.vram_reg.set_coarse_y(self.temp_vram_reg.get_coarse_y());
      self.vram_reg.set_fine_y(self.temp_vram_reg.get_fine_y());
    }

    fn load_background_shift_registers_with_next_tile(&mut self) {
      self.bg_shifter_pattern_lo = (self.bg_shifter_pattern_lo & 0xFF00) | (self.bg_next_tile_lsb as u16);
			self.bg_shifter_pattern_hi = (self.bg_shifter_pattern_hi & 0xFF00) | (self.bg_next_tile_msb as u16);
      if (self.bg_next_tile_attribute & 0b01) != 0 {
        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | 0xFF;
      } else {
        self.bg_shifter_attrib_lo = (self.bg_shifter_attrib_lo & 0xFF00) | 0x00;
      }
      
      if (self.bg_next_tile_attribute & 0b10) != 0 {
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | 0xFF;
      } else {
        self.bg_shifter_attrib_hi = (self.bg_shifter_attrib_hi & 0xFF00) | 0x00;
      }
    }

    fn update_background_shift_registers(&mut self) {
      self.bg_shifter_pattern_lo <<= 1;
			self.bg_shifter_pattern_hi <<= 1;
			self.bg_shifter_attrib_lo <<= 1;
			self.bg_shifter_attrib_hi <<= 1;
    }

    fn update_foreground_shift_registers(&mut self) {
      for i in 0..self.sprites_on_curr_scanline_pattern_lsb.len() {
        let sprite = self.sprites_on_curr_scanline[i];
        if (self.cycle - 1 >= (sprite.x as i16) && self.cycle - 1 < (sprite.x as i16 + 8)) {
          self.sprites_on_curr_scanline_pattern_lsb[i] <<= 1;
          self.sprites_on_curr_scanline_pattern_msb[i] <<= 1;
        }
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

    fn address_to_palette_index(&self, addr: u16) -> usize {
      
      //The entire palette (3F00-31F) is mirrored in the range (3F00-3FFF)
      let result = (addr & 0xFF) % 32;

      // Additionally, The following address mirrorings occur within the palette itself:
      // - 3F10 -> 3F00
      // - 3F14 -> 3F04
      // - 3F18 -> 3F08
      // - 3F1C -> 3F0C
      match result {
        0x10 => {
          0x00
        },
        0x14 => {
          0x04
        },
        0x18 => {
          0x08
        },
        0x1C => {
          0x0C
        }
        _ => {
          result as usize
        }
      }

    }

    // Useful: https://www.nesdev.org/wiki/PPU_memory_map
    fn write_to_ppu_memory(&mut self, addr: u16, data: u8) -> Result<(), String>{
      if self.in_pattern_table_memory_bounds(addr) {
		    self.pattern_tables[((addr & 0x1000) > 0) as usize][(addr & 0x0FFF) as usize] = data;
        return Ok(());
      }
      else if self.in_name_table_memory_bounds(addr) {
        let mirroring_mode = self.cartridge.borrow_mut().mirroring_mode;

        if addr <= 0x23FF {
          self.name_tables[0][(addr & 0x3FF) as usize] = data;
        } else if addr <= 0x27FF {
          if (matches!(mirroring_mode, MirroringMode::Horizontal)) {
            self.name_tables[0][(addr & 0x3FF) as usize] = data;
          } else if (matches!(mirroring_mode, MirroringMode::Vertical)) {
            self.name_tables[1][(addr & 0x3FF) as usize] = data;
          } else {
            todo!("Mirroring mode {:?} not implemented!", mirroring_mode);
          }
        } else if addr <= 0x2BFF {
          if (matches!(mirroring_mode, MirroringMode::Horizontal)) {
            self.name_tables[1][(addr & 0x3FF) as usize] = data;
          } else if (matches!(mirroring_mode, MirroringMode::Vertical)) {
            self.name_tables[0][(addr & 0x3FF) as usize] = data;
          } else {
            todo!("Mirroring mode {:?} not implemented!", mirroring_mode);
          }
        } else if addr <= 0x2FFF {
          self.name_tables[1][(addr & 0x3FF) as usize] = data;
        } else {
          // Addresses 3000-3EFF mirror addresses 2000-2EFF
          return self.write_to_ppu_memory(addr - 0x1000, data);
        }
        return Ok(());
      }
      else if self.in_palette_memory_bounds(addr) {
        // Address space is $3F00-$3F1F, mirrored in the range $3F00-$3FFF
        self.palette[self.address_to_palette_index(addr)] = data;
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
        let mirroring_mode = self.cartridge.borrow().mirroring_mode;
        if addr <= 0x23FF {
          return Ok(self.name_tables[0][(addr & 0x3FF) as usize]);
        } else if addr <= 0x27FF {
          if (matches!(mirroring_mode, MirroringMode::Horizontal)) {
            return Ok(self.name_tables[0][(addr & 0x3FF) as usize]);
          } else if (matches!(mirroring_mode, MirroringMode::Vertical)) {
            return Ok(self.name_tables[1][(addr & 0x3FF) as usize]);
          } else {
            todo!("Mirroring mode {:?} not implemented!", mirroring_mode);
          }
        } else if addr <= 0x2BFF {
          if (matches!(mirroring_mode, MirroringMode::Horizontal)) {
            return Ok(self.name_tables[1][(addr & 0x3FF) as usize]);
          } else if (matches!(mirroring_mode, MirroringMode::Vertical)) {
            return Ok(self.name_tables[0][(addr & 0x3FF) as usize]);
          } else {
            todo!("Mirroring mode {:?} not implemented!", mirroring_mode);
          }
        } else if addr <= 0x2FFF {
          return Ok(self.name_tables[1][(addr & 0x3FF) as usize]);
        } else {
          // Addresses 3000-3EFF mirror addresses 2000-2EFF
          return self.read_from_ppu_memory(addr - 0x1000);
        }
      }
      else if self.in_palette_memory_bounds(addr) {
        let data = self.palette[self.address_to_palette_index(addr)];
        return Ok(data);
      }
      else {
        return  Err(format!("Tried reading from PPU memory, but provided address wasn't within pattern_table,
                  name_table, or palette memory bounds!. Provided address was 0x{:X}", addr));
      }
    }

    fn read_from_ppu_bus(&self, addr: u16) -> Result<u8, String> {
      let read_from_cartridge = self.cartridge.borrow_mut().read(addr);
      match read_from_cartridge {
        Ok(retrieved_data) => {
          return Ok(retrieved_data);
        },
        Err(message) => {
          // println!("Tried to read from cartridge, but failed with error: {}. Reading from PPU internal memory instead :)" , message);
          return Ok(self.read_from_ppu_memory(addr).unwrap());
        }
      }
    }

    fn write_to_ppu_bus(&mut self, addr: u16, data: u8) -> Result<(), String> {
      let write_to_cartridge = self.cartridge.borrow_mut().write(addr, data);
      match write_to_cartridge {
        Ok(()) => {
          return Ok(());
        },
        Err(message) => {
          // println!("Tried to write to cartridge, but failed with error: {}. Writing to PPU internal memory instead :)" , message);
          return Ok(self.write_to_ppu_memory(addr, data).unwrap());
        }
      }
    }

    fn read_from_oam_memory(&self, addr: u8) -> u8 {
      let index = (addr / 4) as usize;
      match (addr % 4) {
        0 => {
          return self.oam_memory[index].y;
        },
        1 => {
          return self.oam_memory[index].tile_id;
        },
        2 => {
          return self.oam_memory[index].attributes;
        },
        3 => {
          return self.oam_memory[index].x;
        },
        _  => {
          return 0;
        }
      }
    }

    pub fn write_to_oam_memory(&mut self, addr: u8, data: u8) -> () {
      let index = (addr / 4) as usize;
      match (addr % 4) {
        0 => {
          self.oam_memory[index].y = data;
        },
        1 => {
          self.oam_memory[index].tile_id = data;
        },
        2 => {
          self.oam_memory[index].attributes = data;
        },
        3 => {
          self.oam_memory[index].x = data;
        },
        _  => {
          return;
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
            self.temp_vram_reg.set_nametable_x(self.controller_reg.get_nametable_x());
            self.temp_vram_reg.set_nametable_y(self.controller_reg.get_nametable_y());
          },
          0x1 => { // Mask
            self.mask_reg.flags = data;
          },
          0x2 => { // Status

          },
          0x3 => { // OAM Address
            self.oam_data_addr = data;
          },
          0x4 => { // OAM Data
            self.write_to_oam_memory(self.oam_data_addr, data);
            self.oam_data_addr = self.oam_data_addr.wrapping_add(1);
          },
          0x5 => { // Scroll
            if self.writing_high_byte_of_addr {
              self.fine_x = bitwise_utils::get_bits_16(data as u16, 0, 2) as u8;
              self.temp_vram_reg.set_coarse_x(bitwise_utils::get_bits_16(data as u16, 3, 7) as u8);
            } else {
              self.temp_vram_reg.set_fine_y(bitwise_utils::get_bits_16(data as u16, 0, 2) as u8);
              self.temp_vram_reg.set_coarse_y(bitwise_utils::get_bits_16(data as u16, 3, 7) as u8);
            }
            self.writing_high_byte_of_addr = !self.writing_high_byte_of_addr;

          },
          0x6 => { // PPU Address
            if self.writing_high_byte_of_addr {
              self.temp_vram_reg.flags &= 0xFF;
              self.temp_vram_reg.flags += ((data & 0x3F) as u16) << 8; 
            } else {
              self.temp_vram_reg.flags &= 0xFF00;
              self.temp_vram_reg.flags += (data as u16);
              self.vram_reg = self.temp_vram_reg; 
            }
            self.writing_high_byte_of_addr = !self.writing_high_byte_of_addr;
          },
          0x7 => { // PPU data
            self.write_to_ppu_bus(self.vram_reg.flags, data).unwrap();
            let increment_amount = if (self.controller_reg.get_increment_mode() != 0) { 32 } else { 1 };
            self.vram_reg.flags = (self.vram_reg.flags + increment_amount) & 0x3FFF;
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
            // panic!("Tried to read from PPU control register, which is not readable!");
          },
          0x1 => { // Mask
            return Ok(self.mask_reg.flags);
            // panic!("Tried to read from PPU mask register, which is not readable!");
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
            return Ok(self.oam_data_addr);
            // return Err(String::from("CPU tried to read from OAM address register, which is undefined."));
          },
          0x4 => { // OAM Data
            return Ok(self.read_from_oam_memory(self.oam_data_addr));
            // return Err(String::from("CPU tried to read from OAM data register, which is undefined."));
            
          },
          0x5 => { // Scroll
            panic!("Tried to read from PPU scroll register, which is not readable!");
            // return Ok(0);
          },
          0x6 => { // PPU Address
            panic!("Tried to read from PPU address register, which is not readable!");
            // return Ok(0);
          },
          0x7 => { // PPU data
            let read_result = self.read_from_ppu_bus(self.vram_reg.flags).unwrap();

            let return_value : u8;
            // Unless reading from palette memory, we return the value that is currently 
            // stored on the read buffer, and then update the buffer with the 
            // data located at self.ppu_addr
            // Essentially, most read() operations are delayed one cycle.
            if self.in_palette_memory_bounds(self.vram_reg.flags) {
              self.ppu_data_read_buffer = read_result;
              return_value = read_result;
            } else {
              return_value = self.ppu_data_read_buffer;
              self.ppu_data_read_buffer = read_result;
            }

            let increment_amount = if (self.controller_reg.get_increment_mode() != 0) { 32 } else { 1 };
            self.vram_reg.flags = (self.vram_reg.flags + increment_amount) & 0x3FFF; // Are we clearing the fine_y information here? Should we restore it after the increment?
            return Ok(return_value);

          },
          _ => return Err(String::from("Error while mirroring address in PPU write() function!"))
        }
      } else {
        return Err(String::from("Tried to read outside PPU bounds!"));
      }
    }
  }