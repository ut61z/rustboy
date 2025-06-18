// ===== 現在の問題点 =====
// 定数が複数ファイルに散らばっている：
// - src/memory/mod.rs に一部の定数
// - src/memory/bootrom.rs で個別に使用
// - src/memory/wram.rs で個別に使用
// - src/peripherals.rs でも重複

// ===== 改善案: 専用ファイルでメモリマップを一元管理 =====

// src/memory_map.rs - 新しく作成
//! GameBoy メモリマップ定義
//! 
//! このファイルはGameBoyのメモリマップを一元管理します。
//! 全てのアドレス範囲、サイズ、特別なレジスタアドレスを定義。

/// GameBoy DMG (オリジナル) のメモリマップ
pub mod dmg {
    // ===== BootROM =====
    pub const BOOTROM_START: u16 = 0x0000;
    pub const BOOTROM_END: u16 = 0x00FF;
    pub const BOOTROM_SIZE: usize = 0x100;  // 256バイト
    
    // ===== カートリッジROM =====
    pub const CARTRIDGE_ROM_START: u16 = 0x0100;  // BootROM無効化後
    pub const CARTRIDGE_ROM_END: u16 = 0x7FFF;
    pub const CARTRIDGE_ROM_BANK0_START: u16 = 0x0000;  // Bank 0 (BootROM有効時は隠れる)
    pub const CARTRIDGE_ROM_BANK0_END: u16 = 0x3FFF;
    pub const CARTRIDGE_ROM_BANKN_START: u16 = 0x4000;  // Bank 1+
    pub const CARTRIDGE_ROM_BANKN_END: u16 = 0x7FFF;
    
    // ===== Video RAM =====
    pub const VRAM_START: u16 = 0x8000;
    pub const VRAM_END: u16 = 0x9FFF;
    pub const VRAM_SIZE: usize = 0x2000;    // 8KB
    
    // VRAM内部構造
    pub const TILE_DATA_START: u16 = 0x8000;
    pub const TILE_DATA_END: u16 = 0x97FF;
    pub const TILE_MAP_0_START: u16 = 0x9800;
    pub const TILE_MAP_0_END: u16 = 0x9BFF;
    pub const TILE_MAP_1_START: u16 = 0x9C00;
    pub const TILE_MAP_1_END: u16 = 0x9FFF;
    
    // ===== カートリッジRAM =====
    pub const CARTRIDGE_RAM_START: u16 = 0xA000;
    pub const CARTRIDGE_RAM_END: u16 = 0xBFFF;
    pub const CARTRIDGE_RAM_SIZE: usize = 0x2000;  // 8KB (MBCによって異なる)
    
    // ===== Work RAM =====
    pub const WRAM_START: u16 = 0xC000;
    pub const WRAM_END: u16 = 0xDFFF;
    pub const WRAM_SIZE: usize = 0x2000;    // 8KB
    
    // ===== Work RAM Echo (使用禁止) =====
    pub const WRAM_ECHO_START: u16 = 0xE000;
    pub const WRAM_ECHO_END: u16 = 0xFDFF;
    
    // ===== Object Attribute Memory =====
    pub const OAM_START: u16 = 0xFE00;
    pub const OAM_END: u16 = 0xFE9F;
    pub const OAM_SIZE: usize = 0xA0;       // 160バイト
    
    // ===== 未使用領域 =====
    pub const UNUSED_START: u16 = 0xFEA0;
    pub const UNUSED_END: u16 = 0xFEFF;
    
    // ===== I/O レジスタ =====
    pub const IO_REGISTERS_START: u16 = 0xFF00;
    pub const IO_REGISTERS_END: u16 = 0xFF7F;
    
    // ===== High RAM =====
    pub const HRAM_START: u16 = 0xFF80;
    pub const HRAM_END: u16 = 0xFFFE;
    pub const HRAM_SIZE: usize = 0x7F;      // 127バイト
    
    // ===== 割り込み許可レジスタ =====
    pub const IE_REGISTER: u16 = 0xFFFF;
}

/// 重要なI/Oレジスタのアドレス
pub mod io_registers {
    // ===== ジョイパッド =====
    pub const JOYP: u16 = 0xFF00;  // ジョイパッド
    
    // ===== シリアル通信 =====
    pub const SB: u16 = 0xFF01;    // シリアルデータ
    pub const SC: u16 = 0xFF02;    // シリアル制御
    
    // ===== タイマー =====
    pub const DIV: u16 = 0xFF04;   // 分周器
    pub const TIMA: u16 = 0xFF05;  // タイマーカウンタ
    pub const TMA: u16 = 0xFF06;   // タイマーモジュロ
    pub const TAC: u16 = 0xFF07;   // タイマー制御
    
    // ===== 割り込み =====
    pub const IF: u16 = 0xFF0F;    // 割り込みフラグ
    
