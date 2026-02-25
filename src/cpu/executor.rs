use super::instruction::{Instruction, Opcode, Operands};
use super::state::{ConditionCode, Cpu, CpuError};

/// Execute a single instruction on the CPU
pub fn execute_instruction(cpu: &mut Cpu, instruction: &Instruction) -> Result<(), CpuError> {
    if cpu.is_halted() {
        return Err(CpuError::Halted);
    }

    match instruction.opcode {
        // RR Format Instructions
        Opcode::BR => execute_br(cpu, &instruction.operands),
        Opcode::LR => execute_lr(cpu, &instruction.operands),
        Opcode::AR => execute_ar(cpu, &instruction.operands),
        Opcode::SR => execute_sr(cpu, &instruction.operands),
        Opcode::MR => execute_mr(cpu, &instruction.operands),
        Opcode::DR => execute_dr(cpu, &instruction.operands),
        Opcode::CR => execute_cr(cpu, &instruction.operands),

        // RX Format Instructions
        Opcode::L => execute_l(cpu, &instruction.operands),
        Opcode::ST => execute_st(cpu, &instruction.operands),
        Opcode::A => execute_a(cpu, &instruction.operands),
        Opcode::S => execute_s(cpu, &instruction.operands),
        Opcode::M => execute_m(cpu, &instruction.operands),
        Opcode::D => execute_d(cpu, &instruction.operands),
        Opcode::C => execute_c(cpu, &instruction.operands),

        // SI Format Instructions
        Opcode::MVI => execute_mvi(cpu, &instruction.operands),
        Opcode::CLI => execute_cli(cpu, &instruction.operands),

        // S Format Instructions
        Opcode::HIO => execute_hio(cpu),

        // NOP
        Opcode::NOP => Ok(()),
    }?;

    // Increment instruction counter and cycle count
    cpu.instructions_executed += 1;
    cpu.cycles += 1;

    Ok(())
}

/// Calculate effective address for RX format: D2 + (X2 != 0 ? R[X2] : 0) + (B2 != 0 ? R[B2] : 0)
fn calculate_rx_address(cpu: &Cpu, x2: u8, b2: u8, d2: u16) -> Result<u32, CpuError> {
    let mut addr = d2 as u32;

    if x2 != 0 {
        addr = addr.wrapping_add(cpu.get_gpr(x2)?);
    }

    if b2 != 0 {
        addr = addr.wrapping_add(cpu.get_gpr(b2)?);
    }

    Ok(addr)
}

/// Calculate effective address for SI format: D1 + (B1 != 0 ? R[B1] : 0)
fn calculate_si_address(cpu: &Cpu, b1: u8, d1: u16) -> Result<u32, CpuError> {
    let mut addr = d1 as u32;

    if b1 != 0 {
        addr = addr.wrapping_add(cpu.get_gpr(b1)?);
    }

    Ok(addr)
}

/// Calculate effective address for S format: D2 + (B2 != 0 ? R[B2] : 0)
#[allow(dead_code)]
fn calculate_s_address(cpu: &Cpu, b2: u8, d2: u16) -> Result<u32, CpuError> {
    let mut addr = d2 as u32;

    if b2 != 0 {
        addr = addr.wrapping_add(cpu.get_gpr(b2)?);
    }

    Ok(addr)
}

// RR Format Instruction Implementations

