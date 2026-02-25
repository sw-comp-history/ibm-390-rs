use crate::cpu::Cpu;
use asm_game_shared::{Challenge, ChallengeableCpu, Difficulty};
use serde::{Deserialize, Serialize};

/// Test case specific to IBM ESA/390 architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ibm390TestCase {
    pub name: String,

    /// Expected GPR values (register_number, value)
    #[serde(default)]
    pub expected_gprs: Vec<(u8, u32)>,

    /// Expected memory values (address, value)
    #[serde(default)]
    pub expected_memory: Vec<(u32, u32)>,

    /// Expected condition code
    pub expected_cc: Option<u8>,
}

/// Implement ChallengeableCpu trait for IBM 390 CPU
impl ChallengeableCpu for Cpu {
    type TestCase = Ibm390TestCase;

    fn validate_test_case(&self, test_case: &Self::TestCase) -> Result<(), String> {
        // Check GPRs
        for (reg, expected) in &test_case.expected_gprs {
            let actual = self
                .get_gpr(*reg)
                .map_err(|e| format!("Invalid register R{}: {}", reg, e))?;

            if actual != *expected {
                return Err(format!(
                    "R{} mismatch: expected 0x{:08X} ({}), got 0x{:08X} ({})",
                    reg, expected, *expected as i32, actual, actual as i32
                ));
            }
        }

        // Check memory
        for (addr, expected) in &test_case.expected_memory {
            let actual = self
                .read_word(*addr)
                .map_err(|e| format!("Invalid memory address 0x{:08X}: {}", addr, e))?;

            if actual != *expected {
                return Err(format!(
                    "Memory[0x{:08X}] mismatch: expected 0x{:08X} ({}), got 0x{:08X} ({})",
                    addr, expected, *expected as i32, actual, actual as i32
                ));
            }
        }

        // Check condition code
        if let Some(expected_cc) = test_case.expected_cc {
            let actual_cc = self.get_cc() as u8;
            if actual_cc != expected_cc {
                return Err(format!(
                    "Condition code mismatch: expected {}, got {}",
                    expected_cc, actual_cc
                ));
            }
        }

        Ok(())
    }

    fn get_cycles(&self) -> u64 {
        self.cycles
    }

    fn get_instructions(&self) -> u64 {
        self.instructions_executed
    }

    fn is_halted(&self) -> bool {
        self.halted
    }
}

/// Type alias for IBM 390 challenges
pub type Ibm390Challenge = Challenge<Ibm390TestCase>;

/// Get all available challenges for IBM 390
pub fn get_all_challenges() -> Vec<Ibm390Challenge> {
    vec![
        challenge_1_load_value(),
        challenge_2_add_numbers(),
        challenge_3_multiply(),
    ]
}

/// Challenge 1: Load a Value
fn challenge_1_load_value() -> Ibm390Challenge {
    Challenge::new(
        1,
        "Load a Value",
        "Load the value at memory address 0x100 into register R1, then halt.",
        Difficulty::Beginner,
        50,
    )
    .with_test_case(Ibm390TestCase {
        name: "Load value 42".to_string(),
        expected_gprs: vec![(1, 42)],
        expected_memory: vec![],
        expected_cc: None,
    })
    .with_hint("Use the L (Load) instruction")
    .with_hint("The L instruction format is: L R1, D2(X2,B2)")
    .with_hint("For a simple load from address 0x100, use: L R1, 0x100")
    .with_hint("Don't forget to halt with HIO")
    .with_learning_objective("Understand the RX instruction format")
    .with_learning_objective("Learn how to load data from memory into registers")
}

/// Challenge 2: Add Two Numbers
fn challenge_2_add_numbers() -> Ibm390Challenge {
    Challenge::new(
        2,
        "Add Two Numbers",
        "Load values from addresses 0x100 and 0x104, add them, store result at 0x108, then halt.",
        Difficulty::Beginner,
        100,
    )
    .with_test_case(Ibm390TestCase {
        name: "Add 15 + 27 = 42".to_string(),
        expected_gprs: vec![],
        expected_memory: vec![(0x108, 42)],
        expected_cc: None,
    })
    .with_hint("Use L to load the first number into R1")
    .with_hint("Use A to add the second number to R1")
    .with_hint("Use ST to store R1 to memory")
    .with_hint("L R1, 0x100  ; Load first number")
    .with_hint("A R1, 0x104  ; Add second number")
    .with_hint("ST R1, 0x108 ; Store result")
    .with_learning_objective("Learn arithmetic operations in memory")
    .with_learning_objective("Understand the A (Add) instruction")
    .with_learning_objective("Practice storing results to memory")
}

