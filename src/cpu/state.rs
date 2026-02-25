use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// CPU execution errors
#[derive(Debug, Error)]
pub enum CpuError {
    #[error("Memory access out of bounds: address {0:#08x}")]
    MemoryOutOfBounds(u32),
    #[error("Invalid register number: {0}")]
    InvalidRegister(u8),
    #[error("Invalid instruction at address {0:#08x}")]
    InvalidInstruction(u32),
    #[error("Program halted")]
    Halted,
    #[error("Privileged operation attempted")]
    PrivilegedOperation,
}

/// Condition code values (2 bits in PSW)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ConditionCode {
    #[default]
    Zero = 0, // Result is zero
    Low = 1,      // Result is less than zero
    High = 2,     // Result is greater than zero
    Overflow = 3, // Overflow occurred
}

impl From<u8> for ConditionCode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => ConditionCode::Zero,
            1 => ConditionCode::Low,
            2 => ConditionCode::High,
            3 => ConditionCode::Overflow,
            _ => unreachable!(),
        }
    }
}

/// Program Status Word (simplified for educational purposes)
/// In real ESA/390, PSW is 64 bits with many fields
/// We implement the lower 32-bit half with essential fields:
/// - Bit 0: Addressing mode (0=24-bit, 1=31-bit)
/// - Bits 1-7: Reserved/flags
/// - Bits 8-9: Condition Code
/// - Bits 10-30: Reserved
/// - Bit 31: Wait state
/// - Lower bits 0-30: Instruction Address (24 or 31 bits depending on mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramStatusWord {
    /// Condition code (2 bits, bits 8-9 of PSW)
    pub condition_code: ConditionCode,
    /// Instruction address (24 or 31 bits depending on addressing mode)
    pub instruction_address: u32,
    /// Addressing mode: false = 24-bit, true = 31-bit
    pub addressing_mode_31bit: bool,
    /// Wait state flag
    pub wait: bool,
}

impl Default for ProgramStatusWord {
    fn default() -> Self {
        // ESA/390 initial PSW: 0x000A0000 00000000
        // Upper: 0x000A0000 (bit 12=1, bit 14=1 for wait)
        // Lower: 0x00000000 (24-bit mode, IA=0)
        Self {
            condition_code: ConditionCode::Zero,
            instruction_address: 0,
            addressing_mode_31bit: false, // Start in 24-bit mode
            wait: true,                   // Wait state set until program loaded
        }
    }
}

impl ProgramStatusWord {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set condition code based on signed comparison
    pub fn set_cc_compare(&mut self, result: i32) {
        self.condition_code = if result == 0 {
            ConditionCode::Zero
        } else if result < 0 {
            ConditionCode::Low
        } else {
            ConditionCode::High
        };
    }

    /// Set condition code based on arithmetic result
    pub fn set_cc_arithmetic(&mut self, result: i32, overflow: bool) {
        if overflow {
            self.condition_code = ConditionCode::Overflow;
        } else {
            self.set_cc_compare(result);
        }
    }
}

/// IBM ESA/390 CPU state (simplified for educational purposes)
/// Full ESA/390 has 16 GPRs, 16 ARs, 16 CRs, etc.
/// We implement all 16 GPRs for accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cpu {
    /// General Purpose Registers (16 x 32-bit)
    /// R0-R15 (full ESA/390 specification)
    pub gprs: [u32; 16],

    /// Program Status Word
    pub psw: ProgramStatusWord,

    /// Main memory (64KB for educational purposes)
    /// Real ESA/390 supports up to 2GB
    #[serde(with = "serde_bytes_array")]
    pub memory: Vec<u8>,

    /// Cycle counter for performance tracking
    pub cycles: u64,

    /// Instruction counter
    pub instructions_executed: u64,

    /// Halted flag
    pub halted: bool,
}

/// Custom serialization for memory to handle large arrays efficiently
mod serde_bytes_array {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        data.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Vec::<u8>::deserialize(deserializer)
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    /// Memory size (2MB for educational purposes)
    /// Real ESA/390 supports up to 2GB per address space
    pub const MEMORY_SIZE: usize = 2 * 1024 * 1024; // 2MB

    /// Program start address (1M)
    /// Page zero (0x00000000) is special in S/390, so programs start at 1M
    pub const PROGRAM_START_ADDRESS: u32 = 0x00100000;

    /// Create a new CPU with initialized state
    pub fn new() -> Self {
        Self {
            gprs: [0; 16],
            psw: ProgramStatusWord::new(),
            memory: vec![0; Self::MEMORY_SIZE],
            cycles: 0,
            instructions_executed: 0,
            halted: false,
        }
    }

