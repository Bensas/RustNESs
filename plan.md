# Phase 1: CPU programming [COMPLETE]

## Action Plan

- 6502
	- Status flags enum, 8-bit status register variable, functions to get_flag and set_flag easily
	- Other registers (Accum, x, y, sp, pc)
	- Function for each addressing mode, which should, upon returning, indicate whether an extra clock cycle is required
	- Function for each operation, plus a function for illegal operations. These functions should also indicate if an extra clock cycle should be used, I believe.
	- clock_cycle() function
		- When the cycles variable is 0:
			- Read opcode from the bus at the PC address
			- Increase PC
			- Retrieve instruction from instruction table
			- Set cycles variable to the cycles in the instruction
			- Call function corresponding to the address mode
			- Call function corresponding to the operation
			- If both functions indicated that they need an additional clock cycle, add a clock cycle to the variable
		- Decrement the clock variable 
 
	- reset signal function
	- irq signal function
	- nmi signal function
	
	- fetch data function, as well as variable for storing fetched data (?
	- absolute memory address storage variable (For fetching data after executing address mode functions)
	- relative memory address storage variable (for branching?)
	- opcode storage variable
	- cycle counter for the remaining cycles in the current instruction

	- Instruction struct with pointer to the address mode and operation functions, number of cycles required for the instruction, and maybe name of the instruction
	- 16x16 instruction table for lookup 16x16 matrix, could it be a hash table? (no real benefit to it tho)

- Bus (16-bit)
	- Read function
	- Write function
	- Connected Decives

- Ram
	- 65kb byte array

### Potential problems in original code

- [SOLVED] The Stack Pointer variable is a u8 initialized at 0. When we decrease the SP, it becomes a -1, but it's an unsigned int, so we should see what exactly is happening when we add it to the stack start address.
 	-> When the cpu is reset, the stack pointer is increased to a higher value; it should never become 0 or negative. In any case, rust detected the problem at compilation time with overflow protection; god bless this language :)

## Phase 1.5: Ordering code
The current code was written in one single file because I couldn't be bothered to figure out Rust's import system before I started work on the emulator. We should divide the code into files and place it into one or multiple modules that make sense.

# Phase 2: CPU Testing [COMPLETE with sample code, not with ROM]

## Action plan:
We need a way to test the CPU. Functional requirements:
- Visualize the CPU registers and status flags
- Visualize the RAM contents
	- Raw hex view
	- Instruction decoding view
- Send the CPU clock, reset, and irq signals
- Load content onto RAM, maybe opening a `.rom` file with raw content, or a `.txt` file with the content written in space-separated hex format.
- Allow user input to:
	- Manually send the CPU a clock signal, reset signal or irq signal
- Input params:
	- Input rom/txt file
	- Position in memory to load the file
	- Execution mode: automatic / manual (Whether the user must use input to tell the program to keep sending cycle instructions)

Once we have this program, we can test using the following:
- https://www.masswerk.at/6502/assembler.html to create custom testing code
- https://www.nesdev.org/wiki/Emulator_tests to run more advanced tests once we have basic functionality going

# Phase 3: Bus expansion, adding PPU [COMPLETE]

- Modify RAM class to have address range 0-0x1FFF with mirroring over 0x07FF

