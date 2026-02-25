use crate::cpu::instruction::{Instruction, Opcode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Assembly errors
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum AssemblyError {
    #[error("Unknown instruction: {0}")]
    UnknownInstruction(String),
    #[error("Invalid operand: {0}")]
    InvalidOperand(String),
    #[error("Invalid register: {0}")]
    InvalidRegister(String),
    #[error("Invalid immediate value: {0}")]
    InvalidImmediate(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Line {line}: {message}")]
    ParseError { line: usize, message: String },
    #[error("Program too large: {0} bytes")]
    ProgramTooLarge(usize),
}

/// Assembly line result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledLine {
    pub line_number: usize,
    pub source: String,
    pub address: u32,
    pub bytes: Vec<u8>,
    pub instruction: Option<String>,
}

/// Assembly output with all assembled lines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyOutput {
    pub lines: Vec<AssembledLine>,
    pub total_bytes: usize,
    /// DATA directive initializations (address, value)
    pub data_inits: Vec<(u32, u32)>,
}

/// Assemble a program from source code
/// Programs are assembled starting from address 0 (relative addressing)
/// The CPU loader will place them at PROGRAM_START_ADDRESS (1M)
pub fn assemble(source: &str) -> Result<AssemblyOutput, AssemblyError> {
    let mut assembled_lines = Vec::new();
    let mut current_address = 0u32;
    let mut labels: HashMap<String, u32> = HashMap::new();

    // First pass: collect labels and calculate addresses
    for line in source.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments (support both ; and #)
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        // Check for USING directive (doesn't generate code, skip it)
        if trimmed.to_uppercase().starts_with("USING") {
            continue; // USING doesn't generate code
        }

        // Check for DATA directive (doesn't generate code, skip it)
        if trimmed.to_uppercase().starts_with("DATA") {
            continue; // DATA doesn't generate code
        }

        // Check for labels (ends with :)
        if let Some(colon_idx) = trimmed.find(':') {
            let label = trimmed[..colon_idx].trim().to_string();
            labels.insert(label, current_address);

            // Process rest of line if there's anything after the colon
            let rest = trimmed[colon_idx + 1..].trim();
            if !rest.is_empty()
                && !rest.starts_with(';')
                && !rest.to_uppercase().starts_with("USING")
                && !rest.to_uppercase().starts_with("DATA")
            {
                let instr_len = estimate_instruction_length(rest)?;
                current_address = current_address.wrapping_add(instr_len);
            }
        } else {
            // Regular instruction
            let instr_len = estimate_instruction_length(trimmed)?;
            current_address = current_address.wrapping_add(instr_len);
        }
    }

    // Second pass: assemble instructions and collect DATA directives
    current_address = 0;
    let mut base_register: Option<u8> = None; // Track USING directive base register
    let mut data_inits = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Skip empty lines and comments (support both ; and #)
        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            continue;
        }

        // Check for USING directive (update base register)
        if trimmed.to_uppercase().starts_with("USING") {
            base_register = parse_using_directive(trimmed)?;
            continue; // USING doesn't generate code
        }

        // Check for DATA directive (parse and collect)
        if trimmed.to_uppercase().starts_with("DATA") {
            let (addr, value) = parse_data_directive(trimmed)?;
            data_inits.push((addr, value));
            continue; // DATA doesn't generate code
        }

        // Remove inline comments (support both ; and #)
        let code = if let Some(comment_idx) = trimmed.find(';').or_else(|| trimmed.find('#')) {
            trimmed[..comment_idx].trim()
        } else {
            trimmed
        };

        // Handle labels
        let code = if let Some(colon_idx) = code.find(':') {
            let rest = code[colon_idx + 1..].trim();
            if rest.is_empty() {
                continue;
            }
            rest
        } else {
            code
        };

        // Parse and assemble instruction (with base register context)
        match parse_instruction(code, &labels, base_register) {
            Ok(instruction) => {
                let bytes = instruction.encode();
                let instr_str = format!("{}", instruction);

                assembled_lines.push(AssembledLine {
                    line_number: line_num + 1,
                    source: line.to_string(),
                    address: current_address,
                    bytes: bytes.clone(),
                    instruction: Some(instr_str),
                });

                current_address = current_address.wrapping_add(bytes.len() as u32);
            }
            Err(e) => {
                return Err(AssemblyError::ParseError {
                    line: line_num + 1,
                    message: e.to_string(),
                });
            }
        }
    }

    let total_bytes: usize = assembled_lines.iter().map(|l| l.bytes.len()).sum();

    if total_bytes > 65536 {
        return Err(AssemblyError::ProgramTooLarge(total_bytes));
    }

    Ok(AssemblyOutput {
        lines: assembled_lines,
        total_bytes,
        data_inits,
    })
}

