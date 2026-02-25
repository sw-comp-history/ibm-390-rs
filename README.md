# IBM ESA/390 Assembly Emulator

An interactive browser-based educational game for learning the IBM ESA/390 instruction set architecture. The ESA/390 (Enterprise Systems Architecture/390) was introduced by IBM in 1990 as the successor to System/370, and remains the foundation for modern IBM Z mainframes.

## Live Demo

**[Try it online](https://sw-comp-history.github.io/ibm-390-rs/)**

![IBM ESA/390 Emulator Screenshot](images/ibm-390-interface.png?ts=1772048087000)

## Features

- **IBM ESA/390 CPU emulation** - 32-bit architecture with multiple instruction formats
- **16 General Purpose Registers** - R0-R15 for arithmetic and addressing
- **Multiple instruction formats** - RR (Register-Register), RX (Register-Indexed), SI (Storage-Immediate), S (Storage)
- **Complete instruction subset** - L, ST, LR, AR, SR, MR, DR, CR, A, S, M, D, C, MVI, CLI, BR, HIO
- **Interactive examples** covering register operations, memory access, and arithmetic
- **Progressive challenges** with validation
- **Real-time visualization** of CPU state, registers, and memory

## Documentation

- [Porting Guide](docs/porting.md) - How this project was extracted from game-lib

## Architecture

The IBM ESA/390 emulator implements a simplified 32-bit architecture with:

- **R0-R15** - 16 General Purpose Registers (32-bit each)
- **PSW** - Program Status Word containing condition code and instruction address
- **Condition Code** - 2-bit CC for comparison results (0=Equal, 1=Low, 2=High)
- **Instruction Formats**:
  - **RR** - Register-Register (2 bytes): Operations between two registers
  - **RX** - Register-Indexed Storage (4 bytes): Register with memory address using base+index+displacement
  - **SI** - Storage-Immediate (4 bytes): Memory location with immediate value
  - **S** - Storage (4 bytes): Single memory operand

## Building

### Prerequisites

- [Rust](https://rustup.rs/) (with wasm32-unknown-unknown target)
- [Trunk](https://trunkrs.dev/) - `cargo install trunk`

### Development

```bash
# Run development server with hot reload
trunk serve

# Build for production
trunk build --release
```

The production build outputs to `./pages/`.

### Deploying to GitHub Pages

1. Build locally:
   ```bash
   trunk build --release
   ```

2. Update gh-pages branch:
   ```bash
   git checkout gh-pages
   rm -rf *.js *.wasm *.css index.html
   cp -r pages/* .
   git add .
   git commit -m "Deploy"
   git push
   git checkout main
   ```

## Project Structure

```
ibm-390-rs/
├── src/                    # Main application
│   ├── app.rs             # Yew application component
│   ├── assembler.rs       # Assembly parser
│   ├── challenge.rs       # Challenge system
│   ├── cpu/               # CPU emulation
│   │   ├── executor.rs    # Instruction execution
│   │   ├── instruction.rs # Instruction definitions
│   │   └── state.rs       # CPU state management
│   ├── lib.rs             # Library root
│   └── wasm.rs            # WASM bindings
├── components/            # Shared Yew UI components
│   └── src/
│       ├── components/    # UI components (header, sidebar, etc.)
│       └── lib.rs
├── shared/                # Shared challenge validation types
├── styles/                # CSS stylesheets
├── docs/                  # Documentation
├── images/                # Screenshots
├── index.html             # HTML entry point
├── Trunk.toml             # Trunk configuration
└── Cargo.toml             # Workspace configuration
```

## Instruction Set Summary

### RR Format (Register-Register)
| Opcode | Mnemonic | Operation |
|--------|----------|-----------|
| 18 | LR | Load Register: R1 = R2 |
| 1A | AR | Add Register: R1 = R1 + R2 |
| 1B | SR | Subtract Register: R1 = R1 - R2 |
| 1C | MR | Multiply Register: R1 = R1 * R2 |
| 1D | DR | Divide Register: R1 = R1 / R2 |
| 19 | CR | Compare Register: CC = R1 compare R2 |
| 07 | BR | Branch Register: PC = R2 |

### RX Format (Register-Indexed Storage)
| Opcode | Mnemonic | Operation |
|--------|----------|-----------|
| 58 | L | Load: R1 = Memory[D2(X2,B2)] |
| 50 | ST | Store: Memory[D2(X2,B2)] = R1 |
| 5A | A | Add: R1 = R1 + Memory[D2(X2,B2)] |
| 5B | S | Subtract: R1 = R1 - Memory[D2(X2,B2)] |
| 5C | M | Multiply: R1 = R1 * Memory[D2(X2,B2)] |
| 5D | D | Divide: R1 = R1 / Memory[D2(X2,B2)] |
| 59 | C | Compare: CC = R1 compare Memory[D2(X2,B2)] |

### SI Format (Storage-Immediate)
| Opcode | Mnemonic | Operation |
|--------|----------|-----------|
| 92 | MVI | Move Immediate: Memory[D1(B1)] = I2 |
| 95 | CLI | Compare Logical Immediate |

### S Format (Storage)
| Opcode | Mnemonic | Operation |
|--------|----------|-----------|
| 9E | HIO | Halt I/O (used as HALT) |

## References

### IBM Documentation

- **[IBM ESA/390 Principles of Operation (SA22-7201)](https://publibfp.dhe.ibm.com/epubs/pdf/dz9zr004.pdf)** - The comprehensive reference for ESA/390 architecture
- **[IBM z/Architecture Principles of Operation](https://www.ibm.com/docs/en/zos/latest?topic=zarchitecture-principles-operation)** - Modern successor documentation

### Historical Context

The IBM ESA/390 architecture, introduced in 1990:
- Extended the System/370 architecture with 31-bit addressing (2GB address space)
- Added data spaces and access registers for virtual memory management
- Served as the foundation for all IBM mainframe systems through the 1990s
- Evolved into z/Architecture (64-bit) in 2000, still used in modern IBM Z mainframes
- The instruction formats (RR, RX, SI, S) remain unchanged in today's z/Architecture

## License

MIT
