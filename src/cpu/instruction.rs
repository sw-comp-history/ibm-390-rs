use serde::{Deserialize, Serialize};
use std::fmt;

/// Instruction formats for IBM ESA/390
/// We implement a simplified subset for educational purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstructionFormat {
    /// RR: Register-Register (2 bytes)
    /// Format: OP R1 R2
    /// Bits: [opcode:8][r1:4][r2:4]
    RR,

    /// RX: Register-Indexed Storage (4 bytes)
    /// Format: OP R1, D2(X2, B2)
    /// Bits: [opcode:8][r1:4][x2:4][b2:4][d2:12]
    RX,

    /// SI: Storage-Immediate (4 bytes)
    /// Format: OP D1(B1), I2
    /// Bits: [opcode:8][i2:8][b1:4][d1:12]
    SI,

    /// S: Storage (4 bytes)
    /// Format: OP D2(B2)
    /// Bits: [opcode:16][b2:4][d2:12]
    S,
}

/// Instruction opcodes (simplified subset)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Opcode {
    // RR Format Instructions (Register-Register)
    /// Branch Register (PC = R2)
    BR = 0x07,
    /// Add Register (R1 = R1 + R2)
    AR = 0x1A,
    /// Subtract Register (R1 = R1 - R2)
    SR = 0x1B,
    /// Multiply Register (R1 = R1 * R2)
    MR = 0x1C,
    /// Divide Register (R1 = R1 / R2)
    DR = 0x1D,
    /// Compare Register (CC = compare R1, R2)
    CR = 0x19,
    /// Load Register (R1 = R2)
    LR = 0x18,

    // RX Format Instructions (Register-Indexed Storage)
    /// Load (R1 = memory[addr])
    L = 0x58,
    /// Store (memory[addr] = R1)
    ST = 0x50,
    /// Add (R1 = R1 + memory[addr])
    A = 0x5A,
    /// Subtract (R1 = R1 - memory[addr])
    S = 0x5B,
    /// Multiply (R1 = R1 * memory[addr])
    M = 0x5C,
    /// Divide (R1 = R1 / memory[addr])
    D = 0x5D,
    /// Compare (CC = compare R1, memory[addr])
    C = 0x59,

    // SI Format Instructions (Storage-Immediate)
    /// Move Immediate (memory[addr] = immediate)
    MVI = 0x92,
    /// Compare Logical Immediate (CC = compare memory[addr], immediate)
    CLI = 0x95,

    // S Format Instructions (Storage)
    /// Halt I/O (simplified as HALT for game)
    HIO = 0x9E,

    /// No Operation
    NOP = 0x00,
}

impl Opcode {
    /// Get instruction format for this opcode
    pub fn format(&self) -> InstructionFormat {
        match self {
            Opcode::BR
            | Opcode::AR
            | Opcode::SR
            | Opcode::MR
            | Opcode::DR
            | Opcode::CR
            | Opcode::LR => InstructionFormat::RR,

            Opcode::L | Opcode::ST | Opcode::A | Opcode::S | Opcode::M | Opcode::D | Opcode::C => {
                InstructionFormat::RX
            }

            Opcode::MVI | Opcode::CLI => InstructionFormat::SI,

            Opcode::HIO => InstructionFormat::S,

            Opcode::NOP => InstructionFormat::RR, // NOP can be represented as RR
        }
    }

    /// Get instruction length in bytes
    pub fn length(&self) -> u16 {
        match self.format() {
            InstructionFormat::RR => 2,
            InstructionFormat::RX | InstructionFormat::SI | InstructionFormat::S => 4,
        }
    }

    /// Get mnemonic string
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Opcode::BR => "BR",
            Opcode::AR => "AR",
            Opcode::SR => "SR",
            Opcode::MR => "MR",
            Opcode::DR => "DR",
            Opcode::CR => "CR",
            Opcode::LR => "LR",
            Opcode::L => "L",
            Opcode::ST => "ST",
            Opcode::A => "A",
            Opcode::S => "S",
            Opcode::M => "M",
            Opcode::D => "D",
            Opcode::C => "C",
            Opcode::MVI => "MVI",
            Opcode::CLI => "CLI",
            Opcode::HIO => "HIO",
            Opcode::NOP => "NOP",
        }
    }

    /// Parse opcode from byte
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x07 => Some(Opcode::BR),
            0x1A => Some(Opcode::AR),
            0x1B => Some(Opcode::SR),
            0x1C => Some(Opcode::MR),
            0x1D => Some(Opcode::DR),
            0x19 => Some(Opcode::CR),
            0x18 => Some(Opcode::LR),
            0x58 => Some(Opcode::L),
            0x50 => Some(Opcode::ST),
            0x5A => Some(Opcode::A),
            0x5B => Some(Opcode::S),
            0x5C => Some(Opcode::M),
            0x5D => Some(Opcode::D),
            0x59 => Some(Opcode::C),
            0x92 => Some(Opcode::MVI),
            0x95 => Some(Opcode::CLI),
            0x9E => Some(Opcode::HIO),
            0x00 => Some(Opcode::NOP),
            _ => None,
        }
    }
}

