pub mod assembler;
pub mod challenge;
pub mod cpu;

#[cfg(target_arch = "wasm32")]
pub mod app;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use assembler::{AssembledLine, AssemblyError, AssemblyOutput, assemble};
pub use challenge::{Ibm390Challenge, Ibm390TestCase, get_all_challenges};
pub use cpu::{
    ConditionCode, Cpu, CpuError, Instruction, InstructionFormat, Opcode, Operands,
    ProgramStatusWord, execute_instruction,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_program() {
        let source = r#"
            LR R1, R2
            AR R1, R3
            HIO
        "#;

        let output = assemble(source).unwrap();
        assert_eq!(output.lines.len(), 3);

        let mut cpu = Cpu::new();
        cpu.set_gpr(2, 10).unwrap();
        cpu.set_gpr(3, 20).unwrap();

        // Load program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }
        cpu.load_program(&program).unwrap();

        // Execute LR R1, R2
        let instr = Instruction::decode(&program[0..]).unwrap();
        execute_instruction(&mut cpu, &instr).unwrap();
        cpu.increment_pc(instr.opcode.length() as u32);
        assert_eq!(cpu.get_gpr(1).unwrap(), 10);

        // Execute AR R1, R3
        let instr = Instruction::decode(&program[2..]).unwrap();
        execute_instruction(&mut cpu, &instr).unwrap();
        cpu.increment_pc(instr.opcode.length() as u32);
        assert_eq!(cpu.get_gpr(1).unwrap(), 30);

        // Execute HIO
        let instr = Instruction::decode(&program[4..]).unwrap();
        execute_instruction(&mut cpu, &instr).unwrap();
        assert!(cpu.is_halted());
    }

    #[test]
    fn test_memory_operations() {
        let source = r#"
            L R1, 0x100
            ST R1, 0x200
            HIO
        "#;

        let output = assemble(source).unwrap();
        let mut cpu = Cpu::new();

        // Set up memory
        cpu.write_word(0x100, 0x12345678).unwrap();

        // Load program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }
        cpu.load_program(&program).unwrap();

        // Execute L R1, 0x100
        let instr = Instruction::decode(&program[0..]).unwrap();
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_gpr(1).unwrap(), 0x12345678);

        // Execute ST R1, 0x200
        cpu.increment_pc(4);
        let instr = Instruction::decode(&program[4..]).unwrap();
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.read_word(0x200).unwrap(), 0x12345678);
    }

    #[test]
    fn test_condition_codes() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(1, 10).unwrap();
        cpu.set_gpr(2, 20).unwrap();

        // Compare R1, R2 (10 vs 20)
        let instr = Instruction::new_rr(Opcode::CR, 1, 2);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_cc(), ConditionCode::Low);

        // Compare R2, R1 (20 vs 10)
        let instr = Instruction::new_rr(Opcode::CR, 2, 1);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_cc(), ConditionCode::High);

        // Compare R1, R1 (10 vs 10)
        let instr = Instruction::new_rr(Opcode::CR, 1, 1);
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_cc(), ConditionCode::Zero);
    }

    #[test]
    fn test_indexed_addressing() {
        let source = r#"
            L R1, 0x10(R5, R6)
            HIO
        "#;

        let output = assemble(source).unwrap();
        let mut cpu = Cpu::new();

        // Set up base and index registers
        cpu.set_gpr(5, 0x100).unwrap(); // Base
        cpu.set_gpr(6, 0x020).unwrap(); // Index
        cpu.write_word(0x130, 0xABCDEF00).unwrap(); // 0x10 + 0x100 + 0x20 = 0x130

        // Load program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }
        cpu.load_program(&program).unwrap();

        // Execute L R1, 0x10(R5, R6)
        let instr = Instruction::decode(&program[0..]).unwrap();
        execute_instruction(&mut cpu, &instr).unwrap();
        assert_eq!(cpu.get_gpr(1).unwrap(), 0xABCDEF00);
    }
}
