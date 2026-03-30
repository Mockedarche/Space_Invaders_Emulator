use std::{fs, usize};

/*
 * Todo
 * Perform performance optimization and code consistancy (debugging all opcodes created stuff needing fixing)
 */

 #[derive(PartialEq)]
pub enum LoadRomResult {
    Ok,
    Error,
    NotFound,
}

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum StepInstructionResult {
    Ok,
    Error,
    NoOperation,
    Halt,
}

const MEMORY_SIZE: usize = 65536;

#[allow(dead_code)]
pub struct I8080Core {
    pub memory: [u8; MEMORY_SIZE],

    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    pub program_counter: u16,
    pub stack_pointer: u16,

    pub sign: bool,
    pub zero: bool,
    pub auxiliary_carry: bool,
    pub parity: bool,
    pub carry: bool,
    pub on_out: Option<fn(u8, u8)>,
    pub instruction_number: usize,
}

impl I8080Core {
    pub fn new() -> Self {
        Self {
            memory: [0; MEMORY_SIZE],
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,

            program_counter: 0,
            stack_pointer: 0,
            sign: false,
            zero: false,
            auxiliary_carry: false,
            parity: false,
            carry: false,
            on_out: None,
            instruction_number: 0,
        }
    }

    /* i8080_load_rom - loads the ROM into the cores memory
     * Expects: N/A
     * Does: Takes the ROM places it into memory and if it fails provides print feedback aswell as a LoadRomResult return
     * Returns: A LoadRomResult that indicates what happened
     */
    pub fn i8080_load_rom(&mut self, path: &str, address: u16) -> LoadRomResult {
        match fs::read(path) {
            Ok(data) => {
                let start = address as usize;
                let end = start + data.len();
                
                if end > self.memory.len() {
                    return LoadRomResult::Error;
                }
                
                self.memory[start..end].copy_from_slice(&data);
                self.program_counter = address;
                
                println!("ROM loaded successfully at 0x{:04X}", address);
                LoadRomResult::Ok
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("ROM file not found: {}", path);
                LoadRomResult::NotFound
            }
            Err(_) => {
                println!("Error loading ROM file: {}", path);
                LoadRomResult::Error
            }
        }
    }
    /*
     * set_zero_flag - Function
     * Expects: self to be intialized and value to be valid data (which is to say the resulting value of
     * an opcode)
     * Does: Sets the i8080Core objects zero flag if value is zero to true else false
     */
    #[allow(dead_code)]
    pub fn set_zero_flag(&mut self, value: u8) {
        self.zero = value == 0;
    }

    /*
     * set_sign_flag - Function
     * Expects: self to be intiialized and value to be valid data (which is to say the resulting value of
     * an opcode)
     * Does: Sets the i8080Core objects sign flag if value's most sig bit is 1 to true else false
     */
    #[allow(dead_code)]
    pub fn set_sign_flag(&mut self, value: u8) {
        self.sign = (value >> 7) == 1;
    }

    /*
     * set_auxiliary_carry_addition_flag - Function
     * Expects: self to be initialized and value to be valid data (which is to say the resulting value of
     * an opcode)
     * Does: Sets the i8080Core objects auxiliary carry flag if the most sig bit of the first nibble becomes the
     * first bit of the second nibble EX: 0x0F + 0x01 = 0x10
     */
    #[allow(dead_code)]
    pub fn set_auxiliary_carry_addition_flag(&mut self, first: u8, second: u8, result: u8) {
        self.auxiliary_carry = ((first ^ second ^ result) & 0x10) != 0;
    }

    /*
     * set_auxiliary_carry_subtraction_flag - Function
     * Expects: self to be initialized and value to be valid data (which is to say the resulting value of
     * an opcode)
     * Does: Sets the i8080Core objects auxiliary carry flag if the lower nibble needed a borrow
     */
    #[allow(dead_code)]
    pub fn set_auxiliary_carry_subtraction_flag(&mut self, first: u8, second: u8) {
        self.auxiliary_carry = (first & 0x0F) >= (second & 0x0F);
    }

    /*
     * set_parity_flag - Function
     * Expects: self to be initialized and value to be valid data (which is to say the resulting value of
     * an opcode)
     * Does: Sets the i8080Core objects parity flag if last 8 bits of value are an even amount of 1 then true
     * else false
     */
    #[allow(dead_code)]
    pub fn set_parity_flag(&mut self, result: u16) {
        let low_eight = result & 0xFF;
        let mut count = 0;
        for i in 0..8 {
            if (low_eight & (1 << i)) != 0 {
                count += 1;
            }
        }
        self.parity = (count % 2) == 0;
    }