/// BR: Branch Register (PC = R2)
/// Unconditional branch to address in R2
/// Note: R1 contains the mask (15 for unconditional)
fn execute_br(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1: _, r2 } = operands {
        let target_addr = cpu.get_gpr(*r2)?;
        // Set PC to target address
        // Note: PC will NOT be incremented after this instruction
        // because we're explicitly setting it to the branch target
        cpu.set_pc(target_addr);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// LR: Load Register (R1 = R2)
fn execute_lr(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1, r2 } = operands {
        let value = cpu.get_gpr(*r2)?;
        cpu.set_gpr(*r1, value)?;
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// AR: Add Register (R1 = R1 + R2)
fn execute_ar(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1, r2 } = operands {
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.get_gpr(*r2)? as i32;
        let (result, overflow) = val1.overflowing_add(val2);
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_arithmetic(result, overflow);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// SR: Subtract Register (R1 = R1 - R2)
fn execute_sr(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1, r2 } = operands {
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.get_gpr(*r2)? as i32;
        let (result, overflow) = val1.overflowing_sub(val2);
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_arithmetic(result, overflow);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// MR: Multiply Register (R1 = R1 * R2)
fn execute_mr(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1, r2 } = operands {
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.get_gpr(*r2)? as i32;
        let (result, overflow) = val1.overflowing_mul(val2);
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_arithmetic(result, overflow);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// DR: Divide Register (R1 = R1 / R2)
fn execute_dr(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1, r2 } = operands {
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.get_gpr(*r2)? as i32;

        if val2 == 0 {
            // Division by zero - set overflow condition
            cpu.set_cc(ConditionCode::Overflow);
            return Ok(());
        }

        let result = val1 / val2;
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_compare(result);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// CR: Compare Register (CC = compare R1, R2)
fn execute_cr(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RR { r1, r2 } = operands {
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.get_gpr(*r2)? as i32;
        cpu.psw.set_cc_compare(val1 - val2);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

// RX Format Instruction Implementations

/// L: Load (R1 = memory[addr])
fn execute_l(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let value = cpu.read_word(addr)?;
        cpu.set_gpr(*r1, value)?;
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// ST: Store (memory[addr] = R1)
fn execute_st(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let value = cpu.get_gpr(*r1)?;
        cpu.write_word(addr, value)?;
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// A: Add (R1 = R1 + memory[addr])
fn execute_a(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.read_word(addr)? as i32;
        let (result, overflow) = val1.overflowing_add(val2);
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_arithmetic(result, overflow);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// S: Subtract (R1 = R1 - memory[addr])
fn execute_s(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.read_word(addr)? as i32;
        let (result, overflow) = val1.overflowing_sub(val2);
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_arithmetic(result, overflow);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// M: Multiply (R1 = R1 * memory[addr])
fn execute_m(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.read_word(addr)? as i32;
        let (result, overflow) = val1.overflowing_mul(val2);
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_arithmetic(result, overflow);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// D: Divide (R1 = R1 / memory[addr])
fn execute_d(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.read_word(addr)? as i32;

        if val2 == 0 {
            cpu.set_cc(ConditionCode::Overflow);
            return Ok(());
        }

        let result = val1 / val2;
        cpu.set_gpr(*r1, result as u32)?;
        cpu.psw.set_cc_compare(result);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// C: Compare (CC = compare R1, memory[addr])
fn execute_c(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::RX { r1, x2, b2, d2 } = operands {
        let addr = calculate_rx_address(cpu, *x2, *b2, *d2)?;
        let val1 = cpu.get_gpr(*r1)? as i32;
        let val2 = cpu.read_word(addr)? as i32;
        cpu.psw.set_cc_compare(val1 - val2);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

// SI Format Instruction Implementations

/// MVI: Move Immediate (memory[addr] = immediate)
fn execute_mvi(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::SI { b1, d1, i2 } = operands {
        let addr = calculate_si_address(cpu, *b1, *d1)?;
        cpu.write_byte(addr, *i2)?;
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

/// CLI: Compare Logical Immediate (CC = compare memory[addr], immediate)
fn execute_cli(cpu: &mut Cpu, operands: &Operands) -> Result<(), CpuError> {
    if let Operands::SI { b1, d1, i2 } = operands {
        let addr = calculate_si_address(cpu, *b1, *d1)?;
        let val = cpu.read_byte(addr)?;
        let result = (val as i32) - (*i2 as i32);
        cpu.psw.set_cc_compare(result);
        Ok(())
    } else {
        Err(CpuError::InvalidInstruction(cpu.get_pc()))
    }
}

// S Format Instruction Implementations

/// HIO: Halt I/O (simplified as HALT for game)
fn execute_hio(cpu: &mut Cpu) -> Result<(), CpuError> {
    cpu.halt();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cpu::instruction::Instruction;

    #[test]
    fn test_lr() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(2, 0x12345678).unwrap();

        let instr = Instruction::new_rr(Opcode::LR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_gpr(1).unwrap(), 0x12345678);
    }

    #[test]
    fn test_ar() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 10).unwrap();
        cpu.set_gpr(2, 20).unwrap();

        let instr = Instruction::new_rr(Opcode::AR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_gpr(1).unwrap(), 30);
        assert_eq!(cpu.get_cc(), ConditionCode::High);
    }

    #[test]
    fn test_sr() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 30).unwrap();
        cpu.set_gpr(2, 10).unwrap();

        let instr = Instruction::new_rr(Opcode::SR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_gpr(1).unwrap(), 20);
        assert_eq!(cpu.get_cc(), ConditionCode::High);
    }

    #[test]
    fn test_mr() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 5).unwrap();
        cpu.set_gpr(2, 6).unwrap();

        let instr = Instruction::new_rr(Opcode::MR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_gpr(1).unwrap(), 30);
    }

    #[test]
    fn test_dr() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 20).unwrap();
        cpu.set_gpr(2, 5).unwrap();

        let instr = Instruction::new_rr(Opcode::DR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_gpr(1).unwrap(), 4);
    }

    #[test]
    fn test_dr_divide_by_zero() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 20).unwrap();
        cpu.set_gpr(2, 0).unwrap();

        let instr = Instruction::new_rr(Opcode::DR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_cc(), ConditionCode::Overflow);
    }

    #[test]
    fn test_cr() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 10).unwrap();
        cpu.set_gpr(2, 20).unwrap();

        let instr = Instruction::new_rr(Opcode::CR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.get_cc(), ConditionCode::Low);
    }

    #[test]
    fn test_l_st() {
        let mut cpu = Cpu::new();
        cpu.write_word(0x100, 0x12345678).unwrap();

        // Load from memory
        let instr = Instruction::new_rx(Opcode::L, 1, 0, 0, 0x100);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_gpr(1).unwrap(), 0x12345678);

        // Store to memory
        cpu.set_gpr(2, 0xABCDEF00).unwrap();
        let instr = Instruction::new_rx(Opcode::ST, 2, 0, 0, 0x200);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.read_word(0x200).unwrap(), 0xABCDEF00);
    }

    #[test]
    fn test_indexed_addressing() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(3, 0x100).unwrap(); // Base register
        cpu.set_gpr(4, 0x010).unwrap(); // Index register
        cpu.write_word(0x110, 0x42).unwrap(); // 0x100 + 0x010 = 0x110

        // Load with base and index
        let instr = Instruction::new_rx(Opcode::L, 1, 4, 3, 0);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_gpr(1).unwrap(), 0x42);
    }

    #[test]
    fn test_mvi() {
        let mut cpu = Cpu::new();

        let instr = Instruction::new_si(Opcode::MVI, 0, 0x100, 0x42);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.read_byte(0x100).unwrap(), 0x42);
    }

    #[test]
    fn test_cli() {
        let mut cpu = Cpu::new();
        cpu.write_byte(0x100, 50).unwrap();

        // Compare with smaller value
        let instr = Instruction::new_si(Opcode::CLI, 0, 0x100, 30);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_cc(), ConditionCode::High);

        // Compare with larger value
        let instr = Instruction::new_si(Opcode::CLI, 0, 0x100, 70);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_cc(), ConditionCode::Low);
    }

    #[test]
    fn test_hio() {
        let mut cpu = Cpu::new();
        assert!(!cpu.is_halted());

        let instr = Instruction::new_s(Opcode::HIO, 0, 0);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert!(cpu.is_halted());
    }

    #[test]
    fn test_instruction_counter() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.instructions_executed, 0);

        let instr = Instruction::new_rr(Opcode::NOP, 0, 0);
        execute_instruction(&mut cpu, &instr).unwrap();

        assert_eq!(cpu.instructions_executed, 1);
        assert_eq!(cpu.cycles, 1);
    }
}
