use crate::memory_map::{
    dmg::*,
    io_registers::*,
    get_region_name,
};
use crate::memory::{
    BootRom, WorkRam, HighRam,
};

pub struct Peripherals {
    bootrom: BootRom,
    wram: WorkRam,
    hram: HighRam,
    
    // 統計情報
    read_count: u64,
    write_count: u64,
}

impl Peripherals {
    /// 新しいPeripheralsを作成
    pub fn new(bootrom: BootRom) -> Self {
        Self {
            bootrom,
            wram: WorkRam::new(),
            hram: HighRam::new(),
            read_count: 0,
            write_count: 0,
        }
    }
    
    /// ダミーBootROMでPeripheralsを作成（テスト用）
    pub fn new_with_dummy_bootrom() -> Self {
        Self::new(BootRom::new_dummy())
    }
    
    /// 指定されたアドレスからデータを読み取る
    pub fn read(&mut self, addr: u16) -> u8 {
        self.read_count += 1;
        
        let value = match addr {
            // BootROM領域
            BOOTROM_START..=BOOTROM_END => {
                if self.bootrom.is_active() {
                    self.bootrom.read(addr)
                } else {
                    // BootROM無効時は通常はCartridge ROMを読むが、今は未実装なので0xFF
                    0xFF
                }
            }
            
            // Work RAM領域
            WRAM_START..=WRAM_END => {
                self.wram.read(addr)
            }
            
            // Work RAM Echo領域（0xE000-0xFDFF）
            // WRAMのミラー、実際のゲームでは使用禁止
            0xE000..=0xFDFF => {
                let wram_addr = 0xC000 + (addr - 0xE000);
                self.wram.read(wram_addr)
            }
            
            // High RAM領域
            HRAM_START..=HRAM_END => {
                self.hram.read(addr)
            }
            
            // BootROM無効化レジスタ（読み取り専用、常に0xFF）
            BOOTROM_DISABLE => 0xFF,
            
            // 割り込み許可レジスタ（未実装）
            0xFFFF => 0x00,
            
            // その他の領域（未実装）
            _ => {
                #[cfg(debug_assertions)]
                println!("未実装領域から読み取り: 0x{:04X} ({})", addr, get_region_name(addr));
                0xFF  // 未実装領域は0xFFを返す
            }
        };
        
        #[cfg(feature = "trace_memory")]
        println!("READ  0x{:04X} = 0x{:02X} [{}]", addr, value, addr_to_region_name(addr));
        
        value
    }
    
    /// 指定されたアドレスにデータを書き込む
    pub fn write(&mut self, addr: u16, value: u8) {
        self.write_count += 1;
        
        #[cfg(feature = "trace_memory")]
        println!("WRITE 0x{:04X} = 0x{:02X} [{}]", addr, value, addr_to_region_name(addr));
        
        match addr {
            // BootROM領域（読み取り専用）
            BOOTROM_START..=BOOTROM_END => {
                #[cfg(debug_assertions)]
                println!("警告: BootROM領域への書き込み試行: 0x{:04X} = 0x{:02X}", addr, value);
            }
            
            // Work RAM領域
            WRAM_START..=WRAM_END => {
                self.wram.write(addr, value);
            }
            
            // Work RAM Echo領域
            0xE000..=0xFDFF => {
                let wram_addr = 0xC000 + (addr - 0xE000);
                self.wram.write(wram_addr, value);
            }
            
            // High RAM領域
            HRAM_START..=HRAM_END => {
                self.hram.write(addr, value);
            }
            
            // BootROM無効化レジスタ
            BOOTROM_DISABLE => {
                self.bootrom.write_disable_register(value);
            }
            
            // 割り込み許可レジスタ（未実装）
            0xFFFF => {
                #[cfg(debug_assertions)]
                println!("割り込み許可レジスタへの書き込み: 0x{:02X} (未実装)", value);
            }
            
            // その他の領域
            _ => {
                #[cfg(debug_assertions)]
                println!("未実装領域への書き込み: 0x{:04X} = 0x{:02X} ({})", 
                        addr, value, get_region_name(addr));
            }
        }
    }
    
    /// 16bitデータを読み取る（リトルエンディアン）
    pub fn read16(&mut self, addr: u16) -> u16 {
        let low = self.read(addr) as u16;
        let high = self.read(addr.wrapping_add(1)) as u16;
        (high << 8) | low
    }
    
    /// 16bitデータを書き込む（リトルエンディアン）
    pub fn write16(&mut self, addr: u16, value: u16) {
        self.write(addr, value as u8);           // 下位バイト
        self.write(addr.wrapping_add(1), (value >> 8) as u8);  // 上位バイト
    }
    
