// src/cpu/mod.rs
// GameBoy CPU (Sharp LR35902) の最小限実装

pub mod registers;
pub mod instructions;
pub mod decoder;

pub use registers::Registers;
use crate::peripherals::Peripherals;

/// GameBoy CPU の状態
pub struct Cpu {
    /// CPUレジスタ
    pub registers: Registers,
    /// 割り込み無効フラグ
    pub ime: bool,  // Interrupt Master Enable
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
            halted: false,
            instruction_count: 0,
        }
    }
    
    /// CPUを初期状態にリセット
    pub fn reset(&mut self) {
        self.registers.reset();
        self.ime = false;
        self.halted = false;
        self.instruction_count = 0;
    }
    
    /// 1命令を実行
    pub fn step(&mut self, peripherals: &mut Peripherals) -> Result<u8, String> {
        if self.halted {
            // TODO: 割り込み処理の実装後に適切に処理
            return Ok(4); // HALTは4クロック
        }
        
        // フェッチ
        let opcode = self.fetch_byte(peripherals);
        
        // デコード・実行
        let cycles = self.execute_instruction(opcode, peripherals)?;
        
        self.instruction_count += 1;
        
        Ok(cycles)
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
    
    /// 命令を実行
    fn execute_instruction(&mut self, opcode: u8, peripherals: &mut Peripherals) -> Result<u8, String> {
        match opcode {
            // NOP
            0x00 => Ok(4),
            
            // LD r8, n - 8bitレジスタに即値をロード
            0x3E => { // LD A, n
                let value = self.fetch_byte(peripherals);
                self.registers.a = value;
                Ok(8)
            }
            0x06 => { // LD B, n
                let value = self.fetch_byte(peripherals);
                self.registers.b = value;
                Ok(8)
            }
            0x0E => { // LD C, n
                let value = self.fetch_byte(peripherals);
                self.registers.c = value;
                Ok(8)
            }
            0x16 => { // LD D, n
                let value = self.fetch_byte(peripherals);
                self.registers.d = value;
                Ok(8)
            }
            0x1E => { // LD E, n
                let value = self.fetch_byte(peripherals);
                self.registers.e = value;
                Ok(8)
            }
            0x26 => { // LD H, n
                let value = self.fetch_byte(peripherals);
                self.registers.h = value;
                Ok(8)
            }
            0x2E => { // LD L, n
                let value = self.fetch_byte(peripherals);
                self.registers.l = value;
                Ok(8)
            }
            
            // LD SP, nn - スタックポインタに16bit値をロード
            0x31 => {
                let value = self.fetch_word(peripherals);
                self.registers.sp = value;
                Ok(12)
            }
            
            // JP nn - 絶対ジャンプ
            0xC3 => {
                let addr = self.fetch_word(peripherals);
                self.registers.pc = addr;
                Ok(16)
            }
            
            // JR n - 相対ジャンプ
            0x18 => {
                let offset = self.fetch_byte(peripherals) as i8;
                self.registers.pc = self.registers.pc.wrapping_add(offset as u16);
                Ok(12)
            }
            
            _ => Err(format!("未実装の命令: 0x{:02X} at PC=0x{:04X}", opcode, self.registers.pc.wrapping_sub(1)))
        }
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
        
        // WRAM領域にテストプログラムを配置
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0x00); // NOP
        
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 4);
        assert_eq!(cpu.registers.pc, 0xC001);
        assert_eq!(cpu.instruction_count, 1);
    }
    
    #[test]
    fn test_ld_a_n_instruction() {
        let (mut cpu, mut peripherals) = create_test_system();
        
        // WRAM領域にテストプログラムを配置
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0x3E); // LD A, n
        peripherals.write(0xC001, 0x42); // n = 0x42
        
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 8);
        assert_eq!(cpu.registers.a, 0x42);
        assert_eq!(cpu.registers.pc, 0xC002);
    }
    
    #[test]
    fn test_jp_nn_instruction() {
        let (mut cpu, mut peripherals) = create_test_system();
        
        // WRAM領域にテストプログラムを配置
        cpu.registers.pc = 0xC000;
        peripherals.write(0xC000, 0xC3); // JP nn
        peripherals.write(0xC001, 0x34); // nn low byte
        peripherals.write(0xC002, 0x12); // nn high byte
        
        let cycles = cpu.step(&mut peripherals).unwrap();
        assert_eq!(cycles, 16);
        assert_eq!(cpu.registers.pc, 0x1234);
    }
}
