use cpu6052::*;

#[test]
fn test_cpu_reset_flags() {
    let mut cpu = CPU::default();
    cpu.reset();

    let flags = cpu.get_flags();
    assert!(!flags.carry);
    assert!(!flags.zero);
    assert!(!flags.interrupt_disable);
    assert!(!flags.decimal_mode);
    assert!(!flags.break_command);
    assert!(!flags.overflow);
    assert!(!flags.negative);
}

#[test]
fn test_cpu_sta_absolute() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::StaAbs as u8;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;

    cpu.set_accumulator(0x55);
    cpu.execute(&mut mem, 4);

    assert_eq!(mem[0x2000], 0x55);
}

#[test]
fn test_cpu_inx() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::Inx as u8;

    cpu.set_index_register_x(0xFE);
    cpu.execute(&mut mem, 2);

    assert_eq!(cpu.get_index_register_x(), 0xFF);
    assert!(!cpu.get_zero_flag());
    assert!(cpu.get_negative_flag());

    cpu.execute(&mut mem, 2);

    assert_eq!(cpu.get_index_register_x(), 0x00);
    assert!(cpu.get_zero_flag());
    assert!(!cpu.get_negative_flag());
}

#[test]
fn test_cpu_bne() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::Bne as u8;
    mem[0xFFFD] = 0x02;

    cpu.set_zero_flag(false);
    cpu.execute(&mut mem, 2);

    assert_eq!(cpu.get_program_counter(), 0xFFFE);

    cpu.set_zero_flag(true);
    cpu.reset();
    cpu.execute(&mut mem, 2);

    assert_eq!(cpu.get_program_counter(), 0xFFFD);
}

#[test]
fn test_cpu_lda_zero_flag() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::LdaIm as u8;
    mem[0xFFFD] = 0x00;

    cpu.execute(&mut mem, 2);

    assert_eq!(cpu.get_accumulator(), 0x00);
    assert!(cpu.get_zero_flag());
    assert!(!cpu.get_negative_flag());
}

#[test]
fn test_cpu_adc_with_carry() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();

    mem[0xFFFC] = Opcode::AdcIm as u8;
    mem[0xFFFD] = 0x10;

    cpu.set_accumulator(0xF0);
    cpu.set_carry_flag(true);

    cpu.execute(&mut mem, 2);

    assert_eq!(cpu.get_accumulator(), 0x01);
    assert!(cpu.get_carry_flag());
    assert!(!cpu.get_zero_flag());
    assert!(cpu.get_negative_flag());
}

#[test]
fn test_cpu_jsr() {
    let mut mem = Mem::new();
    let mut cpu = CPU::default();
    cpu.reset();
    mem[0xFFFC] = Opcode::Jsr as u8;
    mem[0xFFFD] = 0x00;
    mem[0xFFFE] = 0x20;
    let initial_stack = cpu.get_stack_register();
    cpu.execute(&mut mem, 6);

    // Check program counter is updated
    assert_eq!(cpu.get_program_counter(), 0x2000);

    // Check stack has been updated correctly
    assert_eq!(cpu.get_stack_register(), initial_stack - 2);

    // Check return address was stored correctly (low byte at stack+1, high byte at stack+2)
    assert_eq!(mem[(initial_stack) as usize], 0xFE); // Low byte
    assert_eq!(mem[(initial_stack - 1) as usize], 0xFF); // High byte
}