/// Estimate instruction length for first pass
fn estimate_instruction_length(line: &str) -> Result<u32, AssemblyError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(0);
    }

    let mnemonic = parts[0].to_uppercase();
    match mnemonic.as_str() {
        "BR" | "AR" | "SR" | "MR" | "DR" | "CR" | "LR" | "NOP" => Ok(2),
        "L" | "ST" | "A" | "S" | "M" | "D" | "C" | "MVI" | "CLI" | "HIO" => Ok(4),
        _ => Err(AssemblyError::UnknownInstruction(mnemonic)),
    }
}

/// Parse a single instruction line with optional base register
fn parse_instruction(
    line: &str,
    labels: &HashMap<String, u32>,
    base_register: Option<u8>,
) -> Result<Instruction, AssemblyError> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Err(AssemblyError::InvalidOperand(
            "Empty instruction".to_string(),
        ));
    }

    let mnemonic = parts[0].to_uppercase();
    let operands = if parts.len() > 1 {
        parts[1..].join(" ")
    } else {
        String::new()
    };

    match mnemonic.as_str() {
        // RR Format
        "BR" => parse_br(&operands), // Branch Register (special case: only uses R2)
        "AR" => parse_rr(Opcode::AR, &operands),
        "SR" => parse_rr(Opcode::SR, &operands),
        "MR" => parse_rr(Opcode::MR, &operands),
        "DR" => parse_rr(Opcode::DR, &operands),
        "CR" => parse_rr(Opcode::CR, &operands),
        "LR" => parse_rr(Opcode::LR, &operands),

        // RX Format
        "L" => parse_rx(Opcode::L, &operands, labels, base_register),
        "ST" => parse_rx(Opcode::ST, &operands, labels, base_register),
        "A" => parse_rx(Opcode::A, &operands, labels, base_register),
        "S" => parse_rx(Opcode::S, &operands, labels, base_register),
        "M" => parse_rx(Opcode::M, &operands, labels, base_register),
        "D" => parse_rx(Opcode::D, &operands, labels, base_register),
        "C" => parse_rx(Opcode::C, &operands, labels, base_register),

        // SI Format
        "MVI" => parse_si(Opcode::MVI, &operands, labels, base_register),
        "CLI" => parse_si(Opcode::CLI, &operands, labels, base_register),

        // S Format
        "HIO" => Ok(Instruction::new_s(Opcode::HIO, 0, 0)),

        // NOP
        "NOP" => Ok(Instruction::new_rr(Opcode::NOP, 0, 0)),

        _ => Err(AssemblyError::UnknownInstruction(mnemonic)),
    }
}

/// Parse BR format: BR R2 (Branch Register - unconditional branch to address in R2)
fn parse_br(operands: &str) -> Result<Instruction, AssemblyError> {
    let r2 = parse_register(operands.trim())?;
    // In ESA/390, BR is encoded as BCR 15,R2 (unconditional branch on condition register)
    // We use R1=15 (mask=all conditions) for unconditional branch
    Ok(Instruction::new_rr(Opcode::BR, 15, r2))
}

