// src/memory/hram.rs
// High RAM: CPUが高速にアクセスできる127バイトの小さなメモリ
// スタック操作や重要な変数の保存に使用される

use crate::memory_map::dmg::{HRAM_SIZE, HRAM_START, HRAM_END};

pub struct HighRam {
    data: Box<[u8; HRAM_SIZE]>,
}

impl HighRam {
    /// 新しいHigh RAMを作成（全て0で初期化）
    pub fn new() -> Self {
        Self {
            data: Box::new([0; HRAM_SIZE]),
        }
    }
    
    /// 指定されたアドレスからデータを読み取る
    pub fn read(&self, addr: u16) -> u8 {
        let index = self.addr_to_index(addr);
        self.data[index]
    }
    
    /// 指定されたアドレスにデータを書き込む
    pub fn write(&mut self, addr: u16, value: u8) {
        let index = self.addr_to_index(addr);
        self.data[index] = value;
    }
    
    /// アドレスを配列のインデックスに変換
    /// アドレス0xFF80-0xFFFEを配列インデックス0-0x7Eにマップ
    fn addr_to_index(&self, addr: u16) -> usize {
        // アドレス範囲チェック（デバッグビルドでのみ）
        debug_assert!(
            addr >= HRAM_START && addr <= HRAM_END,
            "HRAMアドレス範囲外: 0x{:04X} (有効範囲: 0x{:04X}-0x{:04X})",
            addr, HRAM_START, HRAM_END
        );
        
        // 0xFF80を引いて相対アドレスに変換
        (addr - HRAM_START) as usize
    }
    
    /// HRAMの内容をクリア
    pub fn clear(&mut self) {
        self.data.fill(0);
    }
    
    /// スタック操作のヘルパー関数
    /// GameBoyのスタックは通常HRAMの上位部分を使用する
    pub fn push_stack(&mut self, sp: &mut u16, value: u8) -> Result<(), String> {
        if *sp < HRAM_START {
            return Err(format!("スタックポインタが範囲外: 0x{:04X}", sp));
        }
        
        *sp = sp.wrapping_sub(1);
        if *sp >= HRAM_START && *sp <= HRAM_END {
            self.write(*sp, value);
            Ok(())
        } else {
            Err(format!("スタックオーバーフロー: SP=0x{:04X}", sp))
        }
    }
    
    pub fn pop_stack(&mut self, sp: &mut u16) -> Result<u8, String> {
        if *sp > HRAM_END {
            return Err(format!("スタックポインタが範囲外: 0x{:04X}", sp));
        }
        
        let value = self.read(*sp);
        *sp = sp.wrapping_add(1);
        Ok(value)
    }
    
    /// デバッグ用: HRAM全体をダンプ
    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str("=== HRAM Dump ===\n");
        result.push_str("Address : 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n");
        
        for row in 0..8 {  // 127バイト = 8行弱
            let base_addr = HRAM_START + (row * 16);
            result.push_str(&format!("0x{:04X}: ", base_addr));
            
            for col in 0..16 {
                let addr = base_addr + col;
                if addr <= HRAM_END {
                    let value = self.read(addr);
                    result.push_str(&format!("{:02X} ", value));
                } else {
                    result.push_str("   ");
                }
            }
            
            result.push_str(" | ");
            
            // ASCII表示
            for col in 0..16 {
                let addr = base_addr + col;
                if addr <= HRAM_END {
                    let value = self.read(addr);
                    if value >= 32 && value <= 126 {
                        result.push(value as char);
                    } else {
                        result.push('.');
                    }
                } else {
                    result.push(' ');
                }
            }
            
            result.push('\n');
        }
        
        result
    }
    
    /// 使用状況の統計
    pub fn get_usage_stats(&self) -> (usize, usize, f32) {
        let non_zero_count = self.data.iter().filter(|&&b| b != 0).count();
        let total_size = HRAM_SIZE;
        let usage_percent = (non_zero_count as f32 / total_size as f32) * 100.0;
        
        (non_zero_count, total_size, usage_percent)
    }
    
    /// よく使われるHRAMの場所にニックネームを付ける
    pub fn get_location_name(addr: u16) -> &'static str {
        match addr {
            0xFF80..=0xFF8F => "スタック予備領域",
            0xFF90..=0xFF9F => "一時変数領域", 
            0xFFA0..=0xFFAF => "ゲーム専用領域",
            0xFFB0..=0xFFCF => "システム変数",
            0xFFD0..=0xFFFE => "スタック領域",
            _ => "不明",
        }
    }
}

impl Default for HighRam {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hram_read_write() {
        let mut hram = HighRam::new();
        
        // 書き込みテスト
        hram.write(0xFF80, 0x42);
        hram.write(0xFF81, 0x24);
        hram.write(0xFFFE, 0xFF);
        
        // 読み取りテスト
        assert_eq!(hram.read(0xFF80), 0x42);
        assert_eq!(hram.read(0xFF81), 0x24);
        assert_eq!(hram.read(0xFFFE), 0xFF);
    }
    
    #[test]
    fn test_hram_stack_operations() {
        let mut hram = HighRam::new();
        let mut sp = 0xFFFE;  // 初期スタックポインタ
        
        // プッシュテスト
        hram.push_stack(&mut sp, 0x42).unwrap();
        assert_eq!(sp, 0xFFFD);
        
        hram.push_stack(&mut sp, 0x24).unwrap();
        assert_eq!(sp, 0xFFFC);
        
        // ポップテスト（LIFO）
        assert_eq!(hram.pop_stack(&mut sp).unwrap(), 0x24);
        assert_eq!(sp, 0xFFFD);
        
        assert_eq!(hram.pop_stack(&mut sp).unwrap(), 0x42);
        assert_eq!(sp, 0xFFFE);
    }
    
    #[test]
    fn test_hram_clear() {
        let mut hram = HighRam::new();
        
        hram.write(0xFF80, 0x42);
        hram.write(0xFFFE, 0x24);
        
        hram.clear();
        
        assert_eq!(hram.read(0xFF80), 0x00);
        assert_eq!(hram.read(0xFFFE), 0x00);
    }
    
    #[test]
    fn test_hram_addr_to_index() {
        let hram = HighRam::new();
        
        assert_eq!(hram.addr_to_index(0xFF80), 0);
        assert_eq!(hram.addr_to_index(0xFF81), 1);
        assert_eq!(hram.addr_to_index(0xFFFE), 0x7E);
    }
}
