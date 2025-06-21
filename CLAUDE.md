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

This is a GameBoy (DMG) emulator written in Rust, currently implementing Phase 2 (memory system). The architecture follows GameBoy hardware organization:

### Memory System Structure
- **Peripherals** (`src/peripherals.rs`) - Main memory bus that handles address decoding and routing
- **BootROM** (`src/memory/bootrom.rs`) - 256-byte boot ROM with disable functionality
- **WorkRAM** (`src/memory/wram.rs`) - 8KB work RAM with echo region support
- **HighRAM** (`src/memory/hram.rs`) - 127-byte high-speed RAM
- **Memory Map** (`src/memory_map.rs`) - Centralized memory address definitions and utilities

### Key Design Patterns
- Memory components are accessed through the `Peripherals` bus which handles address decoding
- All memory addresses use constants from `memory_map.rs` module (e.g., `dmg::BOOTROM_START`)
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
- Integration tests in `Peripherals` verify memory bus functionality
- Tests cover both normal operation and edge cases
- Memory statistics are used for verification

## Development Notes

### Current Implementation Status
This emulator is in Phase 2 development focusing on memory system implementation. The next phase (Phase 3) will add basic CPU functionality.

### Code Organization
- Main entry point demonstrates memory system functionality
- Japanese comments are used throughout (original development language)
- Memory tracing can be enabled via the `trace_memory` feature flag
- All memory access goes through the centralized `Peripherals` struct

### Important Considerations
- Memory access patterns follow GameBoy hardware behavior
- BootROM becomes inaccessible once disabled (hardware accurate)
- Work RAM echo region mirrors WRAM for hardware compatibility
- 16-bit memory access uses little-endian byte order