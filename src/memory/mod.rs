// src/memory/mod.rs
// メモリモジュール全体を統合（メモリマップ対応版）

pub mod bootrom;
pub mod wram;
pub mod hram;

// 各メモリモジュールを外部から使いやすくする
pub use bootrom::BootRom;
pub use wram::WorkRam;
pub use hram::HighRam;

#[cfg(test)]
mod tests {
    use crate::memory_map::dmg::*;
    use crate::memory_map::{get_memory_region, MemoryRegion};
    
    #[test]
    fn test_memory_constants() {
        // 定数が正しく定義されているかテスト
        assert_eq!(BOOTROM_SIZE, 0x100);
        assert_eq!(WRAM_SIZE, 0x2000);
        assert_eq!(HRAM_SIZE, 0x7F);
        
        // アドレス範囲が正しいかテスト
        assert_eq!(BOOTROM_END - BOOTROM_START + 1, BOOTROM_SIZE as u16);
        assert_eq!(WRAM_END - WRAM_START + 1, WRAM_SIZE as u16);
        assert_eq!(HRAM_END - HRAM_START + 1, HRAM_SIZE as u16);
    }
    
    #[test]
    fn test_memory_region_detection() {
        assert_eq!(get_memory_region(0x0000), MemoryRegion::BootRom);
        assert_eq!(get_memory_region(0x0100), MemoryRegion::CartridgeRom);
        assert_eq!(get_memory_region(0xC000), MemoryRegion::WorkRam);
        assert_eq!(get_memory_region(0xFF80), MemoryRegion::HighRam);
    }
}
