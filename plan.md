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


## Phase 1.5: Ordering code
The current code was written in one single file because I couldn't be bothered to figure out Rust's import system before I started work on the emulator. We should divide the code into files and place it into one or multiple modules that make sense.

# Phase 2: CPU Testing

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


# Potential problems in original code

- The Stack Pointer variable is a u8 initialized at 0. When we decrease the SP, it becomes a -1, but it's an unsigned int, so we should see what exactly is happening when we add it to the stack start address.