use modular_bitfield::prelude::*;
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

#[bitfield]
#[derive(Debug, Clone, Copy)]
struct cpu_flags {
    carry: bool,
    zero: bool,
    disable_inter: bool,
    decimal: bool,
    break_func: bool,
    overflow: bool,
    negative: bool,
    #[skip]
    __: B1,
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

    fn fetch_byte(&mut self, memory: &mut Mem) -> byte {
        let data = memory[self.program_counter as usize];
        self.program_counter += 1;
        data
    }

    const INS_LDA_IM: byte = 0xA9;

    fn execute(&mut self, memory: &mut Mem, cycles: u32) {
        for _ in 0..cycles {
            let instruction = self.fetch_byte(memory);

            match instruction {
                CPU::INS_LDA_IM => {
                    self.accumulator = self.fetch_byte(memory);
                    self.flags.set_zero(self.accumulator == 0);
                    self.flags.set_negative((self.accumulator & 0x80) != 0);
                }
                _ => eprintln!("instruction doesn't exist"),
            }
        }
    }
}

fn main() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();
    cpu.execute(&mut mem, 2);
}