    /// メモリの統計情報を取得
    pub fn get_stats(&self) -> MemoryStats {
        let (wram_used, wram_total, wram_percent) = self.wram.get_usage_stats();
        let (hram_used, hram_total, hram_percent) = self.hram.get_usage_stats();
        
        MemoryStats {
            read_count: self.read_count,
            write_count: self.write_count,
            bootrom_active: self.bootrom.is_active(),
            wram_used_bytes: wram_used,
            wram_total_bytes: wram_total,
            wram_usage_percent: wram_percent,
            hram_used_bytes: hram_used,
            hram_total_bytes: hram_total,
            hram_usage_percent: hram_percent,
        }
    }
    
    /// メモリの特定範囲をダンプ
    pub fn dump_memory(&mut self, start_addr: u16, end_addr: u16) -> String {
        let mut result = String::new();
        result.push_str(&format!("=== Memory Dump 0x{:04X}-0x{:04X} ===\n", start_addr, end_addr));
        
        let mut addr = start_addr & 0xFFF0;  // 16バイト境界に調整
        
        while addr <= end_addr {
            result.push_str(&format!("0x{:04X}: ", addr));
            
            // 16進数表示
            for i in 0..16 {
                let current_addr = addr + i;
                if current_addr <= end_addr {
                    let value = self.read(current_addr);
                    result.push_str(&format!("{:02X} ", value));
                } else {
                    result.push_str("   ");
                }
            }
            
            result.push_str(" | ");
            
            // ASCII表示
            for i in 0..16 {
                let current_addr = addr + i;
                if current_addr <= end_addr {
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
            
            result.push_str(&format!(" [{}]\n", get_region_name(addr)));
            addr += 16;
        }
        
        result
    }
    
    /// 統計情報をリセット
    pub fn reset_stats(&mut self) {
        self.read_count = 0;
        self.write_count = 0;
    }
    
    /// システム全体をリセット
    pub fn reset(&mut self) {
        self.wram.clear_all();
        self.hram.clear();
        self.reset_stats();
        // BootROMは再有効化しない（実際のハードウェアでは不可能）
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub read_count: u64,
    pub write_count: u64,
    pub bootrom_active: bool,
    pub wram_used_bytes: usize,
    pub wram_total_bytes: usize,
    pub wram_usage_percent: f32,
    pub hram_used_bytes: usize,
    pub hram_total_bytes: usize,
    pub hram_usage_percent: f32,
}

impl std::fmt::Display for MemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
            "Memory Statistics:\n\
             - Read operations: {}\n\
             - Write operations: {}\n\
             - BootROM active: {}\n\
             - WRAM usage: {}/{} bytes ({:.1}%)\n\
             - HRAM usage: {}/{} bytes ({:.1}%)",
            self.read_count,
            self.write_count,
            self.bootrom_active,
            self.wram_used_bytes, self.wram_total_bytes, self.wram_usage_percent,
            self.hram_used_bytes, self.hram_total_bytes, self.hram_usage_percent
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_peripherals_bootrom() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();
        
        // BootROM有効時の読み取り
        assert!(peripherals.bootrom.is_active());
        let value = peripherals.read(0x0000);
        assert_eq!(value, 0x00);  // ダミーBootROMは全て0x00
        
        // BootROM無効化
        peripherals.write(0xFF50, 0x01);
        assert!(!peripherals.bootrom.is_active());
        
        // 無効化後の読み取り
        let value = peripherals.read(0x0000);
        assert_eq!(value, 0xFF);  // カートリッジROM未実装なので0xFF
    }
    
    #[test]
    fn test_peripherals_wram() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();
        
        // WRAM書き込み・読み取り
        peripherals.write(0xC000, 0x42);
        peripherals.write(0xDFFF, 0x24);
        
        assert_eq!(peripherals.read(0xC000), 0x42);
        assert_eq!(peripherals.read(0xDFFF), 0x24);
        
        // WRAMエコー領域のテスト
        peripherals.write(0xE000, 0x99);  // エコー領域に書き込み
        assert_eq!(peripherals.read(0xC000), 0x99);  // WRAMから読み取り
    }
    
    #[test]
    fn test_peripherals_hram() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();
        
        peripherals.write(0xFF80, 0xAB);
        peripherals.write(0xFFFE, 0xCD);
        
        assert_eq!(peripherals.read(0xFF80), 0xAB);
        assert_eq!(peripherals.read(0xFFFE), 0xCD);
    }
    
    #[test]
    fn test_peripherals_16bit_access() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();
        
        // 16bit書き込み
        peripherals.write16(0xC000, 0x1234);
        
        // 個別に読み取り（リトルエンディアン）
        assert_eq!(peripherals.read(0xC000), 0x34);  // 下位バイト
        assert_eq!(peripherals.read(0xC001), 0x12);  // 上位バイト
        
        // 16bit読み取り
        assert_eq!(peripherals.read16(0xC000), 0x1234);
    }
}