/// Decoded instruction with operands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub opcode: Opcode,
    pub format: InstructionFormat,
    pub operands: Operands,
}

/// Instruction operands based on format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operands {
    /// RR format: R1, R2
    RR { r1: u8, r2: u8 },

    /// RX format: R1, D2(X2, B2)
    RX { r1: u8, x2: u8, b2: u8, d2: u16 },

    /// SI format: D1(B1), I2
    SI { b1: u8, d1: u16, i2: u8 },

    /// S format: D2(B2)
    S { b2: u8, d2: u16 },
}

impl Instruction {
    /// Create a new RR format instruction
    pub fn new_rr(opcode: Opcode, r1: u8, r2: u8) -> Self {
        Self {
            opcode,
            format: InstructionFormat::RR,
            operands: Operands::RR { r1, r2 },
        }
    }

    /// Create a new RX format instruction
    pub fn new_rx(opcode: Opcode, r1: u8, x2: u8, b2: u8, d2: u16) -> Self {
        Self {
            opcode,
            format: InstructionFormat::RX,
            operands: Operands::RX { r1, x2, b2, d2 },
        }
    }

    /// Create a new SI format instruction
    pub fn new_si(opcode: Opcode, b1: u8, d1: u16, i2: u8) -> Self {
        Self {
            opcode,
            format: InstructionFormat::SI,
            operands: Operands::SI { b1, d1, i2 },
        }
    }

    /// Create a new S format instruction
    pub fn new_s(opcode: Opcode, b2: u8, d2: u16) -> Self {
        Self {
            opcode,
            format: InstructionFormat::S,
            operands: Operands::S { b2, d2 },
        }
    }

    /// Encode instruction to bytes (big-endian)
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        match &self.operands {
            Operands::RR { r1, r2 } => {
                bytes.push(self.opcode as u8);
                bytes.push((r1 << 4) | r2);
            }
            Operands::RX { r1, x2, b2, d2 } => {
                bytes.push(self.opcode as u8);
                bytes.push((r1 << 4) | x2);
                bytes.push((b2 << 4) | ((d2 >> 8) as u8 & 0x0F));
                bytes.push(*d2 as u8);
            }
            Operands::SI { b1, d1, i2 } => {
                bytes.push(self.opcode as u8);
                bytes.push(*i2);
                bytes.push((b1 << 4) | ((d1 >> 8) as u8 & 0x0F));
                bytes.push(*d1 as u8);
            }
            Operands::S { b2, d2 } => {
                bytes.push(self.opcode as u8);
                bytes.push(0x00); // Upper byte of opcode for S format
                bytes.push((b2 << 4) | ((d2 >> 8) as u8 & 0x0F));
                bytes.push(*d2 as u8);
            }
        }

