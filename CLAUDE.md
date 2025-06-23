# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Common Development Commands

### Building and Running
- `cargo run` - Run the emulator with dummy BootROM
- `cargo run <bootrom_file>` - Run with real BootROM file
- `cargo test` - Run all tests
- `cargo test -- --nocapture` - Run tests with output visible
- `cargo check` - Fast compilation check without building executable

### Memory Tracing
- `cargo run --features trace_memory` - Enable memory access tracing for debugging

## Architecture Overview

This is a GameBoy (DMG) emulator written in Rust, currently implementing Phase 3 (basic CPU). The architecture follows GameBoy hardware organization:

### System Architecture

#### Memory System
- **Peripherals** (`src/peripherals.rs`) - Main memory bus that handles address decoding and routing
- **BootROM** (`src/memory/bootrom.rs`) - 256-byte boot ROM with disable functionality
- **WorkRAM** (`src/memory/wram.rs`) - 8KB work RAM with echo region support
- **HighRAM** (`src/memory/hram.rs`) - 127-byte high-speed RAM
- **Memory Map** (`src/memory_map.rs`) - Centralized memory address definitions and utilities

#### CPU System
- **CPU Core** (`src/cpu/mod.rs`) - Main CPU with fetch-decode-execute cycle
- **Registers** (`src/cpu/registers.rs`) - 8-bit/16-bit registers and flag management
- **Instructions** (`src/cpu/instructions.rs`) - Instruction definitions and opcodes
- **Decoder** (`src/cpu/decoder.rs`) - Instruction decoding and error handling

### Key Design Patterns
- Memory components are accessed through the `Peripherals` bus which handles address decoding
- All memory addresses use constants from `memory_map.rs` module (e.g., `dmg::BOOTROM_START`)
- CPU instructions use modular design with separate decode and execute phases
- Register access through both individual 8-bit and combined 16-bit interfaces
- Memory statistics tracking for debugging and analysis
- Conditional compilation features for debug tracing
- Error handling using Result types for file operations

### Memory Layout (GameBoy DMG)
- 0x0000-0x00FF: BootROM (256B) - disabled via 0xFF50 register
- 0x0100-0x7FFF: Cartridge ROM (not yet implemented)
- 0x8000-0x9FFF: Video RAM (not yet implemented)
- 0xA000-0xBFFF: Cartridge RAM (not yet implemented)
- 0xC000-0xDFFF: Work RAM (8KB)
- 0xE000-0xFDFF: Work RAM Echo (mirror of WRAM)
- 0xFF00-0xFF7F: I/O Registers (partially implemented)
- 0xFF80-0xFFFE: High RAM (127B)
- 0xFFFF: Interrupt Enable register

### Testing Strategy
- Each memory component has its own test module
- CPU components have comprehensive unit tests for registers, instructions, and decoder
- Integration tests in `Peripherals` verify memory bus functionality
- CPU execution tests verify fetch-decode-execute cycle
- Tests cover both normal operation and edge cases
- Memory statistics are used for verification

## Development Notes

### Current Implementation Status
This emulator has completed Phase 3 development with basic CPU functionality. Current capabilities:
- Complete memory system (Phase 2)
- Basic CPU with fetch-decode-execute cycle (Phase 3)
- Support for fundamental instructions: NOP, LD (immediate), JP, JR
- Register management with 8-bit/16-bit access patterns
- Simple program execution and loop detection

Next phase (Phase 4) would add more instructions, interrupts, and peripheral devices.

### Code Organization
- Main entry point demonstrates both memory system and CPU functionality
- Japanese comments are used throughout (original development language)
- Memory tracing can be enabled via the `trace_memory` feature flag
- All memory access goes through the centralized `Peripherals` struct
- CPU tests run in WRAM region to avoid BootROM access conflicts

### Important Considerations
- Memory access patterns follow GameBoy hardware behavior
- BootROM becomes inaccessible once disabled (hardware accurate)
- Work RAM echo region mirrors WRAM for hardware compatibility
- 16-bit memory access uses little-endian byte order
- CPU registers support both 8-bit individual access and 16-bit pair access
- Flag register (F) automatically masks lower 4 bits (hardware accurate)
- PC and SP are managed automatically during instruction execution

### Implemented Instructions
Current CPU supports these GameBoy instructions:
- `0x00` NOP - No operation
- `0x3E` LD A,n - Load immediate to A register
- `0x06` LD B,n - Load immediate to B register  
- `0x0E` LD C,n - Load immediate to C register
- `0x16` LD D,n - Load immediate to D register
- `0x1E` LD E,n - Load immediate to E register
- `0x26` LD H,n - Load immediate to H register
- `0x2E` LD L,n - Load immediate to L register
- `0x31` LD SP,nn - Load immediate to stack pointer
- `0xC3` JP nn - Absolute jump
- `0x18` JR n - Relative jump

### Development Guidelines
- このドキュメントは日本語で書くこと
- すべてのファイルは最終行に空の行を追加すること
- CPU tests should use WRAM region (0xC000+) to avoid BootROM conflicts
- Register pair functions should maintain hardware-accurate bit masking