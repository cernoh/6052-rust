use modular_bitfield::prelude::*;
use num_enum::TryFromPrimitive;
use std::convert::TryFrom;
use std::ops::Index;
use std::ops::IndexMut;

type word = u16;
type byte = u8;
const MAX_MEM: usize = 65536;

struct Mem {
    data: [byte; MAX_MEM],
}

impl Mem {
    fn new() -> Self {
        Mem { data: [0; MAX_MEM] }
    }

    fn write_word(&mut self, addr: usize, value: word) {
        self.data[addr] = (value & 0xFF) as byte;
        self.data[addr + 1] = ((value >> 8) & 0xFF) as byte;
    }

    fn read_word(&self, addr: usize) -> word {
        let low_byte = self.data[addr] as word;
        let high_byte = self.data[addr + 1] as word;
        low_byte | (high_byte << 8)
    }
}

impl Index<usize> for Mem {
    type Output = byte;

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
enum Opcode {
    LDA_IM = 0xA9,
    LDA_ZP = 0x45,
    LDA_ZPX = 0xB5,
    JSR = 0x20,
    ADC_IM = 0x69,
    ADC_ZP = 0x65,
    ADC_ZPX = 0x75,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
struct cpu_flags {
    carry: bool,
    zero: bool,
    interrupt_disable: bool,
    decimal: bool,
    break_command: bool,
    unused: bool,
    overflow: bool,
    negative: bool,
}

struct CPU {
    program_counter: word,
    stack_register: word,

    accumulator: byte,
    index_register_x: byte,
    index_register_y: byte,
    flags: cpu_flags,
}

impl Default for CPU {
    fn default() -> Self {
        CPU {
            program_counter: (0xFFFC),
            stack_register: (0x0100),
            accumulator: (0),
            index_register_x: (0),
            index_register_y: (0),
            flags: cpu_flags::new(),
        }
    }
}

impl CPU {
    fn reset(&mut self) {
        self.program_counter = 0xFFFC;
        self.stack_register = 0x00FF;
        self.accumulator = 0;
        self.index_register_x = 0;
        self.index_register_y = 0;
        self.flags = cpu_flags::new();
    }

    fn fetch_byte(&mut self, cycles: &mut u32, memory: &mut Mem) -> byte {
        let data = memory[self.program_counter as usize];
        self.program_counter += 1;
        *cycles -= 1;
        data
    }

    fn read_byte(&mut self, addr: byte, cycles: &mut u32, memory: &mut Mem) -> byte {
        *cycles -= 1;
        memory[addr as usize]
    }

    fn write_byte(&mut self, value: byte, addr: byte, cycles: &mut u32, memory: &mut Mem) {
        *cycles -= 1;
        memory[addr as usize] = value;
    }

    fn write_word(&mut self, value: word, addr: word, cycles: &mut u32, memory: &mut Mem) {
        if (addr + 1) as usize >= MAX_MEM {
            panic!("Memory access out of bounds at address {}", addr);
        }
        self.write_byte((value & 0xFF) as byte, addr as byte, cycles, memory);
        self.write_byte(
            ((value >> 8) & 0xFF) as byte,
            (addr + 1) as byte,
            cycles,
            memory,
        );
    }

    fn fetch_word(&mut self, cycles: &mut u32, memory: &mut Mem) -> word {
        let mut data: word = memory[self.program_counter as usize] as word;
        self.program_counter += 1;
        *cycles -= 1;

        data |= (memory[self.program_counter as usize] as word) << 8;
        self.program_counter += 1;
        *cycles -= 1;
        data
    }

    //opcode abstractions

    fn adc(&mut self, value: byte) {
        let carry_in = if self.flags.carry() { 1 } else { 0 };
        let mut sum = self.accumulator as u16 + value as u16 + carry_in;
        let result = sum as u8;

        self.adc_set_status(&mut sum, value);
        self.accumulator = result;
    }

    //opcode set status
    fn lda_set_status(&mut self) {
        self.flags.set_zero(self.accumulator == 0);
        self.flags
            .set_negative((self.accumulator & 0b10000000) != 0);
    }

    fn adc_set_status(&mut self, sum: &mut word, value: byte) {
        let result = *sum as byte;
        self.flags.set_carry(*sum > 0xFF);
        self.flags.set_zero(result == 0);
        self.flags.set_negative(result & 0x80 != 0);
        self.flags
            .set_overflow(!(self.accumulator ^ value) & (self.accumulator ^ result) & 0x80 != 0);
    }

    fn execute(&mut self, memory: &mut Mem, mut cycles: u32) {
        while cycles > 0 {
            let instruction = self.fetch_byte(&mut cycles, memory);

            match Opcode::try_from(instruction) {
                Ok(Opcode::LDA_IM) => {
                    self.accumulator = self.fetch_byte(&mut cycles, memory);
                    self.lda_set_status();
                }
                Ok(Opcode::LDA_ZP) => {
                    let addr = self.fetch_byte(&mut cycles, memory);
                    self.accumulator = self.read_byte(addr, &mut cycles, memory);
                    self.lda_set_status();
                }
                Ok(Opcode::LDA_ZPX) => {
                    let mut addr = self.fetch_byte(&mut cycles, memory);
                    addr += self.index_register_x;
                    cycles -= 1;
                    self.accumulator = self.read_byte(addr, &mut cycles, memory);
                    self.lda_set_status();
                }

                Ok(Opcode::ADC_IM) => {
                    let value = self.fetch_byte(&mut cycles, memory);
                    self.adc(value);
                }
                Ok(Opcode::ADC_ZP) => {
                    let addr = self.fetch_byte(&mut cycles, memory);
                    let value = self.read_byte(addr, &mut cycles, memory);
                    self.adc(value);
                }
                Ok(Opcode::ADC_ZPX) => {
                    let mut addr = self.fetch_byte(&mut cycles, memory);
                    addr += self.index_register_x;
                    cycles -= 1;
                    let value = self.read_byte(addr, &mut cycles, memory);
                    self.adc(value);
                }
                Ok(Opcode::JSR) => {
                    let sub_addr = self.fetch_word(&mut cycles, memory);

                    let return_addr = self.program_counter - 1;

                    //high byte
                    memory[self.stack_register as usize] = (return_addr >> 8 & 0xFF) as byte;
                    self.stack_register -= 1;
                    cycles -= 2;

                    //low byte
                    memory[self.stack_register as usize] = (return_addr & 0xFF) as byte;
                    self.stack_register -= 1;
                    cycles -= 2;

                    self.program_counter = sub_addr;
                }
                Ok(op) => {
                    eprintln!("Unimplemented instruction {:?}", op)
                }
                Err(_) => {
                    eprintln!("Invalid instruction byte: {:02X}", instruction);
                }
            }
        }
    }
}

fn main() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    let cycles = 2;
    cpu.reset();
    mem[0xfffc] = Opcode::LDA_IM as u8;
    mem[0xfffd] = 0x42;
    cpu.execute(&mut mem, cycles);
}