    /// Reset CPU to initial state
    /// Note: Does NOT clear memory, allowing program to be re-run
    pub fn reset(&mut self) {
        self.gprs = [0; 16];
        self.psw = ProgramStatusWord::new();
        // Memory is NOT cleared - program remains loaded
        self.cycles = 0;
        self.instructions_executed = 0;
        self.halted = false;
    }

    /// Get a GPR value
    pub fn get_gpr(&self, reg: u8) -> Result<u32, CpuError> {
        if reg >= 16 {
            return Err(CpuError::InvalidRegister(reg));
        }
        Ok(self.gprs[reg as usize])
    }

    /// Set a GPR value
    pub fn set_gpr(&mut self, reg: u8, value: u32) -> Result<(), CpuError> {
        if reg >= 16 {
            return Err(CpuError::InvalidRegister(reg));
        }
        self.gprs[reg as usize] = value;
        Ok(())
    }

    /// Read a byte from memory
    pub fn read_byte(&self, addr: u32) -> Result<u8, CpuError> {
        let addr_usize = addr as usize;
        if addr_usize >= self.memory.len() {
            return Err(CpuError::MemoryOutOfBounds(addr));
        }
        Ok(self.memory[addr_usize])
    }

    /// Write a byte to memory
    pub fn write_byte(&mut self, addr: u32, value: u8) -> Result<(), CpuError> {
        let addr_usize = addr as usize;
        if addr_usize >= self.memory.len() {
            return Err(CpuError::MemoryOutOfBounds(addr));
        }
        self.memory[addr_usize] = value;
        Ok(())
    }

    /// Read a halfword (16-bit) from memory (big-endian)
    pub fn read_halfword(&self, addr: u32) -> Result<u16, CpuError> {
        let high = self.read_byte(addr)? as u16;
        let low = self.read_byte(addr.wrapping_add(1))? as u16;
        Ok((high << 8) | low)
    }

    /// Write a halfword (16-bit) to memory (big-endian)
    pub fn write_halfword(&mut self, addr: u32, value: u16) -> Result<(), CpuError> {
        self.write_byte(addr, (value >> 8) as u8)?;
        self.write_byte(addr.wrapping_add(1), value as u8)?;
        Ok(())
    }

    /// Read a word (32-bit) from memory (big-endian)
    pub fn read_word(&self, addr: u32) -> Result<u32, CpuError> {
        let high = self.read_halfword(addr)? as u32;
        let low = self.read_halfword(addr.wrapping_add(2))? as u32;
        Ok((high << 16) | low)
    }

    /// Write a word (32-bit) to memory (big-endian)
    pub fn write_word(&mut self, addr: u32, value: u32) -> Result<(), CpuError> {
        self.write_halfword(addr, (value >> 16) as u16)?;
        self.write_halfword(addr.wrapping_add(2), value as u16)?;
        Ok(())
    }

    /// Load program into memory starting at PROGRAM_START_ADDRESS (1M)
    pub fn load_program(&mut self, program: &[u8]) -> Result<(), CpuError> {
        let start_addr = Self::PROGRAM_START_ADDRESS as usize;
        let end_addr = start_addr + program.len();

        if end_addr > self.memory.len() {
            return Err(CpuError::MemoryOutOfBounds(end_addr as u32));
        }

        self.memory[start_addr..end_addr].copy_from_slice(program);
        Ok(())
    }

    /// Get current instruction address
    pub fn get_pc(&self) -> u32 {
        self.psw.instruction_address
    }

    /// Set instruction address
    pub fn set_pc(&mut self, addr: u32) {
        self.psw.instruction_address = addr;
    }

    /// Increment instruction address by offset
    pub fn increment_pc(&mut self, offset: u32) {
        self.psw.instruction_address = self.psw.instruction_address.wrapping_add(offset);
    }

    /// Get condition code
    pub fn get_cc(&self) -> ConditionCode {
        self.psw.condition_code
    }

    /// Set condition code
    pub fn set_cc(&mut self, cc: ConditionCode) {
        self.psw.condition_code = cc;
    }

    /// Halt the CPU
    pub fn halt(&mut self) {
        self.halted = true;
        self.psw.wait = true;
    }

