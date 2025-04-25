use crate::main::{CPU, Mem, Opcode};


use modular_bitfield::prelude::*;

#[test]
fn test_lda_immediate() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::LDA_IM as u8;
    mem[0xFFFD] = 0x42; // Load value 0x42 into the accumulator

    let cycles = 2;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x42);
    assert!(cpu.flags.zero() == false);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_lda_zero_flag() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::LDA_IM as u8;
    mem[0xFFFD] = 0x00; // Load value 0x00 into the accumulator

    let cycles = 2;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x00);
    assert!(cpu.flags.zero() == true);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_adc_immediate() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.accumulator = 0x10;
    mem[0xFFFC] = Opcode::ADC_IM as u8;
    mem[0xFFFD] = 0x20; // Add 0x20 to the accumulator

    let cycles = 2;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x30);
    assert!(cpu.flags.carry() == false);
    assert!(cpu.flags.zero() == false);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_adc_with_carry() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.accumulator = 0xFF;
    mem[0xFFFC] = Opcode::ADC_IM as u8;
    mem[0xFFFD] = 0x01; // Add 0x01 to the accumulator

    let cycles = 2;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x00); // Result wraps around
    assert!(cpu.flags.carry() == true);
    assert!(cpu.flags.zero() == true);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_jsr() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::JSR as u8;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20; // Jump to address 0x2000

    let cycles = 6;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.program_counter, 0x2000);
    assert_eq!(mem[cpu.stack_register as usize + 1], 0xFD); // Check return address pushed to stack
    assert_eq!(mem[cpu.stack_register as usize + 2], 0xFF);
}

#[test]
fn test_lda_zero_page() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::LDA_ZP as u8;
    mem[0xFFFD] = 0x10; // Load value from zero-page address 0x10
    mem[0x0010] = 0x42;

    let cycles = 3;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x42);
    assert!(cpu.flags.zero() == false);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_lda_zero_page_x() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.index_register_x = 0x01;
    mem[0xFFFC] = Opcode::LDA_ZPX as u8;
    mem[0xFFFD] = 0x10; // Load value from zero-page address 0x10 + X
    mem[0x0011] = 0x42;

    let cycles = 4;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x42);
    assert!(cpu.flags.zero() == false);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_adc_zero_page() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.accumulator = 0x10;
    mem[0xFFFC] = Opcode::ADC_ZP as u8;
    mem[0xFFFD] = 0x10; // Add value from zero-page address 0x10
    mem[0x0010] = 0x20;

    let cycles = 3;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x30);
    assert!(cpu.flags.carry() == false);
    assert!(cpu.flags.zero() == false);
    assert!(cpu.flags.negative() == false);
}

#[test]
fn test_adc_zero_page_x() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    cpu.accumulator = 0x10;
    cpu.index_register_x = 0x01;
    mem[0xFFFC] = Opcode::ADC_ZPX as u8;
    mem[0xFFFD] = 0x10; // Add value from zero-page address 0x10 + X
    mem[0x0011] = 0x20;

    let cycles = 4;
    cpu.execute(&mut mem, cycles);

    assert_eq!(cpu.accumulator, 0x30);
    assert!(cpu.flags.carry() == false);
    assert!(cpu.flags.zero() == false);
    assert!(cpu.flags.negative() == false);
}