    // ===== 音声 (APU) =====
    pub const NR10: u16 = 0xFF10;  // チャンネル1スイープ
    pub const NR11: u16 = 0xFF11;  // チャンネル1長さ/波形
    pub const NR12: u16 = 0xFF12;  // チャンネル1エンベロープ
    pub const NR13: u16 = 0xFF13;  // チャンネル1周波数下位
    pub const NR14: u16 = 0xFF14;  // チャンネル1周波数上位/制御
    
    pub const NR21: u16 = 0xFF16;  // チャンネル2長さ/波形
    pub const NR22: u16 = 0xFF17;  // チャンネル2エンベロープ
    pub const NR23: u16 = 0xFF18;  // チャンネル2周波数下位
    pub const NR24: u16 = 0xFF19;  // チャンネル2周波数上位/制御
    
    pub const NR30: u16 = 0xFF1A;  // チャンネル3オン/オフ
    pub const NR31: u16 = 0xFF1B;  // チャンネル3長さ
    pub const NR32: u16 = 0xFF1C;  // チャンネル3出力レベル
    pub const NR33: u16 = 0xFF1D;  // チャンネル3周波数下位
    pub const NR34: u16 = 0xFF1E;  // チャンネル3周波数上位/制御
    
    pub const NR41: u16 = 0xFF20;  // チャンネル4長さ
    pub const NR42: u16 = 0xFF21;  // チャンネル4エンベロープ
    pub const NR43: u16 = 0xFF22;  // チャンネル4多項式
    pub const NR44: u16 = 0xFF23;  // チャンネル4制御
    
    pub const NR50: u16 = 0xFF24;  // マスター音量/VIN
    pub const NR51: u16 = 0xFF25;  // 音声出力選択
    pub const NR52: u16 = 0xFF26;  // 音声オン/オフ
    
    // 波形RAM
    pub const WAVE_RAM_START: u16 = 0xFF30;
    pub const WAVE_RAM_END: u16 = 0xFF3F;
    
    // ===== LCD制御 =====
    pub const LCDC: u16 = 0xFF40;  // LCD制御
    pub const STAT: u16 = 0xFF41;  // LCDステータス
    pub const SCY: u16 = 0xFF42;   // スクロールY
    pub const SCX: u16 = 0xFF43;   // スクロールX
    pub const LY: u16 = 0xFF44;    // LCD Y座標
    pub const LYC: u16 = 0xFF45;   // LY比較
    pub const DMA: u16 = 0xFF46;   // OAM DMA転送
    pub const BGP: u16 = 0xFF47;   // BG パレット
    pub const OBP0: u16 = 0xFF48;  // オブジェクトパレット0
    pub const OBP1: u16 = 0xFF49;  // オブジェクトパレット1
    pub const WY: u16 = 0xFF4A;    // ウィンドウY
    pub const WX: u16 = 0xFF4B;    // ウィンドウX
    
    // ===== その他 =====
    pub const BOOTROM_DISABLE: u16 = 0xFF50;  // BootROM無効化
}

/// メモリ領域の種類を識別する列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryRegion {
    BootRom,
    CartridgeRom,
    VideoRam,
    CartridgeRam,
    WorkRam,
    WorkRamEcho,
    Oam,
    Unused,
    IoRegisters,
    HighRam,
    InterruptEnable,
}

/// アドレスからメモリ領域を判定する関数
pub fn get_memory_region(addr: u16) -> MemoryRegion {
    use dmg::*;
    
    match addr {
        BOOTROM_START..=BOOTROM_END => MemoryRegion::BootRom,
        CARTRIDGE_ROM_START..=CARTRIDGE_ROM_END => MemoryRegion::CartridgeRom,
        VRAM_START..=VRAM_END => MemoryRegion::VideoRam,
        CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => MemoryRegion::CartridgeRam,
        WRAM_START..=WRAM_END => MemoryRegion::WorkRam,
        WRAM_ECHO_START..=WRAM_ECHO_END => MemoryRegion::WorkRamEcho,
        OAM_START..=OAM_END => MemoryRegion::Oam,
        UNUSED_START..=UNUSED_END => MemoryRegion::Unused,
        IO_REGISTERS_START..=IO_REGISTERS_END => MemoryRegion::IoRegisters,
        HRAM_START..=HRAM_END => MemoryRegion::HighRam,
        IE_REGISTER => MemoryRegion::InterruptEnable,
    }
}

/// メモリ領域名を取得
pub fn get_region_name(addr: u16) -> &'static str {
    match get_memory_region(addr) {
        MemoryRegion::BootRom => "BootROM",
        MemoryRegion::CartridgeRom => "Cartridge ROM",
        MemoryRegion::VideoRam => "Video RAM",
        MemoryRegion::CartridgeRam => "Cartridge RAM",
        MemoryRegion::WorkRam => "Work RAM",
        MemoryRegion::WorkRamEcho => "Work RAM Echo",
        MemoryRegion::Oam => "OAM",
        MemoryRegion::Unused => "Unused",
        MemoryRegion::IoRegisters => "I/O Registers",
        MemoryRegion::HighRam => "High RAM",
        MemoryRegion::InterruptEnable => "Interrupt Enable",
    }
}