- Class to represent the PPU olc2C02 (Impl Device)
	- Includes its own bus that it can read and write to and from, with the following connected devices:
		- VRAM -> 2 name tables, each 1024 bytes
		- Palette ram with 32 bytes
	- Includes a reference to the main bus to read/write to/from cartridge? Maybe just a cartridge object that can be passed on init -> We decided to use a reference, hopefully Rust doesn't bite us for that :-)
	- Address range: 0x2000 - 0x3FFF with mirroring over 0x0007
	- read() and write() function will take the address and returns/modify data as follows:
		- 0x1 = Control
		- 0x2 Mask
		- 0x3 OAM Address
		- 0x4 OAM Data
		- 0x5 Scroll
		- 0x6 PPu Address
		- 0x7 PPU data
	- Use array with 64 pixel values to store color palette, get values from nesdev
	- Include 256x240 pixel screen buffer
	- Include name table visualization buffers (Two name tables)
	- Include pattern table visualization buffers (Two pattern tables)
	- Scanline, cycle and frame_render_complete variables
	- clock() function which:
		- sets pixel value of pixel screen buffer at cycle and scan_line position using color from color palette.
		- increases the `cycle` variable 
		- If the cycle variable reaches 341, we set it to 0 and increase the scan_line variable
		- If the scan_line reaches 261, we set it to -1 and se the frame_complete boolean to true;
	
- Class to represent the cartridge (impl Device)
	- Program memory "PRG" (vector of bytes)
	- Pattern memory "CHR" (vector of bytes)
	- Mapper trait that maps incoming address (from CPU or PPU) to correct memory
		- Contains number of CHR and PRG banks
		- Contains read and write functions (Could if be a Device impl?), these might need to be CPU and PPU specific, which take an address and return the remaped address
		- We'll implement mapper 000 to start with:
			- Read/Write from CPU will:
				- Check address bounds 0x8000 - 0xFFFF
				- Return address, mirrored if we only have 1 PRG bank `mapped_addr = addr & (seldf.nPRGBanks > 1 ? 0x7FFF : 0x3FFF)`
			- Read from PPU will just check address bounds 0x0000 - 0x1FFF, return the same adddress
	- Variables to define which mapper we are using (There are different physical mapping circuits that were used each cartridge used a specific one), how many banks we have on PRG and how many banks we have on CHR
	- Struct to store ROM header information:
		- char name[4]
		- u8 prg_rom_chunks;
		- u8 chr_rom_chunks;
		- u8 mapper1;
		- u8 mapper2;
		- u8 prg_ram_size;
		- u8 tv_system1;
		- u8 tv_system2;
		- char unused[];
	- Method to load information form `.nes` file:
		- Reads header
		- Ignores next 512 bytes
		- Obtain mapperID from the `mapper1` and `mapper2` header variables
			- nMapperID = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4)
		- Determine file type (0, 1, or 2) and act accordingly. For now we assume type 1 and do:
			- Read number of PRG banks from header
			- Set length of PRG vector to 16384 multiplied by number of banks
			- Read PRG data from file onto PRG vector
			- Do the same for CHR data, but using 8192 bytes per bank
		- Instantiate appropriate mapper struct based on mapperID
	- Read and write functions that will, depending on whether the CPU or PPU read:
		- Use the mapper to remap the address
		- Read/write the data (CPU to PRG, PPu to CHR)

### Potential problems in code

- [SOLVED] When we load a `.nes` file in the `create_cartridge_from_ines_file()` function, we might not be reading all the CHR and PRG data, due to inclusive/exclusive indexing (might not be reading the first byte of the CHR data, for example).

# Phase 4: Creating emulator struct, adding screen, name table and pattern table visualization to emulator [COMPLETE, without name_table or pattern_table visualization yet]
- Struct that will contain the CPU, PPU, Bus, and Cartridge
- We should run the ppu.clock() function three times as often as the cpu.clock() function
- Use Canvas widget with `fill_rectangle` to draw pixels
- Add function that calls main clock() function until PPU has completed drawing frame and then sets the boolean to false again

# Phase 5: PPU Land 1: Basic functionality and pattern tables [COMPLETE]

- Function to read the patter tables from the PPU (which might read it from the cartridge) and load it onto our visualization buffer
	- Each table is 4kb long
		- Contains 256 tiles, 16 bytes each
			- Each tile represents an 8x8 pixel grid, where each pixel has a value between 0 and 3 repreesnted by two bits (one LSB and one MSB)
			- Memory wise, each tile has, fist, 8 bytes, each representing the LSB of the pixel values for one row of the tile. After that, 8 more bytes, each byte containing the MSB of the pixel value for one row.