/// Parse RR format: OP R1, R2
fn parse_rr(opcode: Opcode, operands: &str) -> Result<Instruction, AssemblyError> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblyError::InvalidOperand(
            "RR format requires 2 operands".to_string(),
        ));
    }

    let r1 = parse_register(parts[0])?;
    let r2 = parse_register(parts[1])?;

    Ok(Instruction::new_rr(opcode, r1, r2))
}

/// Parse RX format: OP R1, D2(X2, B2) or OP R1, D2(B2) or OP R1, D2
fn parse_rx(
    opcode: Opcode,
    operands: &str,
    labels: &HashMap<String, u32>,
    base_register: Option<u8>,
) -> Result<Instruction, AssemblyError> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err(AssemblyError::InvalidOperand(
            "RX format requires at least 1 operand".to_string(),
        ));
    }

    let r1 = parse_register(parts[0])?;

    // Parse address part: D2(X2, B2) or D2(B2) or D2
    let addr_part = if parts.len() > 1 {
        parts[1..].join(",")
    } else {
        return Err(AssemblyError::InvalidOperand(
            "RX format requires address operand".to_string(),
        ));
    };

    let (d2, x2, b2) = parse_address(&addr_part, labels, base_register)?;

    // Truncate to 12-bit displacement (0-4095) for RX format
    let d2_12bit = (d2 & 0xFFF) as u16;
    Ok(Instruction::new_rx(opcode, r1, x2, b2, d2_12bit))
}

/// Parse SI format: OP D1(B1), I2 or OP D1, I2
fn parse_si(
    opcode: Opcode,
    operands: &str,
    labels: &HashMap<String, u32>,
    base_register: Option<u8>,
) -> Result<Instruction, AssemblyError> {
    let parts: Vec<&str> = operands.split(',').map(|s| s.trim()).collect();
    if parts.len() != 2 {
        return Err(AssemblyError::InvalidOperand(
            "SI format requires 2 operands".to_string(),
        ));
    }

    let (d1, _, b1) = parse_address(parts[0], labels, base_register)?;
    let i2 = parse_immediate(parts[1])?;

    // Truncate to 12-bit displacement (0-4095) for SI format
    let d1_12bit = (d1 & 0xFFF) as u16;
    Ok(Instruction::new_si(opcode, b1, d1_12bit, i2))
}

/// Parse register: R0-R15 (ESA/390 has 16 general purpose registers)
fn parse_register(s: &str) -> Result<u8, AssemblyError> {
    let s = s.trim().to_uppercase();
    if !s.starts_with('R') {
        return Err(AssemblyError::InvalidRegister(s.to_string()));
    }

    let num_str = &s[1..];
    let num = num_str
        .parse::<u8>()
        .map_err(|_| AssemblyError::InvalidRegister(s.to_string()))?;

    if num > 15 {
        return Err(AssemblyError::InvalidRegister(format!(
            "Register number must be 0-15, got {}",
            num
        )));
    }

    Ok(num)
}

/// Parse immediate value (decimal or hex)
fn parse_immediate(s: &str) -> Result<u8, AssemblyError> {
    let s = s.trim();

    if s.starts_with("0x") || s.starts_with("0X") {
        u8::from_str_radix(&s[2..], 16).map_err(|_| AssemblyError::InvalidImmediate(s.to_string()))
    } else {
        s.parse::<u8>()
            .map_err(|_| AssemblyError::InvalidImmediate(s.to_string()))
    }
}