/// Challenge 3: Multiply Two Numbers
fn challenge_3_multiply() -> Ibm390Challenge {
    Challenge::new(
        3,
        "Multiply Two Numbers",
        "Load values from addresses 0x100 and 0x104, multiply them, store result at 0x108, then halt.",
        Difficulty::Intermediate,
        150,
    )
    .with_test_case(Ibm390TestCase {
        name: "Multiply 6 * 7 = 42".to_string(),
        expected_gprs: vec![],
        expected_memory: vec![(0x108, 42)],
        expected_cc: None,
    })
    .with_hint("Use L to load the first number")
    .with_hint("Use M to multiply with the second number")
    .with_hint("Use ST to store the result")
    .with_hint("L R1, 0x100  ; Load first number")
    .with_hint("M R1, 0x104  ; Multiply by second number")
    .with_hint("ST R1, 0x108 ; Store result")
    .with_learning_objective("Learn multiplication operations")
    .with_learning_objective("Understand the M (Multiply) instruction")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembler::assemble;
    use crate::cpu::Instruction;
    use crate::cpu::execute_instruction;

    #[test]
    fn test_challenge_1_solution() {
        let source = r#"
            L R1, 0x100
            HIO
        "#;

        let output = assemble(source).unwrap();
        let mut cpu = Cpu::new();

        // Set up test data
        cpu.write_word(0x100, 42).unwrap();

        // Load and execute program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }
        cpu.load_program(&program).unwrap();

        // Run until halt
        while !cpu.is_halted() {
            let instr = Instruction::decode(&program[cpu.get_pc() as usize..]).unwrap();
            execute_instruction(&mut cpu, &instr).unwrap();
            cpu.increment_pc(instr.opcode.length() as u32);
        }

        // Validate
        let challenge = challenge_1_load_value();
        let result = challenge.validate_solution(&cpu).unwrap();
        assert!(result.passed, "Challenge 1 should pass: {}", result.message);
    }

    #[test]
    fn test_challenge_2_solution() {
        let source = r#"
            L R1, 0x100
            A R1, 0x104
            ST R1, 0x108
            HIO
        "#;

        let output = assemble(source).unwrap();
        let mut cpu = Cpu::new();

        // Set up test data
        cpu.write_word(0x100, 15).unwrap();
        cpu.write_word(0x104, 27).unwrap();

        // Load and execute program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }
        cpu.load_program(&program).unwrap();

        // Run until halt
        while !cpu.is_halted() {
            let instr = Instruction::decode(&program[cpu.get_pc() as usize..]).unwrap();
            execute_instruction(&mut cpu, &instr).unwrap();
            cpu.increment_pc(instr.opcode.length() as u32);
        }

        // Validate
        let challenge = challenge_2_add_numbers();
        let result = challenge.validate_solution(&cpu).unwrap();
        assert!(result.passed, "Challenge 2 should pass: {}", result.message);
    }

    #[test]
    fn test_challenge_3_solution() {
        let source = r#"
            L R1, 0x100
            M R1, 0x104
            ST R1, 0x108
            HIO
        "#;

        let output = assemble(source).unwrap();
        let mut cpu = Cpu::new();

        // Set up test data
        cpu.write_word(0x100, 6).unwrap();
        cpu.write_word(0x104, 7).unwrap();

        // Load and execute program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }
        cpu.load_program(&program).unwrap();

        // Run until halt
        while !cpu.is_halted() {
            let instr = Instruction::decode(&program[cpu.get_pc() as usize..]).unwrap();
            execute_instruction(&mut cpu, &instr).unwrap();
            cpu.increment_pc(instr.opcode.length() as u32);
        }

        // Validate
        let challenge = challenge_3_multiply();
        let result = challenge.validate_solution(&cpu).unwrap();
        assert!(result.passed, "Challenge 3 should pass: {}", result.message);
    }
}