        bytes
    }

    /// Decode instruction from bytes (big-endian)
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let opcode = Opcode::from_byte(bytes[0])?;
        let format = opcode.format();

        match format {
            InstructionFormat::RR => {
                if bytes.len() < 2 {
                    return None;
                }
                let r1 = (bytes[1] >> 4) & 0x0F;
                let r2 = bytes[1] & 0x0F;
                Some(Self::new_rr(opcode, r1, r2))
            }
            InstructionFormat::RX => {
                if bytes.len() < 4 {
                    return None;
                }
                let r1 = (bytes[1] >> 4) & 0x0F;
                let x2 = bytes[1] & 0x0F;
                let b2 = (bytes[2] >> 4) & 0x0F;
                let d2 = (((bytes[2] & 0x0F) as u16) << 8) | (bytes[3] as u16);
                Some(Self::new_rx(opcode, r1, x2, b2, d2))
            }
            InstructionFormat::SI => {
                if bytes.len() < 4 {
                    return None;
                }
                let i2 = bytes[1];
                let b1 = (bytes[2] >> 4) & 0x0F;
                let d1 = (((bytes[2] & 0x0F) as u16) << 8) | (bytes[3] as u16);
                Some(Self::new_si(opcode, b1, d1, i2))
            }
            InstructionFormat::S => {
                if bytes.len() < 4 {
                    return None;
                }
                let b2 = (bytes[2] >> 4) & 0x0F;
                let d2 = (((bytes[2] & 0x0F) as u16) << 8) | (bytes[3] as u16);
                Some(Self::new_s(opcode, b2, d2))
            }
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode.mnemonic())?;

        match &self.operands {
            Operands::RR { r1, r2 } => {
                write!(f, " R{}, R{}", r1, r2)?;
            }
            Operands::RX { r1, x2, b2, d2 } => {
                if *x2 == 0 && *b2 == 0 {
                    write!(f, " R{}, {:#x}", r1, d2)?;
                } else if *x2 == 0 {
                    write!(f, " R{}, {:#x}(R{})", r1, d2, b2)?;
                } else if *b2 == 0 {
                    write!(f, " R{}, {:#x}(R{})", r1, d2, x2)?;
                } else {
                    write!(f, " R{}, {:#x}(R{}, R{})", r1, d2, x2, b2)?;
                }
            }
            Operands::SI { b1, d1, i2 } => {
                if *b1 == 0 {
                    write!(f, " {:#x}, {:#x}", d1, i2)?;
                } else {
                    write!(f, " {:#x}(R{}), {:#x}", d1, b1, i2)?;
                }
            }
            Operands::S { b2, d2 } => {
                if *b2 == 0 {
                    write!(f, " {:#x}", d2)?;
                } else {
                    write!(f, " {:#x}(R{})", d2, b2)?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_format() {
        assert_eq!(Opcode::AR.format(), InstructionFormat::RR);
        assert_eq!(Opcode::L.format(), InstructionFormat::RX);
        assert_eq!(Opcode::MVI.format(), InstructionFormat::SI);
        assert_eq!(Opcode::HIO.format(), InstructionFormat::S);
    }

    #[test]
    fn test_opcode_length() {
        assert_eq!(Opcode::AR.length(), 2);
        assert_eq!(Opcode::L.length(), 4);
        assert_eq!(Opcode::MVI.length(), 4);
        assert_eq!(Opcode::HIO.length(), 4);
    }

    #[test]
    fn test_opcode_mnemonic() {
        assert_eq!(Opcode::AR.mnemonic(), "AR");
        assert_eq!(Opcode::L.mnemonic(), "L");
        assert_eq!(Opcode::MVI.mnemonic(), "MVI");
        assert_eq!(Opcode::HIO.mnemonic(), "HIO");
    }

    #[test]
    fn test_opcode_from_byte() {
        assert_eq!(Opcode::from_byte(0x1A), Some(Opcode::AR));
        assert_eq!(Opcode::from_byte(0x58), Some(Opcode::L));
        assert_eq!(Opcode::from_byte(0x92), Some(Opcode::MVI));
        assert_eq!(Opcode::from_byte(0x9E), Some(Opcode::HIO));
        assert_eq!(Opcode::from_byte(0xFF), None);
    }

    #[test]
    fn test_rr_encode_decode() {
        let instr = Instruction::new_rr(Opcode::AR, 3, 5);
        let bytes = instr.encode();
        assert_eq!(bytes, vec![0x1A, 0x35]);

        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(decoded.opcode, Opcode::AR);
        match decoded.operands {
            Operands::RR { r1, r2 } => {
                assert_eq!(r1, 3);
                assert_eq!(r2, 5);
            }
            _ => panic!("Wrong operand type"),
        }
    }

    #[test]
    fn test_rx_encode_decode() {
        let instr = Instruction::new_rx(Opcode::L, 2, 0, 5, 0x100);
        let bytes = instr.encode();
        assert_eq!(bytes, vec![0x58, 0x20, 0x51, 0x00]);

        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(decoded.opcode, Opcode::L);
        match decoded.operands {
            Operands::RX { r1, x2, b2, d2 } => {
                assert_eq!(r1, 2);
                assert_eq!(x2, 0);
                assert_eq!(b2, 5);
                assert_eq!(d2, 0x100);
            }
            _ => panic!("Wrong operand type"),
        }
    }

    #[test]
    fn test_si_encode_decode() {
        let instr = Instruction::new_si(Opcode::MVI, 3, 0x200, 0x42);
        let bytes = instr.encode();
        assert_eq!(bytes, vec![0x92, 0x42, 0x32, 0x00]);

        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(decoded.opcode, Opcode::MVI);
        match decoded.operands {
            Operands::SI { b1, d1, i2 } => {
                assert_eq!(b1, 3);
                assert_eq!(d1, 0x200);
                assert_eq!(i2, 0x42);
            }
            _ => panic!("Wrong operand type"),
        }
    }

    #[test]
    fn test_s_encode_decode() {
        let instr = Instruction::new_s(Opcode::HIO, 0, 0);
        let bytes = instr.encode();
        assert_eq!(bytes, vec![0x9E, 0x00, 0x00, 0x00]);

        let decoded = Instruction::decode(&bytes).unwrap();
        assert_eq!(decoded.opcode, Opcode::HIO);
        match decoded.operands {
            Operands::S { b2, d2 } => {
                assert_eq!(b2, 0);
                assert_eq!(d2, 0);
            }
            _ => panic!("Wrong operand type"),
        }
    }

    #[test]
    fn test_instruction_display() {
        let instr = Instruction::new_rr(Opcode::AR, 1, 2);
        assert_eq!(format!("{}", instr), "AR R1, R2");

        let instr = Instruction::new_rx(Opcode::L, 3, 0, 5, 0x100);
        assert_eq!(format!("{}", instr), "L R3, 0x100(R5)");

        let instr = Instruction::new_si(Opcode::MVI, 2, 0x50, 0xFF);
        assert_eq!(format!("{}", instr), "MVI 0x50(R2), 0xff");

        let instr = Instruction::new_s(Opcode::HIO, 0, 0);
        assert_eq!(format!("{}", instr), "HIO 0x0");
    }
}