/// Parse address: D2(X2, B2) or D2(B2) or D2 or LABEL
/// Returns (displacement, index_reg, base_reg)
/// If no base register is explicitly specified in the address and base_register is Some,
/// the USING base register will be used (ESA/390 convention)
fn parse_address(
    s: &str,
    labels: &HashMap<String, u32>,
    base_register: Option<u8>,
) -> Result<(u32, u8, u8), AssemblyError> {
    let s = s.trim();

    // Check if it's a label
    if let Some(&addr) = labels.get(s) {
        // Labels without explicit base use USING base register
        let b = base_register.unwrap_or(0);
        return Ok((addr, 0, b));
    }

    // Check for parentheses (explicit base/index specification)
    if let Some(paren_idx) = s.find('(') {
        let d_str = s[..paren_idx].trim();
        let d = parse_displacement(d_str)?;

        let reg_part = &s[paren_idx + 1..];
        if let Some(close_idx) = reg_part.find(')') {
            let regs = &reg_part[..close_idx];
            let reg_parts: Vec<&str> = regs.split(',').map(|s| s.trim()).collect();

            if reg_parts.len() == 1 {
                // D2(B2)
                let b2 = parse_register(reg_parts[0])?;
                Ok((d, 0, b2))
            } else if reg_parts.len() == 2 {
                // D2(X2, B2)
                let x2 = parse_register(reg_parts[0])?;
                let b2 = parse_register(reg_parts[1])?;
                Ok((d, x2, b2))
            } else {
                Err(AssemblyError::InvalidAddress(s.to_string()))
            }
        } else {
            Err(AssemblyError::InvalidAddress(s.to_string()))
        }
    } else {
        // Just displacement - use USING base register if available
        let d = parse_displacement(s)?;
        let b = base_register.unwrap_or(0);
        Ok((d, 0, b))
    }
}

/// Parse displacement value (decimal or hex)
fn parse_displacement(s: &str) -> Result<u32, AssemblyError> {
    let s = s.trim();

    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16).map_err(|_| AssemblyError::InvalidAddress(s.to_string()))
    } else {
        s.parse::<u32>()
            .map_err(|_| AssemblyError::InvalidAddress(s.to_string()))
    }
}

/// Parse USING directive: USING *,Rx or USING label,Rx
/// Returns the base register number
fn parse_using_directive(line: &str) -> Result<Option<u8>, AssemblyError> {
    // Strip comments first (support both ; and #)
    let line = if let Some(comment_idx) = line.find(';').or_else(|| line.find('#')) {
        &line[..comment_idx]
    } else {
        line
    };

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(AssemblyError::InvalidOperand(
            "USING directive requires format: USING *,Rx or USING label,Rx".to_string(),
        ));
    }

    // Format: USING location,register
    // Parse the register part (after comma)
    let operands = parts[1..].join(" ");
    if let Some(comma_idx) = operands.find(',') {
        let reg_str = operands[comma_idx + 1..].trim();
        let base_reg = parse_register(reg_str)?;
        Ok(Some(base_reg))
    } else {
        Err(AssemblyError::InvalidOperand(
            "USING directive missing comma".to_string(),
        ))
    }
}

