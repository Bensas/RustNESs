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

# Phase 2: CPU Testing [Complete with sample code, not with ROM]

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

# Phase 4: Creating emulator struct, adding screen, name table and pattern table visualization to emulator
- Struct that will contain the CPU, PPU, Bus, and Cartridge
- We should run the ppu.clock() function three times as often as the cpu.clock() function
- [PENDING] Use Canvas widget with Quad primitives to draw pixels
- [PENDING] Add function that calls main clock() function until PPU has completed drawing frame and then sets the boolean to false again