    /*
     * set_carry_flag_arithmetic_addition - Function
     * Epects: self to be initialized and value to be valid data (which is to say its the correct registers value)
     * Does: Sets the carry flag if the 9th bit is 1 indicating the u8 arithmetic actually overflowed
     */
    #[allow(dead_code)]
    pub fn set_carry_flag_arithmetic_addition(&mut self, value: u16) {
        self.carry = (value & 0x0100) != 0;
    }

    /*
     * set_carry_flag_arithmetic_subtraction - Function
     * Epects: self to be initialized and value to be valid data (which is to say its the correct registers value)
     * Does: Sets the carry flag if a borrow occured in a subtraction
     */
    #[allow(dead_code)]
    pub fn set_carry_flag_arithmetic_subtraction(&mut self, first: u8, second: u8) {
        self.carry = first < second;
    }

    /*
     * print_state - Debug tool that prints the entire state of the core besides all the memory
     * Expects: N/A
     * Does: Prints out in a nice style the cores variables for debugging
     */
    #[allow(dead_code)]
    pub fn print_state(&self) {
        println!("PC: 0x{:04X} | SP: 0x{:04X}", self.program_counter, self.stack_pointer);
        println!("A: 0x{:02X} | B: 0x{:02X} | C: 0x{:02X} | D: 0x{:02X} | E: 0x{:02X} | H: 0x{:02X} | L: 0x{:02X}", 
            self.a, self.b, self.c, self.d, self.e, self.h, self.l);
        println!("Flags: Z={} S={} P={} C={} AC={}", 
            self.zero as u8, self.sign as u8, self.parity as u8, self.carry as u8, self.auxiliary_carry as u8);
    }

