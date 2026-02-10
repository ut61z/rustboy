// src/cpu/registers.rs
// GameBoy CPU レジスタシステム

/// GameBoy CPU のフラグレジスタビット定義
pub mod flags {
    pub const ZERO: u8 = 0b1000_0000;        // Z: Zero flag
    pub const SUBTRACT: u8 = 0b0100_0000;    // N: Subtract flag
    pub const HALF_CARRY: u8 = 0b0010_0000;  // H: Half carry flag
    pub const CARRY: u8 = 0b0001_0000;       // C: Carry flag
}

/// GameBoy CPU レジスタ
/// Sharp LR35902 (8080/Z80系) のレジスタセット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Registers {
    /// アキュムレータ
    pub a: u8,
    /// フラグレジスタ（下位4bitは常に0）
    pub f: u8,
    /// 汎用レジスタB
    pub b: u8,
    /// 汎用レジスタC
    pub c: u8,
    /// 汎用レジスタD
    pub d: u8,
    /// 汎用レジスタE
    pub e: u8,
    /// 汎用レジスタH
    pub h: u8,
    /// 汎用レジスタL
    pub l: u8,
    /// スタックポインタ
    pub sp: u16,
    /// プログラムカウンタ
    pub pc: u16,
}

impl Registers {
    /// 新しいレジスタセットを作成（初期値）
    pub fn new() -> Self {
        Self {
            a: 0x00,
            f: 0x00,
            b: 0x00,
            c: 0x00,
            d: 0x00,
            e: 0x00,
            h: 0x00,
            l: 0x00,
            sp: 0x0000,
            pc: 0x0000,
        }
    }
    
    /// レジスタを初期状態にリセット
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// 16bitレジスタペアのアクセサ
    