- Function to get a color from the palette using the palette ID plus the 0-3 (2-bit) color identifier.
	- Palette ram is stored at location 0x3F00 in the cartridge.
	- Each palette is 4 bytes in size, so we multiply the paletteID by four to get the palette memoty offset, and then add the color identifier value to get the adecuate color value.

- `trigger_cpu_nmi` variable to be used to let tthe CPU know we're at vertical_blank. (Only used if the `enable_nmi` flag is active in the controller register)

- Some PPU Registers:
	- Status register (from LSB to MSB):
		- 5 bits Unused 
		- 1 bit sprite_overflow
		- 1 bit sprite_zero_hit
		- 1 bit vertical_blank
	- Mask register (from LSB to MSB):
		- grayscale
		- render_background_left
		- render_sprites_left
		- render_background
		- render_sprites
		- enhance_red
		- enhance_green
		- enhance_blue
	- Controller regiter (from LSB to MSB):
		- nametable_x
		- nametable_y
		- increment_mode
		- pattern_sprite
		- pattern_background
		- sprite_size
		- slave_mode
		- enable_nmi
	- PPU Address -> Because it's one byte, the address is passed in two parts
	- PPU Data

- Complete write() and read() functions setting or getting the value of the registers.
	- For the PPU Address and PPU Data registers we take a specific approach:
		- We have the following variables
			- `address_high_byte` -> indicates whether we're writing to the low ot the high byte on thw address register
			- `ppu_data_read_buffer` -> stores the ppu data to be read by the CPU
			- `ppu_address :u16` -> stores the completed address specified by the CPU during two consecutide write() operations
		- PPU Address write:
			- On each write, we toggle between the high or the low byte and store it onto the ppu_address variable (starting on the high_byte)
		- PPU data write:
			- Writes the data onto the address specified by `ppu_address`
			- Increases the value of `ppu_address`
		- PPU Address read needs no implementation
		- PPU data read:
			- It's delayed by one read() action, except when reading from palette range
			- When we read(), we return the current contents of the `ppu_data_read_buffer`, and the data from `ppu_address` is loaded onto that buffer (intead of reading the data form `ppu_address` and returning it directly)
			- When we're reading from palette range ( > 0x3F00), we read the data at `ppu_address`, load it onto the buffer, and return it immediately.
			- We also increment the value of `ppu_address`
	- Status register:
		- read() will change the state of the device
			- `data = (status.reg & 0xE0) | (ppu_data_buffer & 0x1F)`
			- `status.vertical_blank = 0`
			- `address_high_byte = true`
- clock_cycle() function:
	- `if scanline == 241 && cycle == 1`
		- Set vertical_blank flag to 1
		- If `enable_nmi` is set to 1 in the control register, set the boolean variable `nmi` to true.
	- `if scanline == -1 && cycle == 1)`
		- `status.vertical_blank = 0`

- On the RustNESs clock() function, we should check if the `nmi` variable is set to true on the PPU, then call the `nmi()` function on the CPU and set `nmi` back to false.

- DOUBT: "reading from the cartridge" in David's implementation is reading from the PPU's internal arrays. I'm not sure when the information from the cartridge is loaded onto those arrays
	-> I believe this is incorrect. The read() and write() functions on the PPU call the read() and write() functions on the cartridge. Depending on the mapper, these might fail, in which case we will use the PPU's internal arrays. In the case of Mapper000, fro example, we will never use the PPU's internal memory for reading pattern table information, but we will store name table and palette information on the PPU's internal memory (done through the PPU's `data` register).


## Phase 5.5: testing pattern tables [COMPLETE]
- Visualization for Pattern tables in GUI, allowing the user to select the palette (value between 0 and 7) by pressing the "p" key, which increments the value ans wraps around.
- Try loading `nestest.nes`, maybe other games, veryifying that we can see patterns and palettes.