    /*
     * i8080_step - Function
     * Epxects: self to be initialized
     * Does: Performs one instruction (the one pointed at by the program counter)
     * Returns: A StepInstructionResult indicating how things went in the execution of this instruction
     */
    #[allow(dead_code)]
    pub fn i8080_step(&mut self) -> StepInstructionResult {
        let debug = false;
        let instruction: u8;
        let temp1_8: u8;
        let temp2_8: u8;
        let mut temp3_16: u16;

        self.instruction_number = self.instruction_number.wrapping_add(1);
        instruction = self.memory[self.program_counter as usize];

        // Debug statement to see what opcodes are running and the cores state (generally useful for basic debug)
        if debug {
            println!("Core state before instruction number #{} and instruction: {:02X}", self.instruction_number, instruction);
            self.print_state();
            
        }
        
        // Given the instruction performs the indicated opcode 
        match instruction {
            0x00 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x01 => {
                self.b = self.memory[(self.program_counter as usize) + 2];
                self.c = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x02 => {
                self.memory[((self.b as u16) << 8 | self.c as u16) as usize] = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x03 => {
                temp3_16 = (self.b as u16) << 8 | (self.c as u16);
                temp3_16 = temp3_16.wrapping_add(1);
                self.b = (temp3_16 >> 8) as u8;
                self.c = (temp3_16 & 0x00FF) as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x04 => {
                let original = self.b;

                self.b = self.b.wrapping_add(1);
                self.set_sign_flag(self.b);
                self.set_zero_flag(self.b);
                self.set_auxiliary_carry_addition_flag(original, 1, self.b);
                self.set_parity_flag(self.b as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x05 => {

                let original = self.b;
                self.b = self.b.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(self.b);
                self.set_zero_flag(self.b);
                self.set_parity_flag(self.b as u16);
                self.program_counter = self.program_counter.wrapping_add(1);

                return StepInstructionResult::Ok;
            }
            0x06 => {
                self.b = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x07 => {
                self.carry = (self.a & 0x80) != 0;
                self.a = self.a << 1 | if self.carry { 1 } else { 0 };
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x08 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x09 => {
                let hl = (self.h as u16) << 8 | self.l as u16;
                let bc = (self.b as u16) << 8 | self.c as u16;
                let sum = hl.wrapping_add(bc);
                self.carry = sum < hl;
                self.h = (sum >> 8) as u8;
                self.l = sum as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x0A => {
                self.a = self.memory[((self.b as u16) << 8 | self.c as u16) as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x0B => {
                temp3_16 = (self.b as u16) << 8 | self.c as u16;
                temp3_16 = temp3_16.wrapping_sub(1);
                self.b = (temp3_16 >> 8) as u8;
                self.c = (temp3_16 & 0x00FF) as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x0C => {
                let original = self.c;
                self.c = self.c.wrapping_add(1);
                self.set_sign_flag(self.c);
                self.set_zero_flag(self.c);
                self.set_auxiliary_carry_addition_flag(original, 1, self.c);
                self.set_parity_flag(self.c as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x0D => {
                let original = self.c;
                self.c = self.c.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(self.c);
                self.set_zero_flag(self.c);
                self.set_parity_flag(self.c as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x0E => {
                self.c = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x0F => {
                self.carry = (self.a & 0x01) != 0; 

                self.a = (self.a >> 1) | if self.carry { 0x80 } else { 0 };
  
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x10 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x11 => {
                self.d = self.memory[(self.program_counter as usize) + 2];
                self.e = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x12 => {
                self.memory[((self.d as u16) << 8 | self.e as u16) as usize] = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x13 => {
                temp3_16 = (self.d as u16) << 8 | (self.e as u16);
                temp3_16 = temp3_16.wrapping_add(1);
                self.d = (temp3_16 >> 8) as u8;
                self.e = (temp3_16 & 0x00FF) as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x14 => {
                let original = self.d;

                self.d = self.d.wrapping_add(1);
                self.set_sign_flag(self.d);
                self.set_zero_flag(self.d);
                self.set_auxiliary_carry_addition_flag(original, 1, self.d);
                self.set_parity_flag(self.d as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x15 => {
                let original = self.d;
                self.d = self.d.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0; 
                self.set_sign_flag(self.d);
                self.set_zero_flag(self.d);
                self.set_parity_flag(self.d as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x16 => {
                self.d = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x17 => {
                let old_carry = self.carry;
                let new_carry = (self.a & 0x80) != 0;  // bit 7 before shift
                self.a = (self.a << 1) | if old_carry { 1 } else { 0 };
                self.carry = new_carry;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x18 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x19 => {
                let hl = (self.h as u16) << 8 | self.l as u16;
                let de = (self.d as u16) << 8 | self.e as u16;
                let sum = hl.wrapping_add(de);
                self.carry = sum < hl;
                self.h = (sum >> 8) as u8;
                self.l = sum as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x1A => {
                self.a = self.memory[((self.d as u16) << 8 | (self.e as u16)) as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x1B => {
                temp3_16 = (self.d as u16) << 8 | self.e as u16;
                temp3_16 = temp3_16.wrapping_sub(1);
                self.d = (temp3_16 >> 8) as u8;
                self.e = (temp3_16 & 0x00FF) as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x1C => {
                let original = self.e;
                self.e = self.e.wrapping_add(1);
                self.set_sign_flag(self.e);
                self.set_zero_flag(self.e);
                self.set_auxiliary_carry_addition_flag(original, 1, self.e);
                self.set_parity_flag(self.e as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x1D => {
                let original = self.e;
                self.e = self.e.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(self.e);
                self.set_zero_flag(self.e);
                self.set_parity_flag(self.e as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x1E => {
                self.e = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x1F => {
                let old_carry = self.carry;
                let new_carry = (self.a & 0x01) != 0;  // bit 0 before shift
                self.a = (self.a >> 1) | if old_carry { 0x80 } else { 0 };
                self.carry = new_carry;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x20 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x21 => {
                self.h = self.memory[(self.program_counter as usize) + 2];
                self.l = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x22 => {
                let addr = (self.memory[(self.program_counter as usize) + 2] as u16) << 8
                    | (self.memory[(self.program_counter as usize) + 1] as u16);
                self.memory[addr as usize] = self.l;
                self.memory[addr as usize + 1] = self.h;
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x23 => {
                temp3_16 = (self.h as u16) << 8 | (self.l as u16);
                temp3_16 = temp3_16.wrapping_add(1);
                self.h = (temp3_16 >> 8) as u8;
                self.l = (temp3_16 & 0x00FF) as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x24 => {
                let original = self.h;

                self.h = self.h.wrapping_add(1);
                self.set_sign_flag(self.h);
                self.set_zero_flag(self.h);
                self.set_auxiliary_carry_addition_flag(original, 1, self.h);
                self.set_parity_flag(self.h as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x25 => {
                let original = self.h;
                self.h = self.h.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(self.h);
                self.set_zero_flag(self.h);
                self.set_parity_flag(self.h as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x26 => {
                self.h = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x27 => {
                let lsb = self.a & 0x0F;
                let msb = self.a >> 4;
                let mut correction: u8 = 0;
                let mut cy = self.carry;

                if self.auxiliary_carry || lsb > 9 {
                    correction += 0x06;
                }
                if self.carry || msb > 9 || (msb >= 9 && lsb > 9) {
                    correction += 0x60;
                    cy = true;
                }

                // Full ADD to set S, Z, P, AC via XOR formula
                let original_a = self.a;
                let result = original_a.wrapping_add(correction);
                self.set_auxiliary_carry_addition_flag(original_a, correction, result);
                self.a = result;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                // Carry is determined by whether high correction was needed, not by overflow
                self.carry = cy;

                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x28 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x29 => {
                let hl = (self.h as u16) << 8 | self.l as u16;
                let sum = hl.wrapping_add(hl);
                self.carry = sum < hl;
                self.h = (sum >> 8) as u8;
                self.l = sum as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x2A => {
                let addr = (self.memory[(self.program_counter as usize) + 2] as u16) << 8
                    | (self.memory[(self.program_counter as usize) + 1] as u16);
                self.l = self.memory[addr as usize];
                self.h = self.memory[addr as usize + 1];
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x2B => {
                temp3_16 = (self.h as u16) << 8 | self.l as u16;
                temp3_16 = temp3_16.wrapping_sub(1);
                self.h = (temp3_16 >> 8) as u8;
                self.l = (temp3_16 & 0x00FF) as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x2C => {
                let original = self.l;
                self.l = self.l.wrapping_add(1);
                self.set_sign_flag(self.l);
                self.set_zero_flag(self.l);
                self.set_auxiliary_carry_addition_flag(original, 1, self.l);
                self.set_parity_flag(self.l as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x2D => {
                let original = self.l;
                self.l = self.l.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(self.l);
                self.set_zero_flag(self.l);
                self.set_parity_flag(self.l as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x2E => {
                self.l = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x2F => {
                self.a = !self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x30 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x31 => {
                self.stack_pointer = (self.memory[(self.program_counter as usize) + 2] as u16) << 8
                    | self.memory[(self.program_counter as usize) + 1] as u16;


                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x32 => {
                let addr = (self.memory[(self.program_counter as usize) + 2] as u16) << 8
                    | (self.memory[(self.program_counter as usize) + 1] as u16);
                self.memory[addr as usize] = self.a;
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x33 => {
                self.stack_pointer = self.stack_pointer.wrapping_add(1);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x34 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let original = self.memory[addr as usize];
                let result = original.wrapping_add(1);
                self.memory[addr as usize] = result;

                self.set_sign_flag(result);
                self.set_zero_flag(result);
                self.set_auxiliary_carry_addition_flag(original, 1, result);
                self.set_parity_flag(result as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x35 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let original = self.memory[addr as usize];
                let result = original.wrapping_sub(1);  
                self.memory[addr as usize] = result;    

                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(result);
                self.set_zero_flag(result);
                self.set_parity_flag(result as u16);
                self.program_counter = self.program_counter.wrapping_add(1);

                return StepInstructionResult::Ok;
            }
            0x36 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);

                self.memory[addr as usize] = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x37 => {
                self.carry = true;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x38 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0x39 => {
                let hl = (self.h as u16) << 8 | self.l as u16;
                let sum = hl.wrapping_add(self.stack_pointer);
                self.carry = sum < hl;
                self.h = (sum >> 8) as u8;
                self.l = sum as u8;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x3A => {
                let addr = (self.memory[(self.program_counter as usize) + 2] as u16) << 8
                    | (self.memory[(self.program_counter as usize) + 1] as u16);


                self.a = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(3);
                return StepInstructionResult::Ok;
            }
            0x3B => {
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x3C => {
                let original = self.a;
                self.a = self.a.wrapping_add(1);
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_auxiliary_carry_addition_flag(original, 1, self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x3D => {
                let original = self.a;
                self.a = self.a.wrapping_sub(1);
                self.auxiliary_carry = (original & 0x0F) != 0;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x3E => {
                self.a = self.memory[(self.program_counter as usize) + 1];
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0x3F => {
                self.carry = !self.carry;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x40 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x41 => {
                self.b = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x42 => {
                self.b = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x43 => {
                self.b = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x44 => {
                self.b = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x45 => {
                self.b = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x46 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                if debug {
                    let value = self.memory[addr as usize];
                    println!("MOV B,M: reading 0x{:02X} from address 0x{:04X}", value, addr);
                }
                self.b = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x47 => {
                self.b = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x48 => {
                self.c = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x49 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x4A => {
                self.c = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x4B => {
                self.c = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x4C => {
                self.c = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x4D => {
                self.c = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x4E => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.c = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x4F => {
                self.c = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x50 => {
                self.d = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x51 => {
                self.d = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x52 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x53 => {
                self.d = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x54 => {
                self.d = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x55 => {
                self.d = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x56 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.d = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x57 => {
                self.d = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x58 => {
                self.e = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x59 => {
                self.e = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x5A => {
                self.e = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x5B => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x5C => {
                self.e = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x5D => {
                self.e = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x5E => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.e = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x5F => {
                self.e = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x60 => {
                self.h = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x61 => {
                self.h = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x62 => {
                self.h = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x63 => {
                self.h = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x64 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x65 => {
                self.h = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x66 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.h = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x67 => {
                self.h = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x68 => {
                self.l = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x69 => {
                self.l = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x6A => {
                self.l = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x6B => {
                self.l = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x6C => {
                self.l = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x6D => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x6E => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.l = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x6F => {
                self.l = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x70 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                if debug {
                    println!("MOV M,B: writing 0x{:02X} to address 0x{:04X}", self.b, addr);
                }
                self.memory[addr as usize] = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x71 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.memory[addr as usize] = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x72 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.memory[addr as usize] = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x73 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.memory[addr as usize] = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x74 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.memory[addr as usize] = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x75 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.memory[addr as usize] = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x76 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Halt;
            }
            0x77 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.memory[addr as usize] = self.a;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x78 => {
                self.a = self.b;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x79 => {
                self.a = self.c;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x7A => {
                self.a = self.d;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x7B => {
                self.a = self.e;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x7C => {

                self.a = self.h;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x7D => {
                self.a = self.l;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x7E => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                self.a = self.memory[addr as usize];
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x7F => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x80 => {
                let sum = self.a as u16 + self.b as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.b, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x81 => {
                let sum = self.a as u16 + self.c as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.c, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x82 => {
                let sum = self.a as u16 + self.d as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.d, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x83 => {
                let sum = self.a as u16 + self.e as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.e, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x84 => {
                let sum = self.a as u16 + self.h as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.h, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x85 => {
                let sum = self.a as u16 + self.l as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.l, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x86 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let sum = self.a as u16 + self.memory[addr as usize] as u16;
                self.set_auxiliary_carry_addition_flag(
                    self.a,
                    self.memory[addr as usize],
                    sum as u8,
                );
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x87 => {
                let sum = self.a as u16 + self.a as u16;
                self.set_auxiliary_carry_addition_flag(self.a, self.a, sum as u8);
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x88 => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.b as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.b & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x89 => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.c as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.c & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x8A => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.d as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.d & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x8B => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.e as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.e & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x8C => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.h as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.h & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x8D => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.l as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.l & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x8E => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16
                    + self.memory[addr as usize] as u16
                    + carry_in as u16;

                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.memory[addr as usize] & 0x0F) + carry_in) > 0x0F;
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x8F => {
                let carry_in = if self.carry { 1 } else { 0 } as u8;
                let sum = self.a as u16 + self.a as u16 + carry_in as u16;
                
                // Auxiliary carry from bits 3-4 including the carry in
                self.auxiliary_carry = ((self.a & 0x0F) + (self.a & 0x0F) + carry_in) > 0x0F;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.set_carry_flag_arithmetic_addition(sum);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x90 => {
                let dif = (self.a as u16).wrapping_sub(self.b as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.b);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.b);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x91 => {
                let dif = (self.a as u16).wrapping_sub(self.c as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.c);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.c);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x92 => {
                let dif = (self.a as u16).wrapping_sub(self.d as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.d);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.d);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x93 => {
                let dif = (self.a as u16).wrapping_sub(self.e as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.e);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.e);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x94 => {
                let dif = (self.a as u16).wrapping_sub(self.h as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.h);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.h);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x95 => {
                let dif = (self.a as u16).wrapping_sub(self.l as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.l);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.l);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x96 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let value_from_memory = self.memory[addr as usize] as u8;
                let dif = (self.a as u16).wrapping_sub(value_from_memory as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, value_from_memory);
                self.set_carry_flag_arithmetic_subtraction(self.a, value_from_memory);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x97 => {
                let dif = (self.a as u16).wrapping_sub(self.a as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.a);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.a);
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x98 => {  // SBB B
                let a = self.a;
                let b = self.b;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (b as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                // Auxiliary carry – borrow from bit 3, including carry_in
                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((b & 0x0F) as u16 + carry_in as u16);
                // Carry flag – borrow from bit 7
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }

            0x99 => {  // SBB C
                let a = self.a;
                let c = self.c;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (c as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((c & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }

            0x9A => {  // SBB D
                let a = self.a;
                let d = self.d;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (d as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((d & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }

            0x9B => {  // SBB E
                let a = self.a;
                let e = self.e;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (e as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((e & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }

            0x9C => {  // SBB H
                let a = self.a;
                let h = self.h;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (h as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((h & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }

            0x9D => {  // SBB L
                let a = self.a;
                let l = self.l;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (l as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((l & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x9E => {  // SBB M
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let mem = self.memory[addr as usize];
                let a = self.a;
                let carry_in = if self.carry { 1u8 } else { 0u8 };

                let full_borrow = (mem as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);
                let result8 = dif as u8;

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((mem & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;

                self.a = result8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0x9F => {  // SBB A
                let a = self.a;
                let carry_in = if self.carry { 1u8 } else { 0u8 };
                let full_borrow = (a as u16) + (carry_in as u16);
                let dif = (a as u16).wrapping_sub(full_borrow);

                self.auxiliary_carry = ((a & 0x0F) as u16) >= ((a & 0x0F) as u16 + carry_in as u16);
                self.carry = (a as u16) < full_borrow;
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA0 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.b >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.b;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA1 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.c >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.c;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA2 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.d >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.d;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA3 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.e >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.e;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA4 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.h >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.h;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA5 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.l >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.l;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA6 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let value_from_memory = self.memory[addr as usize] as u8;
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (value_from_memory >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & value_from_memory;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA7 => {
                // Save original values for auxiliary carry calculation
                let first_bit3 = (self.a >> 3) & 1;
                let second_bit3 = (self.a >> 3) & 1;
                
                // Set auxiliary carry based on OR of bit 3 from both operands
                self.auxiliary_carry = (first_bit3 | second_bit3) == 1;
                self.a = self.a & self.a;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA8 => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.b;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xA9 => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.c;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xAA => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.d;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xAB => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.e;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xAC => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.h;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xAD => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.l;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xAE => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let value_from_memory = self.memory[addr as usize] as u8;
                self.auxiliary_carry = false;
                self.a = self.a ^ value_from_memory;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xAF => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.a;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB0 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.b;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB1 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.c;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB2 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.d;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB3 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.e;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB4 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.h;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB5 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.l;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB6 => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let value_from_memory = self.memory[addr as usize] as u8;
                self.auxiliary_carry = false;
                self.a = self.a | value_from_memory;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB7 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.a;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB8 => {
                let dif = (self.a as u16).wrapping_sub(self.b as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.b);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.b);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xB9 => {
                let dif = (self.a as u16).wrapping_sub(self.c as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.c);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.c);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xBA => {
                let dif = (self.a as u16).wrapping_sub(self.d as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.d);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.d);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xBB => {
                let dif = (self.a as u16).wrapping_sub(self.e as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.e);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.e);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xBC => {
                let dif = (self.a as u16).wrapping_sub(self.h as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.h);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.h);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xBD => {
                let dif = (self.a as u16).wrapping_sub(self.l as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.l);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.l);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xBE => {
                let addr = (self.h as u16) << 8 | (self.l as u16);
                let value_from_memory = self.memory[addr as usize] as u8;
                let dif = (self.a as u16).wrapping_sub(value_from_memory as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, value_from_memory);
                self.set_carry_flag_arithmetic_subtraction(self.a, value_from_memory);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xBF => {
                let dif = (self.a as u16).wrapping_sub(self.a as u16);
                self.set_auxiliary_carry_subtraction_flag(self.a, self.a);
                self.set_carry_flag_arithmetic_subtraction(self.a, self.a);
                self.set_sign_flag(dif as u8);
                self.set_zero_flag(dif as u8);
                self.set_parity_flag(dif as u16);
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xC0 => {
                if !self.zero {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xC1 => {
                self.b = self.memory[self.stack_pointer as usize + 1];
                self.c = self.memory[self.stack_pointer as usize];

                self.program_counter = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_add(2);

                return StepInstructionResult::Ok;
            }
            0xC2 => {
                if !self.zero {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xC3 => {
                
                self.program_counter = (self.memory[self.program_counter as usize + 2] as u16) << 8
                    | self.memory[self.program_counter as usize + 1] as u16;


                return StepInstructionResult::Ok;
            }
            0xC4 => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if !self.zero {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xC5 => {
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.b;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.c;

                self.program_counter = self.program_counter.wrapping_add(1);

                return StepInstructionResult::Ok;
            }
            0xC6 => {
                let sum = self.a as u16 + self.memory[self.program_counter as usize + 1] as u16;

                self.auxiliary_carry = ((self.a & 0x0F) + (self.memory[self.program_counter as usize + 1] & 0x0F)) > 0x0F;
                self.carry = (self.a as u16 + self.memory[self.program_counter as usize + 1] as u16) > 0xFF;

                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);

                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xC7 => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0000;
                return StepInstructionResult::Ok;
            }
            0xC8 => {
                if self.zero {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xC9 => {
                self.program_counter = (self.memory[self.stack_pointer as usize + 1] as u16) << 8
                    | self.memory[self.stack_pointer as usize] as u16;

                self.stack_pointer = self.stack_pointer.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xCA => {
                if debug{
                    println!("zero flag before 0xCA: {}", self.zero);
                }
                if self.zero {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xCB => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0xCC => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if self.zero {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            // CALL opcode used by CP/M
            0xCD => {

                temp3_16 = self.program_counter.wrapping_add(3);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = (self.memory[self.program_counter as usize + 2] as u16) << 8
                    | self.memory[self.program_counter as usize + 1] as u16;
                return StepInstructionResult::Ok;
            }
            0xCE => {
                let carry_in = if self.carry { 1u8 } else { 0u8 };
                let imm = self.memory[self.program_counter as usize + 1];
                let sum = self.a as u16 + imm as u16 + carry_in as u16;
                self.auxiliary_carry = ((self.a & 0x0F) + (imm & 0x0F) + carry_in) > 0x0F;
                self.carry = sum > 0xFF;
                self.a = sum as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xCF => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0008;
                return StepInstructionResult::Ok;
            }
            0xD0 => {
                if !self.carry {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xD1 => {
                self.d = self.memory[self.stack_pointer as usize + 1];
                self.e = self.memory[self.stack_pointer as usize];

                self.program_counter = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_add(2);

                return StepInstructionResult::Ok;
            }
            0xD2 => {
                if !self.carry {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xD3 => {

                let port = self.memory[self.program_counter as usize + 1];
                if let Some(callback) = self.on_out {
                    callback(port, self.a);
                }
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xD4 => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if !self.carry {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xD5 => {
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.d;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.e;

                self.program_counter = self.program_counter.wrapping_add(1);

                return StepInstructionResult::Ok;
            }
            0xD6 => {
                let imm = self.memory[self.program_counter as usize + 1] as u16;
                let dif = (self.a as u16).wrapping_sub(imm);
                
                self.auxiliary_carry = (self.a & 0x0F) >= (imm as u8 & 0x0F);
                
                self.carry = self.a < imm as u8;
                self.a = dif as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                

                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xD7 => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0010;
                return StepInstructionResult::Ok;
            }
            0xD8 => {
                if self.carry {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xD9 => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0xDA => {
                if self.carry {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xDB => {
                // TODO ADD SPECIFIC IN BEHAVIOR (AS A GENERIC)
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xDC => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if self.carry {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xDD => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::NoOperation;
            }
            0xDE => {
                let imm = self.memory[self.program_counter as usize + 1];
                let carry_in = if self.carry { 1u8 } else { 0u8 };
                let full_borrow = (imm as u16) + (carry_in as u16);
                let dif = (self.a as u16).wrapping_sub(full_borrow);
                self.auxiliary_carry = ((self.a & 0x0F) as u16) >= ((imm & 0x0F) as u16 + carry_in as u16);
                self.carry = (self.a as u16) < full_borrow;
                self.a = dif as u8;

                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);

                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xDF => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0018;
                return StepInstructionResult::Ok;
            }
            0xE0 => {
                if !self.parity {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xE1 => {
                self.h = self.memory[self.stack_pointer as usize + 1];
                self.l = self.memory[self.stack_pointer as usize];

                self.program_counter = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_add(2);

                return StepInstructionResult::Ok;
            }
            0xE2 => {
                if !self.parity {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xE3 => {
                temp1_8 = self.h;
                temp2_8 = self.l;

                self.l = self.memory[self.stack_pointer as usize];
                self.h = self.memory[self.stack_pointer as usize + 1];
                self.memory[self.stack_pointer as usize] = temp2_8;
                self.memory[self.stack_pointer as usize + 1] = temp1_8;

                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xE4 => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if debug {
                    println!("parity flag before 0xE4: {}", self.parity);
                }
                if !self.parity {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xE5 => {
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.h;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.l;

                self.program_counter = self.program_counter.wrapping_add(1);

                return StepInstructionResult::Ok;
            }
            0xE6 => {
                let imm = self.memory[self.program_counter as usize + 1];
                
                self.auxiliary_carry = ((self.a | imm) & 0x08) != 0;
                
                self.a = self.a & imm;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xE7 => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0020;
                return StepInstructionResult::Ok;
            }
            0xE8 => {

                if self.parity {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xE9 => {
                self.program_counter = (self.h as u16) << 8 | (self.l as u16);
                return StepInstructionResult::Ok;
            }
            0xEA => {
                if self.parity {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xEB => {
                temp1_8 = self.h;
                temp2_8 = self.l;

                self.h = self.d;
                self.l = self.e;
                self.d = temp1_8;
                self.e = temp2_8;

                self.program_counter = self.program_counter.wrapping_add(1);

                return StepInstructionResult::Ok;
            }
            0xEC => {

                temp3_16 = self.program_counter.wrapping_add(3);
                if self.parity {

                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xED => {
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xEE => {
                self.auxiliary_carry = false;
                self.a = self.a ^ self.memory[self.program_counter as usize + 1] as u8;
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(2);

                return StepInstructionResult::Ok;
            }
            0xEF => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0028;
                return StepInstructionResult::Ok;
            }
            0xF0 => {
                if !self.sign {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xF1 => {
                self.a = self.memory[self.stack_pointer as usize + 1];
                let flags = self.memory[self.stack_pointer as usize];

                

                self.carry = (flags & 0b00000001) != 0;      // bit 0
                self.parity = (flags & 0b00000100) != 0;     // bit 2
                self.auxiliary_carry = (flags & 0b00010000) != 0; // bit 4
                self.zero = (flags & 0b01000000) != 0;       // bit 6
                self.sign = (flags & 0b10000000) != 0;       // bit 7

                self.program_counter = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xF2 => {
                if !self.sign {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xF3 => {
                //TODO attach intruupts and this disables them till 0xFB
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xF4 => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if !self.sign {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xF5 => {
                // Build the flags byte according to Intel 8080 format
                let mut flags = 0b00000010; // Bit 1 is always set
                
                if self.carry { flags |= 0b00000001; }      // Bit 0 - Carry
                if self.parity { flags |= 0b00000100; }     // Bit 2 - Parity
                if self.auxiliary_carry { flags |= 0b00010000; } // Bit 4 - Aux Carry
                if self.zero { flags |= 0b01000000; }       // Bit 6 - Zero
                if self.sign { flags |= 0b10000000; }       // Bit 7 - Sign
                
                // Explicitly clear reserved bits (though they should already be 0)
                //flags &= 0b11010111; // Clear bits 3 and 5 (keep them 0)
                
                // Push flags first, then accumulator
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = self.a;  // flags first
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = flags;  // A second
                
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xF6 => {
                self.auxiliary_carry = false;
                self.a = self.a | self.memory[self.program_counter as usize + 1];
                self.set_sign_flag(self.a);
                self.set_zero_flag(self.a);
                self.set_parity_flag(self.a as u16);
                self.carry = false;
                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xF7 => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0030;
                return StepInstructionResult::Ok;
            }
            0xF8 => {
                if self.sign {
                    temp1_8 = self.memory[self.stack_pointer as usize];
                    temp2_8 = self.memory[self.stack_pointer as usize + 1];
                    self.stack_pointer = self.stack_pointer.wrapping_add(2);

                    self.program_counter = (temp2_8 as u16) << 8 | temp1_8 as u16;
                    return StepInstructionResult::Ok;
                }
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xF9 => {
                self.stack_pointer = (self.h as u16) << 8 | self.l as u16;
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xFA => {
                if self.sign {
                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }

                return StepInstructionResult::Ok;
            }
            0xFB => {
                //todo!(enable interupts)
                self.program_counter = self.program_counter.wrapping_add(1);
                return StepInstructionResult::Ok;
            }
            0xFC => {
                temp3_16 = self.program_counter.wrapping_add(3);
                if self.sign {
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                    self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                    self.memory[self.stack_pointer as usize] = temp3_16 as u8;

                    self.program_counter = (self.memory[self.program_counter as usize + 2] as u16)
                        << 8
                        | self.memory[self.program_counter as usize + 1] as u16;
                } else {
                    self.program_counter = self.program_counter.wrapping_add(3);
                }
                return StepInstructionResult::Ok;
            }
            0xFD => {
                return StepInstructionResult::Ok;
            }
            0xFE => { 

                temp1_8 = self.memory[self.program_counter as usize + 1] as u8;   
                let dif = (self.a as u16).wrapping_sub(temp1_8 as u16);

                self.auxiliary_carry = (self.a & 0x0F) >= (temp1_8 as u8 & 0x0F);
                
                self.carry = self.a < temp1_8 as u8;

                self.set_sign_flag(dif as u8);

                self.set_zero_flag(dif as u8);

                self.set_parity_flag(dif as u16);

                self.program_counter = self.program_counter.wrapping_add(2);
                return StepInstructionResult::Ok;
            }
            0xFF => {
                temp3_16 = self.program_counter.wrapping_add(1);
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = (temp3_16 >> 8) as u8;
                self.stack_pointer = self.stack_pointer.wrapping_sub(1);
                self.memory[self.stack_pointer as usize] = temp3_16 as u8;
                self.program_counter = 0x0038;
                return StepInstructionResult::Ok;
            }
        }
    }
}