use cpu6052::*;

#[test]
fn test_cpu_reset_flags() {
    let mut cpu = CPU::default();
    cpu.reset();

    assert!(!cpu.get_carry_flag());
    assert!(!cpu.get_zero_flag());
    assert!(!cpu.get_decimal_flag());
    assert!(!cpu.get_interrupt_disable_flag());
    assert!(!cpu.get_break_command_flag());
    assert!(!cpu.get_overflow_flag());
    assert!(!cpu.get_negative_flag());
}

#[test]
fn test_lda_im() {
    let mut memory = Mem::default();
    let mut cpu = CPU::default();

    // Set up the memory with the LDA Immediate instruction and the value to load
    memory[0xFFFC] = Opcode::LdaIm as u8; // LDA Immediate opcode
    memory[0xFFFD] = 0x42; // Value to load into the accumulator

    // Execute the instruction
    cpu.reset();
    cpu.execute(&mut memory, 2);

    // Assert that the accumulator is updated correctly
    assert_eq!(cpu.get_accumulator(), 0x42);

    // Assert that the zero flag is cleared (value is not zero)
    assert!(!cpu.get_zero_flag());

    // Assert that the negative flag is cleared (value is not negative)
    assert!(!cpu.get_negative_flag());
}

#[test]
fn test_lda_zp() {
    let mut memory = Mem::default();
    let mut cpu = CPU::default();

    // Set up the memory with the LDA Zero Page instruction and the value to load
    memory[0xFFFC] = Opcode::LdaZp as u8; // LDA Zero Page opcode
    memory[0xFFFD] = 0x10; // Zero Page address
    memory[0x0010] = 0x42; // Value to load into the accumulator

    // Execute the instruction
    cpu.reset();
    cpu.execute(&mut memory, 3);

    // Assert that the accumulator is updated correctly
    assert_eq!(cpu.get_accumulator(), 0x42);

    // Assert that the zero flag is cleared (value is not zero)
    assert!(!cpu.get_zero_flag());

    // Assert that the negative flag is cleared (value is not negative)
    assert!(!cpu.get_negative_flag());
}

#[test]
fn test_lda_zpx() {
    let mut memory = Mem::default();
    let mut cpu = CPU::default();

    // Set up the memory with the LDA Zero Page, X instruction and the value to load
    memory[0xFFFC] = Opcode::LdaZpx as u8; // LDA Zero Page, X opcode
    memory[0xFFFD] = 0x10; // Base Zero Page address
    memory[0x0015] = 0x42; // Value to load into the accumulator (0x10 + X = 0x15)

    // Execute the instruction
    cpu.reset();
    cpu.set_index_register_x(0x05);
    cpu.execute(&mut memory, 4);

    // Assert that the accumulator is updated correctly
    assert_eq!(cpu.get_accumulator(), 0x42, "Accumulator should be 0x42");

    // Assert that the zero flag is cleared (value is not zero)
    assert!(!cpu.get_zero_flag(), "Zero flag should be cleared");

    // Assert that the negative flag is cleared (value is not negative)
    assert!(!cpu.get_negative_flag(), "Negative flag should be cleared");
}

#[test]
fn test_jsr() {
    let mut memory = Mem::default();
    let mut cpu = CPU::default();

    // Set up the memory with the JSR instruction and the subroutine address
    memory[0xFFFC] = Opcode::Jsr as u8; // JSR opcode
    memory[0xFFFD] = 0x00; // Low byte of subroutine address
    memory[0xFFFE] = 0x20; // High byte of subroutine address

    // Execute the instruction
    cpu.reset();
    cpu.execute(&mut memory, 6);

    // Assert that the program counter is set to the subroutine address
    assert_eq!(
        cpu.get_program_counter(),
        0x2000,
        "Program counter should be 0x2000"
    );

    // Assert that the return address is pushed onto the stack
    let return_addr_low = memory[cpu.get_stack_register() as usize + 1];
    let return_addr_high = memory[cpu.get_stack_register() as usize + 2];
    let return_addr = ((return_addr_high as u16) << 8) | return_addr_low as u16;
    assert_eq!(return_addr, 0xFFFD, "Return address should be 0xFFFD");

    // Assert that the stack pointer is decremented correctly
    assert_eq!(
        cpu.get_stack_register(),
        0x00FD,
        "Stack pointer should be 0x00FD"
    );
}