    /// Check if CPU is halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "IBM ESA/390 CPU State:")?;
        writeln!(f, "  PC: {:#06x}", self.psw.instruction_address)?;
        writeln!(f, "  CC: {:?}", self.psw.condition_code)?;
        writeln!(f, "  Cycles: {}", self.cycles)?;
        writeln!(f, "  Instructions: {}", self.instructions_executed)?;
        writeln!(f, "  Halted: {}", self.halted)?;
        writeln!(f, "  General Purpose Registers:")?;
        for (i, &value) in self.gprs.iter().enumerate() {
            writeln!(f, "    R{}: {:#010x} ({})", i, value, value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_creation() {
        let cpu = Cpu::new();
        assert_eq!(cpu.gprs, [0; 16]);
        assert_eq!(cpu.psw.instruction_address, 0);
        assert_eq!(cpu.cycles, 0);
        assert!(!cpu.halted);
    }

    #[test]
    fn test_gpr_access() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.get_gpr(0).unwrap(), 0);

        cpu.set_gpr(3, 0x12345678).unwrap();
        assert_eq!(cpu.get_gpr(3).unwrap(), 0x12345678);

        // Test all 16 registers are accessible
        cpu.set_gpr(15, 0xABCDEF).unwrap();
        assert_eq!(cpu.get_gpr(15).unwrap(), 0xABCDEF);

        // Register 16 is out of bounds
        assert!(cpu.get_gpr(16).is_err());
        assert!(cpu.set_gpr(16, 0).is_err());
    }

    #[test]
    fn test_memory_byte_access() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.read_byte(0x100).unwrap(), 0);

        cpu.write_byte(0x100, 0xAB).unwrap();
        assert_eq!(cpu.read_byte(0x100).unwrap(), 0xAB);

        assert!(cpu.read_byte(0xFFFF).is_ok());
        assert!(cpu.write_byte(0xFFFF, 0).is_ok());
    }

    #[test]
    fn test_memory_halfword_access() {
        let mut cpu = Cpu::new();
        cpu.write_halfword(0x100, 0x1234).unwrap();
        assert_eq!(cpu.read_halfword(0x100).unwrap(), 0x1234);
        assert_eq!(cpu.read_byte(0x100).unwrap(), 0x12);
        assert_eq!(cpu.read_byte(0x101).unwrap(), 0x34);
    }

    #[test]
    fn test_memory_word_access() {
        let mut cpu = Cpu::new();
        cpu.write_word(0x100, 0x12345678).unwrap();
        assert_eq!(cpu.read_word(0x100).unwrap(), 0x12345678);
        assert_eq!(cpu.read_halfword(0x100).unwrap(), 0x1234);
        assert_eq!(cpu.read_halfword(0x102).unwrap(), 0x5678);
    }

    #[test]
    fn test_program_loading() {
        let mut cpu = Cpu::new();
        let program = vec![0x01, 0x02, 0x03, 0x04];
        cpu.load_program(&program).unwrap();
        // Program is loaded at PROGRAM_START_ADDRESS (1M)
        let base = Cpu::PROGRAM_START_ADDRESS;
        assert_eq!(cpu.read_byte(base).unwrap(), 0x01);
        assert_eq!(cpu.read_byte(base + 1).unwrap(), 0x02);
        assert_eq!(cpu.read_byte(base + 2).unwrap(), 0x03);
        assert_eq!(cpu.read_byte(base + 3).unwrap(), 0x04);
    }

    #[test]
    fn test_condition_code() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.get_cc(), ConditionCode::Zero);

        cpu.set_cc(ConditionCode::High);
        assert_eq!(cpu.get_cc(), ConditionCode::High);

        cpu.psw.set_cc_compare(-5);
        assert_eq!(cpu.get_cc(), ConditionCode::Low);

        cpu.psw.set_cc_compare(10);
        assert_eq!(cpu.get_cc(), ConditionCode::High);

        cpu.psw.set_cc_compare(0);
        assert_eq!(cpu.get_cc(), ConditionCode::Zero);
    }

    #[test]
    fn test_pc_manipulation() {
        let mut cpu = Cpu::new();
        assert_eq!(cpu.get_pc(), 0);

        cpu.set_pc(0x100);
        assert_eq!(cpu.get_pc(), 0x100);

        cpu.increment_pc(4);
        assert_eq!(cpu.get_pc(), 0x104);
    }

    #[test]
    fn test_halt() {
        let mut cpu = Cpu::new();
        assert!(!cpu.is_halted());

        cpu.halt();
        assert!(cpu.is_halted());
        assert!(cpu.psw.wait);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();
        cpu.set_gpr(2, 0x12345678).unwrap();
        cpu.set_pc(0x100);
        cpu.cycles = 10;
        cpu.halt();

        // Write some data to memory
        cpu.write_byte(0x50, 0xAB).unwrap();
        cpu.write_byte(0x51, 0xCD).unwrap();

        cpu.reset();
        assert_eq!(cpu.get_gpr(2).unwrap(), 0);
        assert_eq!(cpu.get_pc(), 0);
        assert_eq!(cpu.cycles, 0);
        assert!(!cpu.is_halted());

        // Memory should be preserved after reset
        assert_eq!(cpu.read_byte(0x50).unwrap(), 0xAB);
        assert_eq!(cpu.read_byte(0x51).unwrap(), 0xCD);
    }
}