    /// AF レジスタペアを取得
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }
    
    /// AF レジスタペアを設定
    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = (value & 0xF0) as u8; // 下位4bitは常に0
    }
    
    /// BC レジスタペアを取得
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }
    
    /// BC レジスタペアを設定
    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }
    
    /// DE レジスタペアを取得
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }
    
    /// DE レジスタペアを設定
    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }
    
    /// HL レジスタペアを取得
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }
    
    /// HL レジスタペアを設定
    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
    
    // get_xxx エイリアス（CPU命令実装から使用）
    pub fn get_af(&self) -> u16 { self.af() }
    pub fn get_bc(&self) -> u16 { self.bc() }
    pub fn get_de(&self) -> u16 { self.de() }
    pub fn get_hl(&self) -> u16 { self.hl() }

    /// フラグ簡易アクセサ（CPU命令実装から使用）
    pub fn get_flag_z(&self) -> bool { self.zero_flag() }
    pub fn get_flag_c(&self) -> bool { self.carry_flag() }

    /// フラグ操作

    /// Zero flag を取得
    pub fn zero_flag(&self) -> bool {
        (self.f & flags::ZERO) != 0
    }
    
    /// Zero flag を設定
    pub fn set_zero_flag(&mut self, value: bool) {
        if value {
            self.f |= flags::ZERO;
        } else {
            self.f &= !flags::ZERO;
        }
    }
    
    /// Subtract flag を取得
    pub fn subtract_flag(&self) -> bool {
        (self.f & flags::SUBTRACT) != 0
    }
    
    /// Subtract flag を設定
    pub fn set_subtract_flag(&mut self, value: bool) {
        if value {
            self.f |= flags::SUBTRACT;
        } else {
            self.f &= !flags::SUBTRACT;
        }
    }
    
    /// Half carry flag を取得
    pub fn half_carry_flag(&self) -> bool {
        (self.f & flags::HALF_CARRY) != 0
    }
    
    /// Half carry flag を設定
    pub fn set_half_carry_flag(&mut self, value: bool) {
        if value {
            self.f |= flags::HALF_CARRY;
        } else {
            self.f &= !flags::HALF_CARRY;
        }
    }
    
    /// Carry flag を取得
    pub fn carry_flag(&self) -> bool {
        (self.f & flags::CARRY) != 0
    }
    
    /// Carry flag を設定
    pub fn set_carry_flag(&mut self, value: bool) {
        if value {
            self.f |= flags::CARRY;
        } else {
            self.f &= !flags::CARRY;
        }
    }
    
    /// 全フラグを一度に設定（デバッグ用）
    pub fn set_flags(&mut self, zero: bool, subtract: bool, half_carry: bool, carry: bool) {
        self.set_zero_flag(zero);
        self.set_subtract_flag(subtract);
        self.set_half_carry_flag(half_carry);
        self.set_carry_flag(carry);
    }
    
    /// フラグの状態を文字列で取得（デバッグ用）
    pub fn flags_string(&self) -> String {
        format!("{}{}{}{}",
            if self.zero_flag() { "Z" } else { "-" },
            if self.subtract_flag() { "N" } else { "-" },
            if self.half_carry_flag() { "H" } else { "-" },
            if self.carry_flag() { "C" } else { "-" }
        )
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_creation() {
        let regs = Registers::new();
        assert_eq!(regs.a, 0x00);
        assert_eq!(regs.pc, 0x0000);
        assert_eq!(regs.sp, 0x0000);
    }
    
    #[test]
    fn test_16bit_register_pairs() {
        let mut regs = Registers::new();
        
        // AF テスト
        regs.a = 0x12;
        regs.f = 0x30;
        assert_eq!(regs.af(), 0x1230);
        
        regs.set_af(0x5678);
        assert_eq!(regs.a, 0x56);
        assert_eq!(regs.f, 0x70); // 下位4bitはマスクされる
        
        // BC テスト
        regs.set_bc(0x1234);
        assert_eq!(regs.b, 0x12);
        assert_eq!(regs.c, 0x34);
        assert_eq!(regs.bc(), 0x1234);
        
        // DE テスト
        regs.set_de(0x5678);
        assert_eq!(regs.d, 0x56);
        assert_eq!(regs.e, 0x78);
        assert_eq!(regs.de(), 0x5678);
        
        // HL テスト
        regs.set_hl(0x9ABC);
        assert_eq!(regs.h, 0x9A);
        assert_eq!(regs.l, 0xBC);
        assert_eq!(regs.hl(), 0x9ABC);
    }
    
    #[test]
    fn test_flag_operations() {
        let mut regs = Registers::new();
        
        // 個別フラグテスト
        assert!(!regs.zero_flag());
        regs.set_zero_flag(true);
        assert!(regs.zero_flag());
        assert_eq!(regs.f, flags::ZERO);
        
        regs.set_carry_flag(true);
        assert!(regs.carry_flag());
        assert_eq!(regs.f, flags::ZERO | flags::CARRY);
        
        // 全フラグ設定テスト
        regs.set_flags(true, true, false, true);
        assert!(regs.zero_flag());
        assert!(regs.subtract_flag());
        assert!(!regs.half_carry_flag());
        assert!(regs.carry_flag());
    }
    
    #[test]
    fn test_f_register_masking() {
        let mut regs = Registers::new();
        
        // Fレジスタの下位4bitは常に0になることを確認
        regs.f = 0xFF;
        assert_eq!(regs.f, 0xFF);
        
        regs.set_af(0x12FF);
        assert_eq!(regs.f, 0xF0); // 下位4bitがマスクされる
    }
    
    #[test]
    fn test_flags_string() {
        let mut regs = Registers::new();
        
        assert_eq!(regs.flags_string(), "----");
        
        regs.set_flags(true, false, true, false);
        assert_eq!(regs.flags_string(), "Z-H-");
        
        regs.set_flags(true, true, true, true);
        assert_eq!(regs.flags_string(), "ZNHC");
    }
}
