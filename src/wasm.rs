use crate::assembler::assemble;
use crate::challenge::get_all_challenges;
use crate::cpu::{ConditionCode, Cpu, Instruction, execute_instruction};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// WASM-exposed CPU wrapper
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmCpu {
    cpu: Cpu,
    program_size: usize,
}

/// Register state for JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterState {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
    pub r13: u32,
    pub r14: u32,
    pub r15: u32,
    pub pc: u32,
    pub cc: u8,
    pub wait: bool,
    pub addressing_mode_31bit: bool,
    pub cycles: u64,
    pub instructions: u64,
    pub halted: bool,
}

#[wasm_bindgen]
impl WasmCpu {
    /// Create a new CPU instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            program_size: 0,
        }
    }

    /// Reset CPU to initial state
    pub fn reset(&mut self) {
        self.cpu.reset();
        self.program_size = 0;
    }

    /// Assemble source code and load into memory
    pub fn assemble(&mut self, source: &str) -> Result<JsValue, JsValue> {
        let output = assemble(source).map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Flatten all bytes into program
        let mut program = Vec::new();
        for line in &output.lines {
            program.extend_from_slice(&line.bytes);
        }

        self.program_size = program.len();
        self.cpu
            .load_program(&program)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Initialize memory from DATA directives
        for (addr, value) in &output.data_inits {
            self.cpu
                .write_word(*addr, *value)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }

        // Set PC to program start address (1M)
        self.cpu.set_pc(Cpu::PROGRAM_START_ADDRESS);

        // Switch to 31-bit addressing mode (from initial 24-bit mode)
        self.cpu.psw.addressing_mode_31bit = true;

        // Clear wait state - program is loaded and ready to execute
        self.cpu.psw.wait = false;

        // Initialize ESA/390 calling convention registers
        // R12: Base register - points to program start (for USING directive)
        self.cpu
            .set_gpr(12, Cpu::PROGRAM_START_ADDRESS)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // R14: Return address - set to HALT address (program returns by BR R14)
        // For simplicity, we use 0 which will cause a halt if branched to
        self.cpu
            .set_gpr(14, 0)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // R15: Return code - initialize to 0
        self.cpu
            .set_gpr(15, 0)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Return assembly output
        serde_wasm_bindgen::to_value(&output).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Execute a single instruction
    pub fn step(&mut self) -> Result<JsValue, JsValue> {
        if self.cpu.is_halted() {
            return Err(JsValue::from_str("CPU is halted"));
        }

        let pc = self.cpu.get_pc();

        // Decode instruction at current PC
        let instr_bytes = (0..4)
            .filter_map(|i| self.cpu.read_byte(pc.wrapping_add(i)).ok())
            .collect::<Vec<_>>();

        if instr_bytes.is_empty() {
            return Err(JsValue::from_str("Failed to read instruction"));
        }

        let instruction = Instruction::decode(&instr_bytes)
            .ok_or_else(|| JsValue::from_str("Failed to decode instruction"))?;

        // Execute instruction
        execute_instruction(&mut self.cpu, &instruction)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Advance PC
        self.cpu.increment_pc(instruction.opcode.length() as u32);

        // Return register state
        self.get_state()
    }

    /// Run until halt or max cycles
    pub fn run(&mut self, max_cycles: u32) -> Result<JsValue, JsValue> {
        let start_cycles = self.cpu.cycles;

        while !self.cpu.is_halted() && (self.cpu.cycles - start_cycles) < max_cycles as u64 {
            self.step()?;
        }

        self.get_state()
    }

    /// Get current register state
    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        let state = RegisterState {
            r0: self.cpu.gprs[0],
            r1: self.cpu.gprs[1],
            r2: self.cpu.gprs[2],
            r3: self.cpu.gprs[3],
            r4: self.cpu.gprs[4],
            r5: self.cpu.gprs[5],
            r6: self.cpu.gprs[6],
            r7: self.cpu.gprs[7],
            r8: self.cpu.gprs[8],
            r9: self.cpu.gprs[9],
            r10: self.cpu.gprs[10],
            r11: self.cpu.gprs[11],
            r12: self.cpu.gprs[12],
            r13: self.cpu.gprs[13],
            r14: self.cpu.gprs[14],
            r15: self.cpu.gprs[15],
            pc: self.cpu.psw.instruction_address,
            cc: self.cpu.psw.condition_code as u8,
            wait: self.cpu.psw.wait,
            addressing_mode_31bit: self.cpu.psw.addressing_mode_31bit,
            cycles: self.cpu.cycles,
            instructions: self.cpu.instructions_executed,
            halted: self.cpu.halted,
        };

        serde_wasm_bindgen::to_value(&state).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get memory contents
    pub fn get_memory(&self, start: u32, length: u32) -> Result<JsValue, JsValue> {
        let mut bytes = Vec::new();
        for i in 0..length {
            let addr = start.wrapping_add(i);
            bytes.push(self.cpu.read_byte(addr).unwrap_or(0));
        }

        serde_wasm_bindgen::to_value(&bytes).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get program start address constant (1M)
    pub fn get_program_start_address(&self) -> u32 {
        Cpu::PROGRAM_START_ADDRESS
    }

    /// Get a specific GPR value
    pub fn get_gpr(&self, reg: u8) -> Result<u32, JsValue> {
        self.cpu
            .get_gpr(reg)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Set a specific GPR value
    pub fn set_gpr(&mut self, reg: u8, value: u32) -> Result<(), JsValue> {
        self.cpu
            .set_gpr(reg, value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Write a 32-bit word to memory
    pub fn write_memory(&mut self, addr: u32, value: u32) -> Result<(), JsValue> {
        self.cpu
            .write_word(addr, value)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get program counter
    pub fn get_pc(&self) -> u32 {
        self.cpu.get_pc()
    }

    /// Set program counter
    pub fn set_pc(&mut self, addr: u32) {
        self.cpu.set_pc(addr);
    }

    /// Get condition code
    pub fn get_cc(&self) -> u8 {
        self.cpu.get_cc() as u8
    }

    /// Check if halted
    pub fn is_halted(&self) -> bool {
        self.cpu.is_halted()
    }

    /// Get program size
    pub fn get_program_size(&self) -> usize {
        self.program_size
    }

    /// Get condition code name
    pub fn get_cc_name(&self) -> String {
        match self.cpu.get_cc() {
            ConditionCode::Zero => "Zero".to_string(),
            ConditionCode::Low => "Low".to_string(),
            ConditionCode::High => "High".to_string(),
            ConditionCode::Overflow => "Overflow".to_string(),
        }
    }

    /// Get all available challenges
    pub fn get_challenges(&self) -> Result<JsValue, JsValue> {
        let challenges = get_all_challenges();
        serde_wasm_bindgen::to_value(&challenges).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Check current CPU state against a challenge
    pub fn check_challenge(&self, challenge_id: u32) -> Result<JsValue, JsValue> {
        let challenges = get_all_challenges();
        let challenge = challenges
            .iter()
            .find(|c| c.id == challenge_id)
            .ok_or_else(|| JsValue::from_str(&format!("Challenge {} not found", challenge_id)))?;

        let result = challenge
            .validate_solution(&self.cpu)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl Default for WasmCpu {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the WASM module and mount Yew app
#[wasm_bindgen(start)]
pub fn init() {
    // Set panic hook for better error messages in browser console
    console_error_panic_hook::set_once();

    // Mount the Yew app
    yew::Renderer::<crate::app::App>::new().render();
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_cpu_creation() {
        let cpu = WasmCpu::new();
        assert!(!cpu.is_halted());
        assert_eq!(cpu.get_pc(), 0);
    }

    #[test]
    fn test_wasm_reset() {
        let mut cpu = WasmCpu::new();
        cpu.set_gpr(1, 100).unwrap();
        cpu.set_pc(50);

        cpu.reset();
        assert_eq!(cpu.get_gpr(1).unwrap(), 0);
        assert_eq!(cpu.get_pc(), 0);
    }

    // Note: Tests that use JsValue (assemble, step) can only run on wasm32 targets
    // They are tested via wasm-bindgen-test or browser-based testing
}