# Phase 6: PPU Land 2: Name tables [COMPLETE]

## Some notes
- NES loads two name_tables (2kb) so that we can scroll them across the screen using the scroll register on the PPU
- SMB uses vertical mirroring (two name_tables are side by side, with other two mirrored below them), while some games use horizontal mirroring, and some mappers llow for switching
- Name table addressing:
	- Name tables are 32x32 bytes, each byte representing one tile from pattern memory
	- We have 4 name tables to address.
	- Our addressing need to specify
		- Which nametable (4 possible values)
		- Which tile (X and Y position, each 32 possible values)
		- Which pixel in the tile (X and Y, each 8 possible values)
		- We can use a coarse address (12 bits) for nametable+tile, and two variables fineX and fineY for pixel.
	- Attribute memory
		- The last 2 rows (64 bytes) of each nametable actually contains the attribute memory, which contains palette info for 8x8=64 "attribute-tiles" on the nametable, each "attribute-tile" containing 16 actual tiles.
		- Each byte of the attribute memory is divided into 4 2-bit numbers, each defining the paletteID to be used for each 4-tile group in the "attribute-tile. From LSB to MSB: First 2 bits correspond to top-left tiles, then top-right, then bottom-left, then bottom-right.
		- You could also interpret it as: the 64 attribute bytes are separated into 128 2-bit values, each determining the palette value for a 2x2-tile "attribute-tile"

## Work
- Cartridge work:
	- When loading a ROM file, use the `mapper` header flags to determine the mirroring type (horizontal or vartical), so the PPU then knows which one to use.

