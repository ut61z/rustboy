// src/cpu/mod.rs
// GameBoy CPU (Sharp LR35902) の実装

pub mod registers;
pub mod instructions;
pub mod decoder;
pub mod interrupts;
pub mod timer;

pub use registers::Registers;
use crate::peripherals::Peripherals;
use interrupts::{get_pending_interrupt, has_pending_interrupt};

/// GameBoy CPU の状態
pub struct Cpu {
    /// CPUレジスタ
    pub registers: Registers,
    /// 割り込みマスター有効フラグ
    pub ime: bool,
    /// EI命令後の1命令遅延フラグ
    pub ime_pending: bool,
    /// 停止状態
    pub halted: bool,
    /// 命令実行カウンタ（デバッグ用）
    pub instruction_count: u64,
}

impl Cpu {
    /// 新しいCPUを作成
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            ime: false,
            ime_pending: false,
            halted: false,
            instruction_count: 0,
        }
    }

    /// CPUを初期状態にリセット
    pub fn reset(&mut self) {
        self.registers.reset();
        self.ime = false;
        self.ime_pending = false;
        self.halted = false;
        self.instruction_count = 0;
    }

    /// 1命令を実行（割り込みチェック込み）
    pub fn step(&mut self, peripherals: &mut Peripherals) -> Result<u8, String> {
        // 割り込み処理
        let interrupt_cycles = self.handle_interrupts(peripherals);
        if interrupt_cycles > 0 {
            return Ok(interrupt_cycles);
        }

        if self.halted {
            // HALT状態: 割り込みが来るまで何もしない
            return Ok(4);
        }

        // EI命令の遅延適用
        if self.ime_pending {
            self.ime = true;
            self.ime_pending = false;
        }

        // フェッチ
        let opcode = self.fetch_byte(peripherals);

        // デコード・実行
        let cycles = self.execute_instruction(opcode, peripherals)?;

        self.instruction_count += 1;

        Ok(cycles)
    }

    /// 割り込みの処理。割り込み処理した場合はサイクル数を返す
    fn handle_interrupts(&mut self, peripherals: &mut Peripherals) -> u8 {
        let if_reg = peripherals.interrupt_flag;
        let ie_reg = peripherals.interrupt_enable;

        // HALT状態の復帰チェック（IMEに関係なく）
        if self.halted && has_pending_interrupt(if_reg, ie_reg) {
            self.halted = false;
        }

        // IMEが無効なら割り込みディスパッチしない
        if !self.ime {
            return 0;
        }

        if let Some(interrupt) = get_pending_interrupt(if_reg, ie_reg) {
            // IME無効化
            self.ime = false;
            self.ime_pending = false;

            // IFの該当ビットをクリア
            peripherals.interrupt_flag &= !interrupt.mask();

            // PCをスタックにプッシュ
            self.push_word(peripherals, self.registers.pc);

            // 割り込みハンドラにジャンプ
            self.registers.pc = interrupt.handler_address();

            // 割り込み処理は20サイクル
            20
        } else {
            0
        }
    }

    /// 16bit値をスタックにプッシュ
    fn push_word(&mut self, peripherals: &mut Peripherals, value: u16) {
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        peripherals.write(self.registers.sp, (value >> 8) as u8);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        peripherals.write(self.registers.sp, value as u8);
    }

    /// スタックから16bit値をポップ
    fn pop_word(&mut self, peripherals: &mut Peripherals) -> u16 {
        let low = peripherals.read(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high = peripherals.read(self.registers.sp) as u16;
        self.registers.sp = self.registers.sp.wrapping_add(1);
        (high << 8) | low
    }
    
    /// 1バイトをフェッチしてPCをインクリメント
    fn fetch_byte(&mut self, peripherals: &mut Peripherals) -> u8 {
        let value = peripherals.read(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);
        value
    }
    
    /// 2バイトをフェッチしてPCをインクリメント（リトルエンディアン）
    fn fetch_word(&mut self, peripherals: &mut Peripherals) -> u16 {
        let low = self.fetch_byte(peripherals) as u16;
        let high = self.fetch_byte(peripherals) as u16;
        (high << 8) | low
    }
    
    /// 8bitレジスタ値を取得（オペコードの下位3bitから）
    fn get_r8(&self, index: u8, peripherals: &mut Peripherals) -> u8 {
        match index {
            0 => self.registers.b,
            1 => self.registers.c,
            2 => self.registers.d,
            3 => self.registers.e,
            4 => self.registers.h,
            5 => self.registers.l,
            6 => { // (HL)
                let addr = self.registers.get_hl();
                peripherals.read(addr)
            }
            7 => self.registers.a,
            _ => unreachable!(),
        }
    }

    /// 8bitレジスタに値を設定（オペコードの下位3bitから）
    fn set_r8(&mut self, index: u8, value: u8, peripherals: &mut Peripherals) {
        match index {
            0 => self.registers.b = value,
            1 => self.registers.c = value,
            2 => self.registers.d = value,
            3 => self.registers.e = value,
            4 => self.registers.h = value,
            5 => self.registers.l = value,
            6 => { // (HL)
                let addr = self.registers.get_hl();
                peripherals.write(addr, value);
            }
            7 => self.registers.a = value,
            _ => unreachable!(),
        }
    }

    /// 命令を実行
    fn execute_instruction(&mut self, opcode: u8, peripherals: &mut Peripherals) -> Result<u8, String> {
        match opcode {
            // ===== NOP =====
            0x00 => Ok(4),

            // ===== 16ビットロード命令 =====
            0x01 => { // LD BC, nn
                let v = self.fetch_word(peripherals);
                self.registers.set_bc(v);
                Ok(12)
            }
            0x11 => { // LD DE, nn
                let v = self.fetch_word(peripherals);
                self.registers.set_de(v);
                Ok(12)
            }
            0x21 => { // LD HL, nn
                let v = self.fetch_word(peripherals);
                self.registers.set_hl(v);
                Ok(12)
            }
            0x31 => { // LD SP, nn
                self.registers.sp = self.fetch_word(peripherals);
                Ok(12)
            }

            // ===== 8ビット即値ロード =====
            0x06 => { let v = self.fetch_byte(peripherals); self.registers.b = v; Ok(8) }
            0x0E => { let v = self.fetch_byte(peripherals); self.registers.c = v; Ok(8) }
            0x16 => { let v = self.fetch_byte(peripherals); self.registers.d = v; Ok(8) }
            0x1E => { let v = self.fetch_byte(peripherals); self.registers.e = v; Ok(8) }
            0x26 => { let v = self.fetch_byte(peripherals); self.registers.h = v; Ok(8) }
            0x2E => { let v = self.fetch_byte(peripherals); self.registers.l = v; Ok(8) }
            0x36 => { // LD (HL), n
                let v = self.fetch_byte(peripherals);
                let addr = self.registers.get_hl();
                peripherals.write(addr, v);
                Ok(12)
            }
            0x3E => { let v = self.fetch_byte(peripherals); self.registers.a = v; Ok(8) }

            // ===== LD r, r' (0x40-0x7F) — HALT(0x76)以外 =====
            0x40..=0x75 | 0x77..=0x7F => {
                let dst = (opcode >> 3) & 0x07;
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.set_r8(dst, value, peripherals);
                let cycles = if src == 6 || dst == 6 { 8 } else { 4 };
                Ok(cycles)
            }

            // ===== HALT =====
            0x76 => {
                self.halted = true;
                Ok(4)
            }

            // ===== LD (BC), A / LD (DE), A =====
            0x02 => {
                let addr = self.registers.get_bc();
                peripherals.write(addr, self.registers.a);
                Ok(8)
            }
            0x12 => {
                let addr = self.registers.get_de();
                peripherals.write(addr, self.registers.a);
                Ok(8)
            }

            // ===== LD A, (BC) / LD A, (DE) =====
            0x0A => {
                let addr = self.registers.get_bc();
                self.registers.a = peripherals.read(addr);
                Ok(8)
            }
            0x1A => {
                let addr = self.registers.get_de();
                self.registers.a = peripherals.read(addr);
                Ok(8)
            }

            // ===== LD (HL+), A / LD (HL-), A =====
            0x22 => { // LD (HL+), A
                let addr = self.registers.get_hl();
                peripherals.write(addr, self.registers.a);
                self.registers.set_hl(addr.wrapping_add(1));
                Ok(8)
            }
            0x32 => { // LD (HL-), A
                let addr = self.registers.get_hl();
                peripherals.write(addr, self.registers.a);
                self.registers.set_hl(addr.wrapping_sub(1));
                Ok(8)
            }

            // ===== LD A, (HL+) / LD A, (HL-) =====
            0x2A => { // LD A, (HL+)
                let addr = self.registers.get_hl();
                self.registers.a = peripherals.read(addr);
                self.registers.set_hl(addr.wrapping_add(1));
                Ok(8)
            }
            0x3A => { // LD A, (HL-)
                let addr = self.registers.get_hl();
                self.registers.a = peripherals.read(addr);
                self.registers.set_hl(addr.wrapping_sub(1));
                Ok(8)
            }

            // ===== LD (nn), A / LD A, (nn) =====
            0xEA => { // LD (nn), A
                let addr = self.fetch_word(peripherals);
                peripherals.write(addr, self.registers.a);
                Ok(16)
            }
            0xFA => { // LD A, (nn)
                let addr = self.fetch_word(peripherals);
                self.registers.a = peripherals.read(addr);
                Ok(16)
            }

            // ===== LDH (n), A / LDH A, (n) =====
            0xE0 => { // LDH (n), A — LD (0xFF00+n), A
                let offset = self.fetch_byte(peripherals) as u16;
                peripherals.write(0xFF00 + offset, self.registers.a);
                Ok(12)
            }
            0xF0 => { // LDH A, (n) — LD A, (0xFF00+n)
                let offset = self.fetch_byte(peripherals) as u16;
                self.registers.a = peripherals.read(0xFF00 + offset);
                Ok(12)
            }

            // ===== LD (C), A / LD A, (C) =====
            0xE2 => { // LD (0xFF00+C), A
                peripherals.write(0xFF00 + self.registers.c as u16, self.registers.a);
                Ok(8)
            }
            0xF2 => { // LD A, (0xFF00+C)
                self.registers.a = peripherals.read(0xFF00 + self.registers.c as u16);
                Ok(8)
            }

            // ===== LD (nn), SP =====
            0x08 => {
                let addr = self.fetch_word(peripherals);
                peripherals.write(addr, self.registers.sp as u8);
                peripherals.write(addr.wrapping_add(1), (self.registers.sp >> 8) as u8);
                Ok(20)
            }

            // ===== LD SP, HL =====
            0xF9 => {
                self.registers.sp = self.registers.get_hl();
                Ok(8)
            }

            // ===== LD HL, SP+n =====
            0xF8 => {
                let offset = self.fetch_byte(peripherals) as i8 as i16;
                let sp = self.registers.sp;
                let result = (sp as i16).wrapping_add(offset) as u16;
                self.registers.f = 0;
                // Half-carry: 下位4bit同士の加算
                if (sp & 0x0F) + (offset as u16 & 0x0F) > 0x0F {
                    self.registers.f |= 0x20;
                }
                // Carry: 下位8bit同士の加算
                if (sp & 0xFF) + (offset as u16 & 0xFF) > 0xFF {
                    self.registers.f |= 0x10;
                }
                self.registers.set_hl(result);
                Ok(12)
            }

            // ===== PUSH =====
            0xC5 => { let v = self.registers.get_bc(); self.push_word(peripherals, v); Ok(16) }
            0xD5 => { let v = self.registers.get_de(); self.push_word(peripherals, v); Ok(16) }
            0xE5 => { let v = self.registers.get_hl(); self.push_word(peripherals, v); Ok(16) }
            0xF5 => { let v = self.registers.get_af(); self.push_word(peripherals, v); Ok(16) }

            // ===== POP =====
            0xC1 => { let v = self.pop_word(peripherals); self.registers.set_bc(v); Ok(12) }
            0xD1 => { let v = self.pop_word(peripherals); self.registers.set_de(v); Ok(12) }
            0xE1 => { let v = self.pop_word(peripherals); self.registers.set_hl(v); Ok(12) }
            0xF1 => { let v = self.pop_word(peripherals); self.registers.set_af(v); Ok(12) }

            // ===== 8ビット算術: ADD A, r =====
            0x80..=0x87 => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_add(value, false);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xC6 => { // ADD A, n
                let v = self.fetch_byte(peripherals);
                self.alu_add(v, false);
                Ok(8)
            }

            // ===== ADC A, r =====
            0x88..=0x8F => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_add(value, true);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xCE => { // ADC A, n
                let v = self.fetch_byte(peripherals);
                self.alu_add(v, true);
                Ok(8)
            }

            // ===== SUB r =====
            0x90..=0x97 => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_sub(value, false);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xD6 => { // SUB n
                let v = self.fetch_byte(peripherals);
                self.alu_sub(v, false);
                Ok(8)
            }

            // ===== SBC A, r =====
            0x98..=0x9F => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_sub(value, true);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xDE => { // SBC A, n
                let v = self.fetch_byte(peripherals);
                self.alu_sub(v, true);
                Ok(8)
            }

            // ===== AND r =====
            0xA0..=0xA7 => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_and(value);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xE6 => { // AND n
                let v = self.fetch_byte(peripherals);
                self.alu_and(v);
                Ok(8)
            }

            // ===== XOR r =====
            0xA8..=0xAF => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_xor(value);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xEE => { // XOR n
                let v = self.fetch_byte(peripherals);
                self.alu_xor(v);
                Ok(8)
            }

            // ===== OR r =====
            0xB0..=0xB7 => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_or(value);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xF6 => { // OR n
                let v = self.fetch_byte(peripherals);
                self.alu_or(v);
                Ok(8)
            }

            // ===== CP r =====
            0xB8..=0xBF => {
                let src = opcode & 0x07;
                let value = self.get_r8(src, peripherals);
                self.alu_cp(value);
                Ok(if src == 6 { 8 } else { 4 })
            }
            0xFE => { // CP n
                let v = self.fetch_byte(peripherals);
                self.alu_cp(v);
                Ok(8)
            }

            // ===== INC r8 =====
            0x04 => { self.registers.b = self.alu_inc(self.registers.b); Ok(4) }
            0x0C => { self.registers.c = self.alu_inc(self.registers.c); Ok(4) }
            0x14 => { self.registers.d = self.alu_inc(self.registers.d); Ok(4) }
            0x1C => { self.registers.e = self.alu_inc(self.registers.e); Ok(4) }
            0x24 => { self.registers.h = self.alu_inc(self.registers.h); Ok(4) }
            0x2C => { self.registers.l = self.alu_inc(self.registers.l); Ok(4) }
            0x34 => { // INC (HL)
                let addr = self.registers.get_hl();
                let v = peripherals.read(addr);
                let result = self.alu_inc(v);
                peripherals.write(addr, result);
                Ok(12)
            }
            0x3C => { self.registers.a = self.alu_inc(self.registers.a); Ok(4) }

            // ===== DEC r8 =====
            0x05 => { self.registers.b = self.alu_dec(self.registers.b); Ok(4) }
            0x0D => { self.registers.c = self.alu_dec(self.registers.c); Ok(4) }
            0x15 => { self.registers.d = self.alu_dec(self.registers.d); Ok(4) }
            0x1D => { self.registers.e = self.alu_dec(self.registers.e); Ok(4) }
            0x25 => { self.registers.h = self.alu_dec(self.registers.h); Ok(4) }
            0x2D => { self.registers.l = self.alu_dec(self.registers.l); Ok(4) }
            0x35 => { // DEC (HL)
                let addr = self.registers.get_hl();
                let v = peripherals.read(addr);
                let result = self.alu_dec(v);
                peripherals.write(addr, result);
                Ok(12)
            }
            0x3D => { self.registers.a = self.alu_dec(self.registers.a); Ok(4) }

            // ===== DAA =====
            0x27 => { self.alu_daa(); Ok(4) }

            // ===== CPL =====
            0x2F => {
                self.registers.a = !self.registers.a;
                self.registers.f |= 0x60; // N=1, H=1
                Ok(4)
            }

            // ===== SCF =====
            0x37 => {
                self.registers.f = (self.registers.f & 0x80) | 0x10; // N=0, H=0, C=1
                Ok(4)
            }

            // ===== CCF =====
            0x3F => {
                let carry = (self.registers.f & 0x10) ^ 0x10;
                self.registers.f = (self.registers.f & 0x80) | carry; // N=0, H=0, C=toggle
                Ok(4)
            }

            // ===== 16ビット算術: ADD HL, rr =====
            0x09 => { self.alu_add_hl(self.registers.get_bc()); Ok(8) }
            0x19 => { self.alu_add_hl(self.registers.get_de()); Ok(8) }
            0x29 => { let hl = self.registers.get_hl(); self.alu_add_hl(hl); Ok(8) }
            0x39 => { self.alu_add_hl(self.registers.sp); Ok(8) }

            // ===== INC rr =====
            0x03 => { self.registers.set_bc(self.registers.get_bc().wrapping_add(1)); Ok(8) }
            0x13 => { self.registers.set_de(self.registers.get_de().wrapping_add(1)); Ok(8) }
            0x23 => { self.registers.set_hl(self.registers.get_hl().wrapping_add(1)); Ok(8) }
            0x33 => { self.registers.sp = self.registers.sp.wrapping_add(1); Ok(8) }

            // ===== DEC rr =====
            0x0B => { self.registers.set_bc(self.registers.get_bc().wrapping_sub(1)); Ok(8) }
            0x1B => { self.registers.set_de(self.registers.get_de().wrapping_sub(1)); Ok(8) }
            0x2B => { self.registers.set_hl(self.registers.get_hl().wrapping_sub(1)); Ok(8) }
            0x3B => { self.registers.sp = self.registers.sp.wrapping_sub(1); Ok(8) }

            // ===== ADD SP, n =====
            0xE8 => {
                let offset = self.fetch_byte(peripherals) as i8 as i16;
                let sp = self.registers.sp;
                let result = (sp as i16).wrapping_add(offset) as u16;
                self.registers.f = 0;
                if (sp & 0x0F) + (offset as u16 & 0x0F) > 0x0F {
                    self.registers.f |= 0x20;
                }
                if (sp & 0xFF) + (offset as u16 & 0xFF) > 0xFF {
                    self.registers.f |= 0x10;
                }
                self.registers.sp = result;
                Ok(16)
            }

            // ===== ローテート（メイン） =====
            0x07 => { // RLCA
                let carry = self.registers.a >> 7;
                self.registers.a = (self.registers.a << 1) | carry;
                self.registers.f = carry << 4; // Z=0, N=0, H=0, C=old bit7
                Ok(4)
            }
            0x0F => { // RRCA
                let carry = self.registers.a & 1;
                self.registers.a = (self.registers.a >> 1) | (carry << 7);
                self.registers.f = carry << 4; // Z=0, N=0, H=0, C=old bit0
                Ok(4)
            }
            0x17 => { // RLA
                let old_carry = (self.registers.f >> 4) & 1;
                let new_carry = self.registers.a >> 7;
                self.registers.a = (self.registers.a << 1) | old_carry;
                self.registers.f = new_carry << 4;
                Ok(4)
            }
            0x1F => { // RRA
                let old_carry = (self.registers.f >> 4) & 1;
                let new_carry = self.registers.a & 1;
                self.registers.a = (self.registers.a >> 1) | (old_carry << 7);
                self.registers.f = new_carry << 4;
                Ok(4)
            }

            // ===== ジャンプ命令 =====
            0xC3 => { // JP nn
                let addr = self.fetch_word(peripherals);
                self.registers.pc = addr;
                Ok(16)
            }
            0xE9 => { // JP (HL)
                self.registers.pc = self.registers.get_hl();
                Ok(4)
            }

            // ===== 条件付きJP =====
            0xC2 => { // JP NZ, nn
                let addr = self.fetch_word(peripherals);
                if !self.registers.get_flag_z() { self.registers.pc = addr; Ok(16) } else { Ok(12) }
            }
            0xCA => { // JP Z, nn
                let addr = self.fetch_word(peripherals);
                if self.registers.get_flag_z() { self.registers.pc = addr; Ok(16) } else { Ok(12) }
            }
            0xD2 => { // JP NC, nn
                let addr = self.fetch_word(peripherals);
                if !self.registers.get_flag_c() { self.registers.pc = addr; Ok(16) } else { Ok(12) }
            }
            0xDA => { // JP C, nn
                let addr = self.fetch_word(peripherals);
                if self.registers.get_flag_c() { self.registers.pc = addr; Ok(16) } else { Ok(12) }
            }

            // ===== JR =====
            0x18 => { // JR n
                let offset = self.fetch_byte(peripherals) as i8;
                self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                Ok(12)
            }

            // ===== 条件付きJR =====
            0x20 => { // JR NZ, n
                let offset = self.fetch_byte(peripherals) as i8;
                if !self.registers.get_flag_z() {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                    Ok(12)
                } else { Ok(8) }
            }
            0x28 => { // JR Z, n
                let offset = self.fetch_byte(peripherals) as i8;
                if self.registers.get_flag_z() {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                    Ok(12)
                } else { Ok(8) }
            }
            0x30 => { // JR NC, n
                let offset = self.fetch_byte(peripherals) as i8;
                if !self.registers.get_flag_c() {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                    Ok(12)
                } else { Ok(8) }
            }
            0x38 => { // JR C, n
                let offset = self.fetch_byte(peripherals) as i8;
                if self.registers.get_flag_c() {
                    self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                    Ok(12)
                } else { Ok(8) }
            }

            // ===== CALL =====
            0xCD => { // CALL nn
                let addr = self.fetch_word(peripherals);
                self.push_word(peripherals, self.registers.pc);
                self.registers.pc = addr;
                Ok(24)
            }

            // ===== 条件付きCALL =====
            0xC4 => { // CALL NZ, nn
                let addr = self.fetch_word(peripherals);
                if !self.registers.get_flag_z() {
                    self.push_word(peripherals, self.registers.pc);
                    self.registers.pc = addr;
                    Ok(24)
                } else { Ok(12) }
            }
            0xCC => { // CALL Z, nn
                let addr = self.fetch_word(peripherals);
                if self.registers.get_flag_z() {
                    self.push_word(peripherals, self.registers.pc);
                    self.registers.pc = addr;
                    Ok(24)
                } else { Ok(12) }
            }
            0xD4 => { // CALL NC, nn
                let addr = self.fetch_word(peripherals);
                if !self.registers.get_flag_c() {
                    self.push_word(peripherals, self.registers.pc);
                    self.registers.pc = addr;
                    Ok(24)
                } else { Ok(12) }
            }
            0xDC => { // CALL C, nn
                let addr = self.fetch_word(peripherals);
                if self.registers.get_flag_c() {
                    self.push_word(peripherals, self.registers.pc);
                    self.registers.pc = addr;
                    Ok(24)
                } else { Ok(12) }
            }

            // ===== RET =====
            0xC9 => { // RET
                self.registers.pc = self.pop_word(peripherals);
                Ok(16)
            }

            // ===== 条件付きRET =====
            0xC0 => { // RET NZ
                if !self.registers.get_flag_z() {
                    self.registers.pc = self.pop_word(peripherals);
                    Ok(20)
                } else { Ok(8) }
            }
            0xC8 => { // RET Z
                if self.registers.get_flag_z() {
                    self.registers.pc = self.pop_word(peripherals);
                    Ok(20)
                } else { Ok(8) }
            }
            0xD0 => { // RET NC
                if !self.registers.get_flag_c() {
                    self.registers.pc = self.pop_word(peripherals);
                    Ok(20)
                } else { Ok(8) }
            }
            0xD8 => { // RET C
                if self.registers.get_flag_c() {
                    self.registers.pc = self.pop_word(peripherals);
                    Ok(20)
                } else { Ok(8) }
            }

            // ===== RETI =====
            0xD9 => {
                self.registers.pc = self.pop_word(peripherals);
                self.ime = true;
                Ok(16)
            }

            // ===== RST =====
            0xC7 => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x00; Ok(16) }
            0xCF => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x08; Ok(16) }
            0xD7 => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x10; Ok(16) }
            0xDF => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x18; Ok(16) }
            0xE7 => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x20; Ok(16) }
            0xEF => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x28; Ok(16) }
            0xF7 => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x30; Ok(16) }
            0xFF => { self.push_word(peripherals, self.registers.pc); self.registers.pc = 0x38; Ok(16) }

            // ===== DI / EI =====
            0xF3 => { // DI
                self.ime = false;
                self.ime_pending = false;
                Ok(4)
            }
            0xFB => { // EI（1命令遅延で有効化）
                self.ime_pending = true;
                Ok(4)
            }

            // ===== CB-prefix =====
            0xCB => {
                let cb_opcode = self.fetch_byte(peripherals);
                self.execute_cb(cb_opcode, peripherals)
            }

            _ => Err(format!("未実装の命令: 0x{:02X} at PC=0x{:04X}", opcode, self.registers.pc.wrapping_sub(1)))
        }
    }

    // ===== ALU ヘルパーメソッド =====

    /// ADD A, value (with_carry = false) / ADC A, value (with_carry = true)
    fn alu_add(&mut self, value: u8, with_carry: bool) {
        let carry = if with_carry && (self.registers.f & 0x10 != 0) { 1u16 } else { 0 };
        let a = self.registers.a as u16;
        let result = a + value as u16 + carry;
        let half = (a & 0x0F) + (value as u16 & 0x0F) + carry;

        self.registers.a = result as u8;
        self.registers.f = 0;
        if self.registers.a == 0 { self.registers.f |= 0x80; } // Z
        if half > 0x0F { self.registers.f |= 0x20; }           // H
        if result > 0xFF { self.registers.f |= 0x10; }         // C
    }

    /// SUB value (with_carry = false) / SBC A, value (with_carry = true)
    fn alu_sub(&mut self, value: u8, with_carry: bool) {
        let carry = if with_carry && (self.registers.f & 0x10 != 0) { 1u16 } else { 0 };
        let a = self.registers.a as u16;
        let result = a.wrapping_sub(value as u16).wrapping_sub(carry);
        let half = (a & 0x0F).wrapping_sub(value as u16 & 0x0F).wrapping_sub(carry);

        self.registers.a = result as u8;
        self.registers.f = 0x40; // N=1
        if self.registers.a == 0 { self.registers.f |= 0x80; } // Z
        if half > 0x0F { self.registers.f |= 0x20; }           // H (borrow)
        if result > 0xFF { self.registers.f |= 0x10; }         // C (borrow)
    }

    fn alu_and(&mut self, value: u8) {
        self.registers.a &= value;
        self.registers.f = 0x20; // H=1
        if self.registers.a == 0 { self.registers.f |= 0x80; }
    }

    fn alu_xor(&mut self, value: u8) {
        self.registers.a ^= value;
        self.registers.f = 0;
        if self.registers.a == 0 { self.registers.f |= 0x80; }
    }

    fn alu_or(&mut self, value: u8) {
        self.registers.a |= value;
        self.registers.f = 0;
        if self.registers.a == 0 { self.registers.f |= 0x80; }
    }

    fn alu_cp(&mut self, value: u8) {
        let a = self.registers.a;
        self.alu_sub(value, false);
        self.registers.a = a; // CPはAを変更しない
    }

    fn alu_inc(&mut self, value: u8) -> u8 {
        let result = value.wrapping_add(1);
        let carry = self.registers.f & 0x10; // Cフラグを保持
        self.registers.f = carry;
        if result == 0 { self.registers.f |= 0x80; }           // Z
        if (value & 0x0F) + 1 > 0x0F { self.registers.f |= 0x20; } // H
        result
    }

    fn alu_dec(&mut self, value: u8) -> u8 {
        let result = value.wrapping_sub(1);
        let carry = self.registers.f & 0x10; // Cフラグを保持
        self.registers.f = carry | 0x40; // N=1
        if result == 0 { self.registers.f |= 0x80; }           // Z
        if (value & 0x0F) == 0 { self.registers.f |= 0x20; }   // H (borrow)
        result
    }

    fn alu_add_hl(&mut self, value: u16) {
        let hl = self.registers.get_hl();
        let result = (hl as u32) + (value as u32);
        let z = self.registers.f & 0x80; // Zフラグを保持
        self.registers.f = z;
        if (hl & 0x0FFF) + (value & 0x0FFF) > 0x0FFF { self.registers.f |= 0x20; } // H
        if result > 0xFFFF { self.registers.f |= 0x10; } // C
        self.registers.set_hl(result as u16);
    }

    fn alu_daa(&mut self) {
        let mut a = self.registers.a as i16;
        let n = self.registers.f & 0x40 != 0;
        let h = self.registers.f & 0x20 != 0;
        let c = self.registers.f & 0x10 != 0;

        if !n {
            if c || a > 0x99 { a += 0x60; self.registers.f |= 0x10; }
            if h || (a & 0x0F) > 0x09 { a += 0x06; }
        } else {
            if c { a -= 0x60; }
            if h { a -= 0x06; }
        }

        self.registers.a = a as u8;
        self.registers.f &= !0xA0; // H=0 をクリア（Zは下で設定）
        if self.registers.a == 0 { self.registers.f |= 0x80; } else { self.registers.f &= !0x80; }
    }

    // ===== CB-prefix 命令実行 =====
    fn execute_cb(&mut self, opcode: u8, peripherals: &mut Peripherals) -> Result<u8, String> {
        let reg_index = opcode & 0x07;
        let value = self.get_r8(reg_index, peripherals);
        let is_hl = reg_index == 6;

        let result = match opcode {
            // RLC r (0x00-0x07)
            0x00..=0x07 => {
                let carry = value >> 7;
                let r = (value << 1) | carry;
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // RRC r (0x08-0x0F)
            0x08..=0x0F => {
                let carry = value & 1;
                let r = (value >> 1) | (carry << 7);
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // RL r (0x10-0x17)
            0x10..=0x17 => {
                let old_carry = (self.registers.f >> 4) & 1;
                let new_carry = value >> 7;
                let r = (value << 1) | old_carry;
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if new_carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // RR r (0x18-0x1F)
            0x18..=0x1F => {
                let old_carry = (self.registers.f >> 4) & 1;
                let new_carry = value & 1;
                let r = (value >> 1) | (old_carry << 7);
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if new_carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // SLA r (0x20-0x27)
            0x20..=0x27 => {
                let carry = value >> 7;
                let r = value << 1;
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // SRA r (0x28-0x2F)
            0x28..=0x2F => {
                let carry = value & 1;
                let r = (value >> 1) | (value & 0x80); // 符号ビット保持
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // SWAP r (0x30-0x37)
            0x30..=0x37 => {
                let r = (value >> 4) | (value << 4);
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                Some(r)
            }
            // SRL r (0x38-0x3F)
            0x38..=0x3F => {
                let carry = value & 1;
                let r = value >> 1;
                self.registers.f = 0;
                if r == 0 { self.registers.f |= 0x80; }
                if carry != 0 { self.registers.f |= 0x10; }
                Some(r)
            }
            // BIT b, r (0x40-0x7F)
            0x40..=0x7F => {
                let bit = (opcode >> 3) & 0x07;
                let z = if value & (1 << bit) == 0 { 0x80 } else { 0x00 };
                self.registers.f = z | 0x20 | (self.registers.f & 0x10); // Z, N=0, H=1, C=keep
                None // BITはレジスタを変更しない
            }
            // RES b, r (0x80-0xBF)
            0x80..=0xBF => {
                let bit = (opcode >> 3) & 0x07;
                Some(value & !(1 << bit))
            }
            // SET b, r (0xC0-0xFF)
            0xC0..=0xFF => {
                let bit = (opcode >> 3) & 0x07;
                Some(value | (1 << bit))
            }
        };

        if let Some(r) = result {
            self.set_r8(reg_index, r, peripherals);
        }

        Ok(if is_hl { 16 } else { 8 })
    }
    
    /// CPUの状態をデバッグ出力用の文字列で取得
    pub fn debug_string(&self) -> String {
        format!(
            "PC:{:04X} SP:{:04X} A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} | {:08}",
            self.registers.pc,
            self.registers.sp,
            self.registers.a,
            self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.instruction_count
        )
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::BootRom;
    
    fn create_test_system() -> (Cpu, Peripherals) {
        let cpu = Cpu::new();
        let mut peripherals = Peripherals::new(BootRom::new_dummy());
        // BootROMを無効化してテスト用メモリアクセスを可能にする
        peripherals.write(0xFF50, 0x01);
        (cpu, peripherals)
    }
    
    #[test]
    fn test_cpu_creation() {
        let cpu = Cpu::new();
        assert_eq!(cpu.registers.pc, 0x0000);
        assert_eq!(cpu.instruction_count, 0);
    }

    #[test]
    fn test_nop_instruction() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0x00);
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0xC001);
        assert_eq!(cpu.instruction_count, 1);
    }

    #[test]
    fn test_ld_a_n_instruction() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0x3E);
        peripherals.write(0xC001, 0x42);
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 8);
        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn test_jp_nn_instruction() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0xC3);
        peripherals.write(0xC001, 0x34);
        peripherals.write(0xC002, 0x12);
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 16);
        assert_eq!(cpu.registers.pc, 0x1234);
    }

    #[test]
    fn test_ld_r_r() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.b = 0x42;
        peripherals.write(0xC000, 0x78); // LD A, B
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn test_ld_16bit() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0x01); // LD BC, nn
        peripherals.write(0xC001, 0x34);
        peripherals.write(0xC002, 0x12);
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.get_bc(), 0x1234);
    }

    #[test]
    fn test_push_pop() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xDFF0;
        cpu.registers.set_bc(0xABCD);
        peripherals.write(0xC000, 0xC5); // PUSH BC
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.sp, 0xDFEE);
        peripherals.write(0xC001, 0xD1); // POP DE
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.get_de(), 0xABCD);
        assert_eq!(cpu.registers.sp, 0xDFF0);
    }

    #[test]
    fn test_add_sub() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x10;
        cpu.registers.b = 0x20;
        peripherals.write(0xC000, 0x80); // ADD A, B
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0x30);
        assert!(!cpu.registers.get_flag_z());
        peripherals.write(0xC001, 0x90); // SUB B
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0x10);
    }

    #[test]
    fn test_xor_zero_flag() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0xFF;
        peripherals.write(0xC000, 0xAF); // XOR A
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0x00);
        assert!(cpu.registers.get_flag_z());
    }

    #[test]
    fn test_inc_dec() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.b = 0xFF;
        peripherals.write(0xC000, 0x04); // INC B
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.b, 0x00);
        assert!(cpu.registers.get_flag_z());
        peripherals.write(0xC001, 0x05); // DEC B
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.b, 0xFF);
    }

    #[test]
    fn test_call_ret() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xDFF0;
        peripherals.write(0xC000, 0xCD); // CALL 0xC100
        peripherals.write(0xC001, 0x00);
        peripherals.write(0xC002, 0xC1);
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.pc, 0xC100);
        assert_eq!(cpu.registers.sp, 0xDFEE);
        peripherals.write(0xC100, 0xC9); // RET
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.pc, 0xC003);
        assert_eq!(cpu.registers.sp, 0xDFF0);
    }

    #[test]
    fn test_jr_conditional() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.f = 0x80; // Z=1
        peripherals.write(0xC000, 0x20); // JR NZ, 5 — 不成立
        peripherals.write(0xC001, 0x05);
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.pc, 0xC002);
        assert_eq!(cycles, 8);
        peripherals.write(0xC002, 0x28); // JR Z, 5 — 成立
        peripherals.write(0xC003, 0x05);
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.pc, 0xC009);
        assert_eq!(cycles, 12);
    }

    #[test]
    fn test_cb_bit() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x80;
        peripherals.write(0xC000, 0xCB);
        peripherals.write(0xC001, 0x7F); // BIT 7, A
        cpu.step(&mut peripherals).unwrap();
        assert!(!cpu.registers.get_flag_z());
        peripherals.write(0xC002, 0xCB);
        peripherals.write(0xC003, 0x47); // BIT 0, A
        cpu.step(&mut peripherals).unwrap();
        assert!(cpu.registers.get_flag_z());
    }

    #[test]
    fn test_cb_swap() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0xAB;
        peripherals.write(0xC000, 0xCB);
        peripherals.write(0xC001, 0x37); // SWAP A
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0xBA);
    }

    #[test]
    fn test_ldh() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x42;
        peripherals.write(0xC000, 0xE0); // LDH (0x80), A
        peripherals.write(0xC001, 0x80);
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(peripherals.read(0xFF80), 0x42);
        cpu.registers.a = 0x00;
        peripherals.write(0xC002, 0xF0); // LDH A, (0x80)
        peripherals.write(0xC003, 0x80);
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn test_di_ei() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0xFB); // EI
        cpu.step(&mut peripherals).unwrap();
        assert!(!cpu.ime);
        assert!(cpu.ime_pending);
        peripherals.write(0xC001, 0x00); // NOP（1命令遅延で有効化）
        cpu.step(&mut peripherals).unwrap();
        assert!(cpu.ime);
        peripherals.write(0xC002, 0xF3); // DI
        cpu.step(&mut peripherals).unwrap();
        assert!(!cpu.ime);
    }

    #[test]
    fn test_interrupt_dispatch() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xDFF0;
        cpu.ime = true;
        peripherals.interrupt_flag = 0x01;
        peripherals.interrupt_enable = 0x01;
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 20);
        assert_eq!(cpu.registers.pc, 0x0040);
        assert!(!cpu.ime);
        assert_eq!(peripherals.interrupt_flag & 0x01, 0);
    }

    #[test]
    fn test_halt_and_interrupt_wake() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xDFF0;
        peripherals.write(0xC000, 0x76); // HALT
        cpu.step(&mut peripherals).unwrap();
        assert!(cpu.halted);
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 4);
        assert!(cpu.halted);
        peripherals.interrupt_flag = 0x04;
        peripherals.interrupt_enable = 0x04;
        cpu.step(&mut peripherals).unwrap();
        assert!(!cpu.halted);
    }

    #[test]
    fn test_ld_hl_inc_dec() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x55;
        cpu.registers.set_hl(0xC100);
        peripherals.write(0xC000, 0x22); // LD (HL+), A
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(peripherals.read(0xC100), 0x55);
        assert_eq!(cpu.registers.get_hl(), 0xC101);
        cpu.registers.a = 0xAA;
        peripherals.write(0xC001, 0x32); // LD (HL-), A
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(peripherals.read(0xC101), 0xAA);
        assert_eq!(cpu.registers.get_hl(), 0xC100);
    }

    #[test]
    fn test_cp_instruction() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x42;
        peripherals.write(0xC000, 0xFE); // CP 0x42
        peripherals.write(0xC001, 0x42);
        cpu.step(&mut peripherals).unwrap();
        assert!(cpu.registers.get_flag_z());
        assert_eq!(cpu.registers.a, 0x42);
    }

    #[test]
    fn test_rst() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.sp = 0xDFF0;
        peripherals.write(0xC000, 0xEF); // RST 0x28
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.pc, 0x0028);
    }

    #[test]
    fn test_rotate_rlca() {
        let (mut cpu, mut peripherals) = create_test_system();
        cpu.registers.pc = 0xC000;
        cpu.registers.a = 0x85; // 10000101
        peripherals.write(0xC000, 0x07); // RLCA
        cpu.step(&mut peripherals).unwrap();
        assert_eq!(cpu.registers.a, 0x0B);
        assert!(cpu.registers.get_flag_c());
    }
}
