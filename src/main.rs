use modular_bitfield::prelude::*;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::ops::{Index, IndexMut};

type Word = u16;
type Byte = u8;
const MAX_MEM: usize = 65536;

pub struct Mem {
    data: [Byte; MAX_MEM],
}

impl Mem {
    pub fn new() -> Self {
        Mem { data: [0; MAX_MEM] }
    }

    pub fn write_word(&mut self, addr: usize, value: Word) {
        self.data[addr] = (value & 0xFF) as Byte;
        self.data[addr + 1] = ((value >> 8) & 0xFF) as Byte;
    }

    pub fn read_word(&self, addr: usize) -> Word {
        let low_byte = self.data[addr] as Word;
        let high_byte = self.data[addr + 1] as Word;
        low_byte | (high_byte << 8)
    }
}

impl Default for Mem {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<usize> for Mem {
    type Output = Byte;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Mem {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum Opcode {
    LdaIm = 0xA9,
    LdaZp = 0x45,
    LdaZpx = 0xB5,
    Jsr = 0x20,
    AdcIm = 0x69,
    AdcZp = 0x65,
    AdcZpx = 0x75,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
pub struct CpuFlags {
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal: bool,
    break_command: bool,
    unused: bool,
    overflow: bool,
    negative: bool,
}

impl Default for CpuFlags {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CPU {
    program_counter: Word,
    stack_register: Word,
    accumulator: Byte,
    index_register_x: Byte,
    index_register_y: Byte,
    flags: CpuFlags,
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            program_counter: 0xFFFC,
            stack_register: 0x0100,
            accumulator: 0,
            index_register_x: 0,
            index_register_y: 0,
            flags: CpuFlags::new(),
        }
    }
}

impl CPU {
    pub fn reset(&mut self) {
        self.program_counter = 0xFFFC;
        self.stack_register = 0x00FF;
        self.accumulator = 0;
        self.index_register_x = 0;
        self.index_register_y = 0;
        self.flags = CpuFlags::new();
    }

    pub fn execute(&mut self, memory: &mut Mem, mut cycles: u32) {
        while cycles > 0 {
            let instruction = self.fetch_byte(&mut cycles, memory);

            match Opcode::try_from(instruction) {
                Ok(Opcode::LdaIm) => {
                    self.accumulator = self.fetch_byte(&mut cycles, memory);
                    self.lda_set_status();
                }
                Ok(Opcode::LdaZp) => {
                    let addr = self.fetch_byte(&mut cycles, memory);
                    self.accumulator = self.read_byte(addr, &mut cycles, memory);
                    self.lda_set_status();
                }
                Ok(Opcode::LdaZpx) => {
                    let mut addr = self.fetch_byte(&mut cycles, memory);
                    addr += self.index_register_x;
                    cycles -= 1;
                    self.accumulator = self.read_byte(addr, &mut cycles, memory);
                    self.lda_set_status();
                }
                Ok(Opcode::AdcIm) => {
                    let value = self.fetch_byte(&mut cycles, memory);
                    self.adc(value);
                }
                Ok(Opcode::AdcZp) => {
                    let addr = self.fetch_byte(&mut cycles, memory);
                    let value = self.read_byte(addr, &mut cycles, memory);
                    self.adc(value);
                }
                Ok(Opcode::AdcZpx) => {
                    let mut addr = self.fetch_byte(&mut cycles, memory);
                    addr += self.index_register_x;
                    cycles -= 1;
                    let value = self.read_byte(addr, &mut cycles, memory);
                    self.adc(value);
                }
                Ok(Opcode::Jsr) => {
                    let sub_addr = self.fetch_word(&mut cycles, memory);

                    //TODO: return addr doesnt seem to be returning right
                    let return_addr = self.program_counter.wrapping_sub(1);

                    memory[self.stack_register as usize] = (return_addr >> 8 & 0xFF) as Byte;
                    self.stack_register -= 1;
                    cycles -= 2;

                    memory[self.stack_register as usize] = (return_addr & 0xFF) as Byte;
                    self.stack_register -= 1;
                    cycles -= 1;

                    self.program_counter = sub_addr;
                }
                Err(_) => {
                    eprintln!("Invalid instruction byte: {:02X}", instruction);
                }
            }
        }
    }

    fn fetch_byte(&mut self, cycles: &mut u32, memory: &mut Mem) -> Byte {
        let data = memory[self.program_counter as usize];
        self.program_counter += 1;
        *cycles -= 1;
        data
    }

    fn fetch_word(&mut self, cycles: &mut u32, memory: &mut Mem) -> Word {
        let mut data: Word = memory[self.program_counter as usize] as Word;
        self.program_counter += 1;
        *cycles -= 1;

        data |= (memory[self.program_counter as usize] as Word) << 8;
        self.program_counter += 1;
        *cycles -= 1;
        data
    }

    fn read_byte(&mut self, addr: Byte, cycles: &mut u32, memory: &mut Mem) -> Byte {
        *cycles -= 1;
        memory[addr as usize]
    }

    fn lda_set_status(&mut self) {
        self.flags.set_zero(self.accumulator == 0);
        self.flags
            .set_negative((self.accumulator & 0b10000000) != 0);
    }

    fn adc(&mut self, value: Byte) {
        let carry_in = if self.flags.carry() { 1 } else { 0 };
        let sum = self.accumulator as u16 + value as u16 + carry_in;
        let result = sum as u8;

        self.flags.set_carry(sum > 0xFF);
        self.flags.set_zero(result == 0);
        self.flags.set_negative(result & 0x80 != 0);
        self.flags
            .set_overflow(!(self.accumulator ^ value) & (self.accumulator ^ result) & 0x80 != 0);

        self.accumulator = result;
    }

    //getters for flags for testing
    pub fn get_carry_flag(&self) -> bool {
        self.flags.carry()
    }
    pub fn get_zero_flag(&self) -> bool {
        self.flags.zero()
    }
    pub fn get_interrupt_disable_flag(&self) -> bool {
        self.flags.interrupt_disable()
    }
    pub fn get_decimal_flag(&self) -> bool {
        self.flags.decimal()
    }
    pub fn get_break_command_flag(&self) -> bool {
        self.flags.break_command()
    }
    pub fn get_overflow_flag(&self) -> bool {
        self.flags.overflow()
    }
    pub fn get_negative_flag(&self) -> bool {
        self.flags.negative()
    }

    // setters for flags for testing
    pub fn set_carry_flag(&mut self, value: bool) {
        self.flags.set_carry(value);
    }
    pub fn set_zero_flag(&mut self, value: bool) {
        self.flags.set_zero(value);
    }
    pub fn set_interrupt_disable_flag(&mut self, value: bool) {
        self.flags.set_interrupt_disable(value);
    }
    pub fn set_decimal_flag(&mut self, value: bool) {
        self.flags.set_decimal(value);
    }
    pub fn set_break_command_flag(&mut self, value: bool) {
        self.flags.set_break_command(value);
    }
    pub fn set_overflow_flag(&mut self, value: bool) {
        self.flags.set_overflow(value);
    }
    pub fn set_negative_flag(&mut self, value: bool) {
        self.flags.set_negative(value);
    }

    //getters for CPU registers for testing
    pub fn get_stack_register(&self) -> Word {
        self.stack_register
    }
    pub fn get_program_counter(&self) -> Word {
        self.program_counter
    }
    pub fn get_accumulator(&self) -> Byte {
        self.accumulator
    }
    pub fn get_index_register_x(&self) -> Byte {
        self.index_register_x
    }
    pub fn get_index_register_y(&self) -> Byte {
        self.index_register_y
    }
    pub fn get_flags(&self) -> CpuFlags {
        self.flags
    }

    //setters for CPU registers for testing
    pub fn set_accumulator(&mut self, value: Byte) {
        self.accumulator = value;
    }

    pub fn set_index_register_x(&mut self, value: Byte) {
        self.index_register_x = value;
    }

    pub fn set_index_register_y(&mut self, value: Byte) {
        self.index_register_y = value;
    }
    pub fn set_program_counter(&mut self, value: Word) {
        self.program_counter = value;
    }
    pub fn set_stack_register(&mut self, value: Word) {
        self.stack_register = value;
    }
}

fn main() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    let cycles = 2;
    cpu.reset();
    mem[0xfffc] = Opcode::LdaIm as u8;
    mem[0xfffd] = 0x42;
    cpu.execute(&mut mem, cycles);
}