/// 特定のI/Oレジスタ名を取得
pub fn get_io_register_name(addr: u16) -> Option<&'static str> {
    use io_registers::*;
    
    match addr {
        JOYP => Some("JOYP"),
        SB => Some("SB"),
        SC => Some("SC"),
        DIV => Some("DIV"),
        TIMA => Some("TIMA"),
        TMA => Some("TMA"),
        TAC => Some("TAC"),
        IF => Some("IF"),
        NR10 => Some("NR10"),
        NR11 => Some("NR11"),
        NR12 => Some("NR12"),
        NR13 => Some("NR13"),
        NR14 => Some("NR14"),
        NR21 => Some("NR21"),
        NR22 => Some("NR22"),
        NR23 => Some("NR23"),
        NR24 => Some("NR24"),
        NR30 => Some("NR30"),
        NR31 => Some("NR31"),
        NR32 => Some("NR32"),
        NR33 => Some("NR33"),
        NR34 => Some("NR34"),
        NR41 => Some("NR41"),
        NR42 => Some("NR42"),
        NR43 => Some("NR43"),
        NR44 => Some("NR44"),
        NR50 => Some("NR50"),
        NR51 => Some("NR51"),
        NR52 => Some("NR52"),
        WAVE_RAM_START..=WAVE_RAM_END => Some("WAVE_RAM"),
        LCDC => Some("LCDC"),
        STAT => Some("STAT"),
        SCY => Some("SCY"),
        SCX => Some("SCX"),
        LY => Some("LY"),
        LYC => Some("LYC"),
        DMA => Some("DMA"),
        BGP => Some("BGP"),
        OBP0 => Some("OBP0"),
        OBP1 => Some("OBP1"),
        WY => Some("WY"),
        WX => Some("WX"),
        BOOTROM_DISABLE => Some("BOOTROM_DISABLE"),
        _ => None,
    }
}

/// アドレスの詳細情報を取得
pub fn get_address_info(addr: u16) -> String {
    let region = get_region_name(addr);
    
    if let Some(register_name) = get_io_register_name(addr) {
        format!("0x{:04X} [{}] {}", addr, region, register_name)
    } else {
        format!("0x{:04X} [{}]", addr, region)
    }
}

/// メモリマップ全体を表示
pub fn print_memory_map() {
    println!("=== GameBoy DMG Memory Map ===");
    println!("0x0000-0x00FF: BootROM (256B)");
    println!("0x0100-0x7FFF: Cartridge ROM (32KB-256B)");
    println!("0x8000-0x9FFF: Video RAM (8KB)");
    println!("0xA000-0xBFFF: Cartridge RAM (8KB)");
    println!("0xC000-0xDFFF: Work RAM (8KB)");
    println!("0xE000-0xFDFF: Work RAM Echo (Mirror)");
    println!("0xFE00-0xFE9F: OAM (160B)");
    println!("0xFEA0-0xFEFF: Unused");
    println!("0xFF00-0xFF7F: I/O Registers (128B)");
    println!("0xFF80-0xFFFE: High RAM (127B)");
    println!("0xFFFF:        Interrupt Enable (1B)");
}

pub fn analyze_address(addr: u16) {
    println!("=== Address Analysis: 0x{:04X} ===", addr);
    println!("Region: {}", get_region_name(addr));
    
    if let Some(register_name) = get_io_register_name(addr) {
        println!("Register: {}", register_name);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_regions() {
        assert_eq!(get_memory_region(0x0000), MemoryRegion::BootRom);
        assert_eq!(get_memory_region(0x0100), MemoryRegion::CartridgeRom);
        assert_eq!(get_memory_region(0x8000), MemoryRegion::VideoRam);
        assert_eq!(get_memory_region(0xC000), MemoryRegion::WorkRam);
        assert_eq!(get_memory_region(0xFF80), MemoryRegion::HighRam);
        assert_eq!(get_memory_region(0xFFFF), MemoryRegion::InterruptEnable);
    }
    
    #[test]
    fn test_io_register_names() {
        assert_eq!(get_io_register_name(0xFF40), Some("LCDC"));
        assert_eq!(get_io_register_name(0xFF41), Some("STAT"));
        assert_eq!(get_io_register_name(0xFF50), Some("BOOTROM_DISABLE"));
        assert_eq!(get_io_register_name(0xFF00), Some("JOYP"));
    }
    
    #[test]
    fn test_address_info() {
        let info = get_address_info(0xFF40);
        assert!(info.contains("LCDC"));
        assert!(info.contains("I/O Registers"));
    }
}