/// Parse DATA directive: DATA address value
/// Returns (address, value)
fn parse_data_directive(line: &str) -> Result<(u32, u32), AssemblyError> {
    // Strip comments first (support both ; and #)
    let line = if let Some(comment_idx) = line.find(';').or_else(|| line.find('#')) {
        &line[..comment_idx]
    } else {
        line
    };

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(AssemblyError::InvalidOperand(
            "DATA directive requires format: DATA address value".to_string(),
        ));
    }

    // Parse address (hex or decimal)
    let addr = parse_displacement(parts[1])?;

    // Parse value (hex or decimal)
    let value = parse_displacement(parts[2])?;

    Ok((addr, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register() {
        assert_eq!(parse_register("R0").unwrap(), 0);
        assert_eq!(parse_register("R7").unwrap(), 7);
        assert_eq!(parse_register("r3").unwrap(), 3);
        assert_eq!(parse_register("R12").unwrap(), 12);
        assert_eq!(parse_register("R15").unwrap(), 15);
        assert!(parse_register("R16").is_err()); // ESA/390 has R0-R15
        assert!(parse_register("X1").is_err());
    }

    #[test]
    fn test_parse_immediate() {
        assert_eq!(parse_immediate("42").unwrap(), 42);
        assert_eq!(parse_immediate("0xFF").unwrap(), 255);
        assert_eq!(parse_immediate("0x10").unwrap(), 16);
        assert!(parse_immediate("256").is_err());
    }

    #[test]
    fn test_parse_address() {
        let labels = HashMap::new();

        // Simple displacement (no USING base)
        let (d, x, b) = parse_address("0x100", &labels, None).unwrap();
        assert_eq!(d, 0x100);
        assert_eq!(x, 0);
        assert_eq!(b, 0);

        // Simple displacement with USING base
        let (d, x, b) = parse_address("0x100", &labels, Some(12)).unwrap();
        assert_eq!(d, 0x100);
        assert_eq!(x, 0);
        assert_eq!(b, 12); // Uses USING base register

        // Displacement with explicit base (overrides USING)
        let (d, x, b) = parse_address("0x100(R5)", &labels, Some(12)).unwrap();
        assert_eq!(d, 0x100);
        assert_eq!(x, 0);
        assert_eq!(b, 5); // Explicit base, not USING base

        // Displacement with index and base
        let (d, x, b) = parse_address("0x100(R3, R5)", &labels, None).unwrap();
        assert_eq!(d, 0x100);
        assert_eq!(x, 3);
        assert_eq!(b, 5);

        // Decimal displacement
        let (d, x, b) = parse_address("256(R2)", &labels, None).unwrap();
        assert_eq!(d, 256);
        assert_eq!(x, 0);
        assert_eq!(b, 2);
    }

    #[test]
    fn test_parse_rr() {
        let instr = parse_rr(Opcode::AR, "R1, R2").unwrap();
        assert_eq!(instr.opcode, Opcode::AR);
    }

    #[test]
    fn test_assemble_simple() {
        let source = r#"
            LR R1, R2
            AR R1, R3
            HIO
        "#;

        let output = assemble(source).unwrap();
        assert_eq!(output.lines.len(), 3);
        assert_eq!(output.lines[0].bytes.len(), 2); // LR is RR format (2 bytes)
        assert_eq!(output.lines[1].bytes.len(), 2); // AR is RR format (2 bytes)
        assert_eq!(output.lines[2].bytes.len(), 4); // HIO is S format (4 bytes)
        assert_eq!(output.total_bytes, 8);
    }

    #[test]
    fn test_assemble_with_comments() {
        let source = r#"
            ; This is a comment
            LR R1, R2  ; Load R2 into R1
            AR R1, R3  ; Add R3 to R1
            HIO        ; Halt
        "#;

        let output = assemble(source).unwrap();
        assert_eq!(output.lines.len(), 3);
    }

    #[test]
    fn test_assemble_with_labels() {
        let source = r#"
            LR R1, R2
        LOOP:
            AR R1, R3
            HIO
        "#;

        let output = assemble(source).unwrap();
        assert_eq!(output.lines.len(), 3);
        assert_eq!(output.lines[1].address, 2); // LOOP label should be at address 2
    }

    #[test]
    fn test_assemble_rx_format() {
        let source = r#"
            L R1, 0x100
            ST R2, 0x200(R5)
            A R3, 0x50(R4, R6)
        "#;

        let output = assemble(source).unwrap();
        assert_eq!(output.lines.len(), 3);
        assert_eq!(output.lines[0].bytes.len(), 4); // RX format
        assert_eq!(output.lines[1].bytes.len(), 4);
        assert_eq!(output.lines[2].bytes.len(), 4);
    }

    #[test]
    fn test_assemble_si_format() {
        let source = r#"
            MVI 0x100, 0x42
            CLI 0x200(R3), 0xFF
        "#;

        let output = assemble(source).unwrap();
        assert_eq!(output.lines.len(), 2);
        assert_eq!(output.lines[0].bytes.len(), 4); // SI format
        assert_eq!(output.lines[1].bytes.len(), 4);
    }

    #[test]
    fn test_invalid_instruction() {
        let source = "INVALID R1, R2";
        assert!(assemble(source).is_err());
    }

    #[test]
    fn test_invalid_register() {
        let source = "LR R1, R16"; // R16 doesn't exist (only R0-R15)
        assert!(assemble(source).is_err());
    }
}
