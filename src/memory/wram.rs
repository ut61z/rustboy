// src/memory/wram.rs
// Work RAM: ゲームが作業用に使用する8KBのメモリ

use crate::memory_map::dmg::{WRAM_SIZE, WRAM_START, WRAM_END};

pub struct WorkRam {
    data: Box<[u8; WRAM_SIZE]>,
}

impl WorkRam {
    /// 新しいWork RAMを作成（全て0で初期化）
    pub fn new() -> Self {
        Self {
            data: Box::new([0; WRAM_SIZE]),
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
    /// アドレス0xC000-0xDFFFを配列インデックス0-0x1FFFにマップ
    fn addr_to_index(&self, addr: u16) -> usize {
        // アドレス範囲チェック（デバッグビルドでのみ）
        debug_assert!(
            addr >= WRAM_START && addr <= WRAM_END,
            "WRAMアドレス範囲外: 0x{:04X} (有効範囲: 0x{:04X}-0x{:04X})",
            addr, WRAM_START, WRAM_END
        );
        
        // 0xC000を引いて相対アドレスに変換し、サイズでマスク
        ((addr - WRAM_START) as usize) & (WRAM_SIZE - 1)
    }
    
    /// メモリの特定の範囲をクリア
    pub fn clear_range(&mut self, start_addr: u16, end_addr: u16) {
        for addr in start_addr..=end_addr {
            if addr >= WRAM_START && addr <= WRAM_END {
                self.write(addr, 0);
            }
        }
    }
    
    /// メモリ全体をクリア
    pub fn clear_all(&mut self) {
        self.data.fill(0);
    }
    
    /// デバッグ用: 指定範囲のメモリ内容をダンプ
    pub fn dump_range(&self, start_addr: u16, end_addr: u16) -> String {
        let mut result = String::new();
        result.push_str(&format!("=== WRAM Dump 0x{:04X}-0x{:04X} ===\n", start_addr, end_addr));
        
        let mut addr = start_addr & 0xFFF0;  // 16バイト境界に調整
        
        while addr <= end_addr {
            result.push_str(&format!("0x{:04X}: ", addr));
            
            for i in 0..16 {
                let current_addr = addr + i;
                if current_addr >= WRAM_START && current_addr <= WRAM_END && current_addr <= end_addr {
                    let value = self.read(current_addr);
                    if current_addr >= start_addr {
                        result.push_str(&format!("{:02X} ", value));
                    } else {
                        result.push_str("   ");
                    }
                } else {
                    result.push_str("   ");
                }
            }
            
            result.push_str(" | ");
            
            // ASCII表示
            for i in 0..16 {
                let current_addr = addr + i;
                if current_addr >= WRAM_START && current_addr <= WRAM_END && 
                   current_addr <= end_addr && current_addr >= start_addr {
                    let value = self.read(current_addr);
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
            addr += 16;
        }
        
        result
    }
    
    /// メモリ使用量の統計
    pub fn get_usage_stats(&self) -> (usize, usize, f32) {
        let non_zero_count = self.data.iter().filter(|&&b| b != 0).count();
        let total_size = WRAM_SIZE;
        let usage_percent = (non_zero_count as f32 / total_size as f32) * 100.0;
        
        (non_zero_count, total_size, usage_percent)
    }
}

impl Default for WorkRam {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wram_read_write() {
        let mut wram = WorkRam::new();
        
        // 書き込みテスト
        wram.write(0xC000, 0x42);
        wram.write(0xC001, 0x24);
        wram.write(0xDFFF, 0xFF);
        
        // 読み取りテスト
        assert_eq!(wram.read(0xC000), 0x42);
        assert_eq!(wram.read(0xC001), 0x24);
        assert_eq!(wram.read(0xDFFF), 0xFF);
    }
    
    #[test]
    fn test_wram_clear() {
        let mut wram = WorkRam::new();
        
        // データを書き込み
        wram.write(0xC000, 0x42);
        wram.write(0xC100, 0x24);
        
        // 範囲クリア
        wram.clear_range(0xC000, 0xC0FF);
        
        assert_eq!(wram.read(0xC000), 0x00);
        assert_eq!(wram.read(0xC100), 0x24);  // 範囲外なので残る
        
        // 全体クリア
        wram.clear_all();
        assert_eq!(wram.read(0xC100), 0x00);
    }
    
    #[test]
    fn test_wram_addr_to_index() {
        let wram = WorkRam::new();
        
        // 境界値のテスト
        assert_eq!(wram.addr_to_index(0xC000), 0);
        assert_eq!(wram.addr_to_index(0xC001), 1);
        assert_eq!(wram.addr_to_index(0xDFFF), 0x1FFF);
    }
}