PPU work:
	- Add two 16-bit loopy registers (`vram` and `tram`) for nametable addressing, each consisting of the flags:
			- coarse_x: 5-bit
			- coarse_y: 5-bit
			- nametable_x: 1 bit
			- nametable_y: 1 bit
			- fine_y: 3 bit
			- unused: 1 bit
		- Add a `fine_x` variable for nametable addressing to be used alongside the loopy regsiters
		- Replace the `ppu_address` variable with the loopy `vram` register
	- read_from_ppu_memory() and write_to_ppu_memory():
		- Check mirroring mode on the PPU and read from the name_table array at the appropriate index based on the received address and mirroring mode
	- read() and write() functions:
		- When we increase the address, increase it by 32 depending on increment mode (flag in the control register; if set, we increment by 32)
		- write() function:
			- when we write to the control register, we update the nametable_x and nametable_y values of the `tram` loopy register
			- When we write to the address register, we write to the `tram` register, and after we write the low byte (`writing_high_byte_of_addr == false`), we will set `vram = tram`.
			- When we write to the scroll register, we're gonna check the `writing_high_byte_of_addr` variable and toggle it, just like when writing to the address regiter. We're also gonna do the following:
				- When `writing_high_byte_of_addr == true`:
					- set the value of `fine_x` to the first 3 least significant bits from the writen data
					- set the `coarse_x` bits of the `tram` register to the following 5 bits of the written data
				- When `writing_high_byte_of_addr == false`:
					- Do the same, but with `fine_y` and `coarse_y` (note that, unlike `fine_x`, `fine_y` is a part of the `tram` register)
	- clock_cycle() function:
		- Following the diagram https://www.nesdev.org/w/images/default/4/4f/Ppu.svg, we store relevant data in the folloging PPU variables:
			- `bg_next_tile_id: u8`
			- `bg_next_tile_attribute: u8`
			- `bg_next_tile_lsb: u8`
			- `bg_next_tile_msb: u8` 
		- We include functions for scrolling:
			- They all check `if (mask.render_background || mask.render_sprites)` in order to do what they need to do.
			- IncrementX will:
				- Increment the `coarse_x` value of the `vram` register
				- If `coarse_x` goes above 31, we switch the value of `nametable_x` (Does this assume that we use vertical mirroring?) and reset `coarse_x` to 0
			- IncrementY will
				- Increment the `fine_y` value of the `vram` register
				- If `fine_y` goes above 7, we increase the value of `coarse_y` and reset `fine_y` to 0.
				- If `coarse_y` goes above 29, we switch the value of `nametable_y` and reset `coarse_y` to 0
				- If `coarse_y` goes above 31 (we're in attribute memory), we only reset `coarse_y`.
			- TransferAddresssX will
				- `vram.nametable_x = tram.nametable_x`
				- `vram.coarse_x = tram.coarse_x`
			- TransferAddresssY will
				- `vram.nametable_y = tram.nametable_y`
				- `vram.coarse_y = tram.coarse_y`
				- `vram.fine_y = tram.fine_y`
	- u16 Shift registers:
		- `bg_shifter_pattern_lo: u16`
		- `bg_shifter_pattern_hi: u16`
		- `bg_shifter_attrib_lo: u16`
		- `bg_shifter_attrib_hi: u16`
	- Functions for shift register handling:
		- LoadBackgroundShifters:
			- `bg_shifter_pattern_lo = (bg_shifter_pattern_lo & 0xFF00) | bg_next_tile_lsb;`
			- `bg_shifter_pattern_hi = (bg_shifter_pattern_hi & 0xFF00) | bg_next_tile_msb;`
			- `bg_shifter_attrib_lo = (bg_shifter_attrib_lo & 0xFF00) | ((bg_next_tile_attrib & 0b01)? 0xFF : 0x00);`
			- `bg_shifter_attrib_hi = (bg_shifter_attrib_hi & 0xFF00) | ((bg_next_tile_attrib & 0b10)? 0xFF : 0x00);`
		- UpdateShifters:
			- Check `if mask.render_background` in order to do what it needs to do
			- `bg_shifter_pattern_lo << 1`
			- `bg_shifter_pattern_hi << 1`
			- `bg_shifter_attrib_lo << 1`
			- `bg_shifter_attrib_hi << 1`

- GUI:
	- For testing, we can visualize the pattern table information (palette id for each tile in the nametable)

# Phase 7: Controller Inputs [COMPLETE]

- Controller input is handled using the memory addresses 0x4016 and 0x4017
- When the CPU writes to address 0x4016 or 0x4017, the internal register of the controller is updated with the current pressed/unpressed value of the 8 buttons.
- After that, the CPU can read 8 consecutive times to address 0x4016 or 0x4017 to receive, one by one, the pressed/unpressed value of each button.

# Phase 8: Performance improvements -> Turned into cycle accuracy [COMPLETE]

- We've measured frame render time at ~100ms, which is not good enough. Additionally, DK takes ages to get from the menu screen to the in-game screen, and SMB doesn't even ever get to the in-game screen.
- I tried removing clock-cycle accuracy, but performance decreased.
- Opcode fetching is donde by indexing an array, so that should not be a performance bottleneck.
### Hypotheses
- Cycle accuracy is still quite imperfect. That might not just be causing performance issues, but it might be preventing games from working properly, which might explain DK's sluggish load time and SMB's infinite one. If we improve cycle accuracy, we might kill two birds with one stone.


### Realization and conclusion:
- I was compiling for development. Running `cargo run --release` improves frame loading time by ~10 times, which is good enough. I'll still improve cycle accuracy.

# Phase 9: PPU Foreground rendering [COMPLETE]

## Some notes
- Sprites are stored in OAM (Object Attribute Memory) of the PPU (not on the PPU bus)
	- 256 bytes in size, containing 64 sprites
	- 4 bytes per sprite:
		- X coord
		- Y coord
		- tile_id (from pattern memory)
		- attributes (priority, palette and orientation) -> orientation allows us to flip sprites horizontally
	- Sprites can be 8x8px or 8x16px
	- OAM addr and OAM data registers in the PPU are meant to be used by the CPU to read/write from/to the OAM memory, but they are very slow, so instead, most games use DMA:
		- WHen the CPU writes the adress of a page in the CPU bus to address 0x4014, DMA (Direct Memory Access) kicks in, disables the CPU for 512 clock cycles, and automatically copies the entire page to the PPU's OAM memory. This is way faster than the manual ethod.

- Rendering sprites:
	- At the end of the visible scanline, we search the OAM for up to 8 sprites that will be visible on the next scanline
	- As the scanline advances, reduce the X coord for each sprite 
	- If X coord reaches 0, we start to draw the sprite
	- if there are multiple sprites, we must resolve priority(based on sprite priority, background priority and transparency) to determine which one we draw

## Work
- PPU:
	- OAM array of sprite_object structs:
		- sprite_object struct:
			y: u8,
			id: u8,
			attributes: u8, // Should it be a struct?
			x: u8
	- `oam_data_addr: u8` variable for traditional OAM access
	- `sprites_on_curr_scanline: Vec<sprite_object>` 
	- `sprites_on_curr_scanline_pattern_lo: Vec<u8>`
	- `sprites_on_curr_scanline_pattern_hi: Vec<u8>`
	- read() and write() functions:
		- OAM Address register:
			- read/write to `oam_addr` variable
		- OAM Data register:
			- read/write from/to OAM array (indexed with `self.oam_addrs`)
	- clock_cycle() function:
		- `if (self.cycle == 257 && self.scan_line >= 0)` // End of the visible scanline
			- empty `sprites_on_current_scanline`
			- Loop through all values in the OAM and for each, do `let diff = scanline - object.y`
				`if diff >= 0 && diff < (self.control_reg.sprite_size ? 16 : 8)`
				`sprites_on_current_scanline.push(object)` (Only if we haven't already found 9 sprites)
			- Check if we have more than 8 sprites(9), then set `self.status_reg.sprite_overflow = 1`
		- `if (self.cycle == 340)`
			- fetch `sprites_on_curr_scanline_pattern_hi` and `-lo`from memory location (for each sprite in `sprites_on_current_scanline`)
				- mem location is determined by `control.pattern_sprite`, `control.sprite_size`, `object.orientation` and `object.tile_id`:
					- `diff = self.scan_ line - object.y`
					- With 8x8:
						- `START_ADDR = PATT_MEMORY_START + (control_reg.pattern_sprite) * 4096`
						- non-flipped sprite
							- (START_ADDR + tile_id * 16 + diff)
							- (START_ADDR + tile_id * 16 + diff + 8)
						- If flipped vertically (msb bit of attribute byte set to 1):
							- (START_ADDR + tile_id * 16 + 7 - diff)
							- (START_ADDR + tile_id * 16 + 7 - diff + 8)
					- With 8x16:
						- `START_ADDR = PATT_MEMORY_START + (object.tile_id & 0x01) * 4096`
						- non-flipped:
							- `if diff < 8` (we're drawing the top half of the sprite):
								- (START_ADDR + (tile_id & 0b11111110) * 16 + (diff % 8))
								- (START_ADDR + (tile_id & 0b11111110) * 16 + (diff % 8) + 8)
							- `else` (we're drawing the bottom half of the sprite):	
								- (START_ADDR + ((tile_id & 0b11111110) + 1) * 16 + (diff % 8))
								- (START_ADDR + ((tile_id & 0b11111110) + 1) * 16 + (diff % 8) + 8)
						- flipped vertically:
							- `if diff < 8` (we're drawing the top half of the sprite, which is actually the bottom half):
								- (START_ADDR + ((tile_id & 0b11111110) + 1) * 16 + (7 - (diff % 8)))
								- (START_ADDR + ((tile_id & 0b11111110) + 1) * 16 + (7 - (diff % 8)) + 8)
							- `else` (we're drawing the bottom half of the sprite, which is actually the top half):	
								- (START_ADDR + (tile_id & 0b11111110) * 16 + (7 - (diff % 8)))
								- (START_ADDR + (tile_id & 0b11111110) * 16 + (7 - (diff % 8)) + 8)
			- Chcek if they should be flipped horizontally (second msb of attribute byte set to 1) and flip them if necessary.
			- Add to `sprites_on_curr_scanline_pattern_hi` and `-lo` vectors
		- `if (self.scan_line == -1 && self.scycle == 1)` (aside from existing reset of vertical_blank)
			- `self.status_reg.est_sprite_overflow(0)`
			- clear `self.sprites_on_curr_scanline_pattern_lo` and `-hi`
	- Modify update_shift_registers():
		- Add `if (self.mask_reg.get_render_sprites() && self.cycles >= 1 && self.cycle < 258)`
			- for each `sprite_obj` in `self.sprites_on_curr_scanline`:
				- `if sprite_obj.x > 0 {  sprite_obj.x -= 1 }`
				- `else`, shift corresponding register in `sprites_on_curr_scanline_pattern_lo` and `-hi`
				- Maybe we could just check `self.scan_line` vs `sprite_obj.x` instead of decreasing. I don't like the concept of modifying `x`, even if it's a copy of the original sprte.

- Sprite ZXero detection algorithm
	- PPU
		- Add `sprite_zero_hit_possible: bool` and `sprite_zero_being_rendered: bool` variables
		- clock_cycle()
			- When we add `sprite_objs` to `sprites_in_curr_scanline`, we check if we're adding the first sprite (sprite zero) and set the value of `sprite_zero_hit_possible` accordingly
			- When we actually render the sprites, we set the `sprite_zero_being_rendered` variable to true if the rendered sprite is sprite zero.
			- When we check for `bg_pixel_value != 0 && fg_pixel_value != 0`, if the previous two variable are set, we do the following:
				- if `self.mask.get_render_background() and self.mask_reg.get_render_sprites()`:
					- `if (self.mask.get_render_background_left() == 0 &&self.mask.get_render_sprites_left() == 0)`
						- `if (self.cycle >= 9 && self.cycle < 258)`
							- `self.status.set_sprite_zero_hit(1)`
					- `else`
						- `if (self.cycle >= 1 && self.cycle < 258)`
							- `self.status.set_sprite_zero_hit(1)` 

- Bus (we might have to implement this on the CPU instead):
	- Add variables:
		- `dma_page: u8`
		- `dma_addr: u8`
		- `dma_data: u8`
		- `dma_transfer: bool`
	- When CPU writes `data` to 0x4014:
		- `data_page = data`
		- `dma_addr = 0x00`
		- `dma_transfer = true`

- RustNESs:
	- clock_cycle() function:
		- only do `self.cpu.clock_cycle()` if `dma_transfer` is set to false on the bus
		- Otherwise:
			- First wait for 2 clock cycles
			- Then alternate between reading from the page in the cpu bus and writing (directly)to the PPU's OAM array, incrementing `oam_addr` on every write (and checking for max_val for u8, where we set `dma_transfer` to false) 

# Phase 10: Debugging

- DK works well, which is really cool!
- SMB has two problems:
	- (a) The background is rendered black
	- (b) The game does not start, it gets stuck on the screen.
- KF works well, but has (c) horrible screen artifacts.
- IC works well


- After some investigation, I figured out problem (a) is being caused by incorrect mirroring on PPU palette, which is now solved.
- Problem (b) is still unresolved, but I believe it has to do with sprite zero collision detection.
- Problem (c) seems to be a combination of factors:
	- Timing issues: there seems to be screen tearing/wobblyness, and I did find (and fix) a timing issue in branch instructions on the CPU. There are probbaly more similar issues. I'll use test roms dedicated to timing debugging for this.
	- One sprite (I believe it's sprite zero) flickers to the top of the screen periodically, since commit ed46d9ecaff4c42a82cb4c619f2b2f9635663ef2.