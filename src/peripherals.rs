use crate::memory_map::{
    dmg::*,
    io_registers::*,
    get_region_name,
};
use crate::memory::{
    BootRom, WorkRam, HighRam,
};
use crate::ppu::Ppu;
use crate::cpu::timer::Timer;
use crate::joypad::Joypad;
use crate::dma::Dma;
use crate::cartridge::Cartridge;
use crate::serial::Serial;
use crate::apu::Apu;

pub struct Peripherals {
    bootrom: BootRom,
    wram: WorkRam,
    hram: HighRam,
    pub ppu: Ppu,
    pub timer: Timer,
    pub joypad: Joypad,
    pub dma: Dma,
    pub cartridge: Option<Cartridge>,
    pub serial: Serial,
    pub apu: Apu,

    // 割り込みレジスタ
    pub interrupt_flag: u8,     // IF (0xFF0F)
    pub interrupt_enable: u8,   // IE (0xFFFF)

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
            ppu: Ppu::new(),
            timer: Timer::new(),
            joypad: Joypad::new(),
            dma: Dma::new(),
            cartridge: None,
            serial: Serial::new(),
            apu: Apu::new(),
            interrupt_flag: 0x00,
            interrupt_enable: 0x00,
            read_count: 0,
            write_count: 0,
        }
    }

    /// ダミーBootROMでPeripheralsを作成（テスト用）
    pub fn new_with_dummy_bootrom() -> Self {
        Self::new(BootRom::new_dummy())
    }

    /// カートリッジをセット
    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
    }

    /// CPUサイクルに同期してPPU/Timer/DMA/Serial/APU/Cartridgeを進める
    pub fn tick(&mut self, cycles: u8) {
        for _ in 0..cycles {
            self.ppu.step();
            self.timer.tick();
            self.serial.tick();
            self.apu.tick();

            // カートリッジRTCティック
            if let Some(ref mut cart) = self.cartridge {
                cart.tick();
            }

            // DMA転送処理
            if let Some((src, dst)) = self.dma.tick() {
                let value = self.dma_read(src);
                let oam_offset = (dst - OAM_START) as usize;
                if oam_offset < 160 {
                    self.ppu.oam[oam_offset] = value;
                }
            }
        }

        // PPUの割り込みフラグをIFに反映
        if self.ppu.vblank_interrupt {
            self.interrupt_flag |= 0x01; // VBlank割り込み (bit 0)
            self.ppu.vblank_interrupt = false;
        }
        if self.ppu.stat_interrupt {
            self.interrupt_flag |= 0x02; // STAT割り込み (bit 1)
            self.ppu.stat_interrupt = false;
        }

        // Timerの割り込みフラグをIFに反映
        if self.timer.interrupt_request {
            self.interrupt_flag |= 0x04; // Timer割り込み (bit 2)
            self.timer.interrupt_request = false;
        }

        // Serialの割り込みフラグをIFに反映
        if self.serial.interrupt_request {
            self.interrupt_flag |= 0x08; // Serial割り込み (bit 3)
            self.serial.interrupt_request = false;
        }

        // Joypadの割り込みフラグをIFに反映
        if self.joypad.interrupt_request {
            self.interrupt_flag |= 0x10; // Joypad割り込み (bit 4)
            self.joypad.interrupt_request = false;
        }
    }

    /// DMA転送用の読み取り（OAMを除く全メモリからの読み取り）
    fn dma_read(&self, addr: u16) -> u8 {
        match addr {
            BOOTROM_START..=BOOTROM_END => {
                if self.bootrom.is_active() {
                    self.bootrom.read(addr)
                } else if let Some(ref cart) = self.cartridge {
                    cart.read_rom(addr)
                } else {
                    0xFF
                }
            }
            0x0100..=0x7FFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_rom(addr)
                } else {
                    0xFF
                }
            }
            VRAM_START..=VRAM_END => self.ppu.read_vram(addr),
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_ram(addr)
                } else {
                    0xFF
                }
            }
            WRAM_START..=WRAM_END => self.wram.read(addr),
            WRAM_ECHO_START..=WRAM_ECHO_END => {
                let wram_addr = WRAM_START + (addr - WRAM_ECHO_START);
                self.wram.read(wram_addr)
            }
            _ => 0xFF,
        }
    }
    
    /// 指定されたアドレスからデータを読み取る
    pub fn read(&mut self, addr: u16) -> u8 {
        self.read_count += 1;

        let value = match addr {
            // BootROM領域
            BOOTROM_START..=BOOTROM_END => {
                if self.bootrom.is_active() {
                    self.bootrom.read(addr)
                } else if let Some(ref cart) = self.cartridge {
                    cart.read_rom(addr)
                } else {
                    0xFF
                }
            }

            // カートリッジROM領域（BootROM以降）
            0x0100..=0x7FFF => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_rom(addr)
                } else {
                    0xFF
                }
            }

            // VRAM領域
            VRAM_START..=VRAM_END => {
                self.ppu.read_vram(addr)
            }

            // カートリッジRAM
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                if let Some(ref cart) = self.cartridge {
                    cart.read_ram(addr)
                } else {
                    0xFF
                }
            }

            // Work RAM領域
            WRAM_START..=WRAM_END => {
                self.wram.read(addr)
            }

            // Work RAM Echo領域（0xE000-0xFDFF）
            WRAM_ECHO_START..=WRAM_ECHO_END => {
                let wram_addr = WRAM_START + (addr - WRAM_ECHO_START);
                self.wram.read(wram_addr)
            }

            // OAM領域
            OAM_START..=OAM_END => {
                self.ppu.read_oam(addr)
            }

            // 未使用領域
            UNUSED_START..=UNUSED_END => 0xFF,

            // I/Oレジスタ領域
            IO_REGISTERS_START..=IO_REGISTERS_END => {
                self.read_io(addr)
            }

            // High RAM領域
            HRAM_START..=HRAM_END => {
                self.hram.read(addr)
            }

            // 割り込み許可レジスタ
            IE_REGISTER => self.interrupt_enable,

            // ここには到達しないはず（u16の全範囲をカバー済み）
        };

        #[cfg(feature = "trace_memory")]
        println!("READ  0x{:04X} = 0x{:02X} [{}]", addr, value, get_region_name(addr));

        value
    }

    /// I/Oレジスタの読み取り
    fn read_io(&self, addr: u16) -> u8 {
        match addr {
            // ジョイパッド
            JOYP => self.joypad.read(),

            // シリアル通信
            SB => self.serial.read_sb(),
            SC => self.serial.read_sc(),

            // PPUレジスタ
            LCDC => self.ppu.registers.lcdc,
            STAT => {
                // STATの下位3bitはPPU状態から構成
                let mode = self.ppu.mode as u8;
                let lyc_flag = if self.ppu.scanline == self.ppu.registers.lyc { 0x04 } else { 0x00 };
                (self.ppu.registers.stat & 0xF8) | lyc_flag | mode
            }
            SCY => self.ppu.registers.scy,
            SCX => self.ppu.registers.scx,
            LY => self.ppu.scanline,
            LYC => self.ppu.registers.lyc,
            DMA => self.dma.read(),
            BGP => self.ppu.registers.bgp,
            OBP0 => self.ppu.registers.obp0,
            OBP1 => self.ppu.registers.obp1,
            WY => self.ppu.registers.wy,
            WX => self.ppu.registers.wx,

            // タイマーレジスタ
            DIV => self.timer.read_div(),
            TIMA => self.timer.tima,
            TMA => self.timer.tma,
            TAC => self.timer.tac | 0xF8, // 上位5bitは常に1

            // APUレジスタ + Wave RAM
            NR10..=NR52 | WAVE_RAM_START..=WAVE_RAM_END => self.apu.read(addr),

            // 割り込みフラグ
            IF => self.interrupt_flag | 0xE0, // 上位3bitは常に1

            // BootROM無効化レジスタ
            BOOTROM_DISABLE => 0xFF,

            // その他のI/Oレジスタ（未実装）
            _ => {
                #[cfg(feature = "trace_memory")]
                println!("未実装I/Oレジスタ読み取り: 0x{:04X}", addr);
                0xFF
            }
        }
    }
    
    /// 指定されたアドレスにデータを書き込む
    pub fn write(&mut self, addr: u16, value: u8) {
        self.write_count += 1;

        #[cfg(feature = "trace_memory")]
        println!("WRITE 0x{:04X} = 0x{:02X} [{}]", addr, value, get_region_name(addr));

        match addr {
            // BootROM/カートリッジROM Bank 0 領域（MBCレジスタ操作）
            BOOTROM_START..=BOOTROM_END => {
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_rom(addr, value);
                }
            }

            // カートリッジROM領域（MBCレジスタ操作）
            0x0100..=0x7FFF => {
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_rom(addr, value);
                }
            }

            // VRAM領域
            VRAM_START..=VRAM_END => {
                self.ppu.write_vram(addr, value);
            }

            // カートリッジRAM
            CARTRIDGE_RAM_START..=CARTRIDGE_RAM_END => {
                if let Some(ref mut cart) = self.cartridge {
                    cart.write_ram(addr, value);
                }
            }

            // Work RAM領域
            WRAM_START..=WRAM_END => {
                self.wram.write(addr, value);
            }

            // Work RAM Echo領域
            WRAM_ECHO_START..=WRAM_ECHO_END => {
                let wram_addr = WRAM_START + (addr - WRAM_ECHO_START);
                self.wram.write(wram_addr, value);
            }

            // OAM領域
            OAM_START..=OAM_END => {
                self.ppu.write_oam(addr, value);
            }

            // 未使用領域
            UNUSED_START..=UNUSED_END => {}

            // I/Oレジスタ領域
            IO_REGISTERS_START..=IO_REGISTERS_END => {
                self.write_io(addr, value);
            }

            // High RAM領域
            HRAM_START..=HRAM_END => {
                self.hram.write(addr, value);
            }

            // 割り込み許可レジスタ
            IE_REGISTER => {
                self.interrupt_enable = value;
            }
        }
    }

    /// I/Oレジスタへの書き込み
    fn write_io(&mut self, addr: u16, value: u8) {
        match addr {
            // ジョイパッド
            JOYP => self.joypad.write(value),

            // シリアル通信
            SB => self.serial.write_sb(value),
            SC => self.serial.write_sc(value),

            // PPUレジスタ
            LCDC => self.ppu.registers.lcdc = value,
            STAT => {
                // STATの下位3bitは読み取り専用（PPU状態）
                self.ppu.registers.stat = (value & 0xF8) | (self.ppu.registers.stat & 0x07);
            }
            SCY => self.ppu.registers.scy = value,
            SCX => self.ppu.registers.scx = value,
            LY => {} // LYは読み取り専用
            LYC => self.ppu.registers.lyc = value,
            DMA => self.dma.start(value),
            BGP => self.ppu.registers.bgp = value,
            OBP0 => self.ppu.registers.obp0 = value,
            OBP1 => self.ppu.registers.obp1 = value,
            WY => self.ppu.registers.wy = value,
            WX => self.ppu.registers.wx = value,

            // タイマーレジスタ
            DIV => self.timer.write_div(),
            TIMA => self.timer.tima = value,
            TMA => self.timer.tma = value,
            TAC => self.timer.tac = value & 0x07,

            // APUレジスタ + Wave RAM
            NR10..=NR52 | WAVE_RAM_START..=WAVE_RAM_END => self.apu.write(addr, value),

            // 割り込みフラグ
            IF => {
                self.interrupt_flag = value & 0x1F; // 下位5bitのみ
            }

            // BootROM無効化レジスタ
            BOOTROM_DISABLE => {
                self.bootrom.write_disable_register(value);
            }

            // その他のI/Oレジスタ（未実装）
            _ => {
                #[cfg(feature = "trace_memory")]
                println!("未実装I/Oレジスタ書き込み: 0x{:04X} = 0x{:02X}", addr, value);
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

    #[test]
    fn test_peripherals_vram() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // VRAM書き込み・読み取り
        peripherals.write(0x8000, 0xAA);
        peripherals.write(0x9FFF, 0x55);

        assert_eq!(peripherals.read(0x8000), 0xAA);
        assert_eq!(peripherals.read(0x9FFF), 0x55);
    }

    #[test]
    fn test_peripherals_oam() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // PPUをHBlankモードにしてOAM書き込みを許可
        peripherals.ppu.mode = crate::ppu::PpuMode::HBlank;

        peripherals.write(0xFE00, 0x10);
        peripherals.write(0xFE9F, 0x20);

        assert_eq!(peripherals.read(0xFE00), 0x10);
        assert_eq!(peripherals.read(0xFE9F), 0x20);
    }

    #[test]
    fn test_peripherals_ppu_registers() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // BGP書き込み・読み取り
        peripherals.write(0xFF47, 0xE4);
        assert_eq!(peripherals.read(0xFF47), 0xE4);

        // SCY/SCX書き込み・読み取り
        peripherals.write(0xFF42, 0x10);
        peripherals.write(0xFF43, 0x20);
        assert_eq!(peripherals.read(0xFF42), 0x10);
        assert_eq!(peripherals.read(0xFF43), 0x20);

        // LCDC書き込み・読み取り
        peripherals.write(0xFF40, 0x91);
        assert_eq!(peripherals.read(0xFF40), 0x91);
    }

    #[test]
    fn test_peripherals_interrupt_registers() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // IE書き込み・読み取り
        peripherals.write(0xFFFF, 0x1F);
        assert_eq!(peripherals.read(0xFFFF), 0x1F);

        // IF書き込み・読み取り（上位3bitは常に1）
        peripherals.write(0xFF0F, 0x05);
        assert_eq!(peripherals.read(0xFF0F), 0x05 | 0xE0);
    }

    #[test]
    fn test_peripherals_tick_vblank() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // VBlankまでPPUを進める（144スキャンライン × 456サイクル）
        let cycles_to_vblank = 144 * 456;
        for _ in 0..cycles_to_vblank {
            peripherals.tick(1);
        }

        // VBlank割り込みがIFに反映されているはず
        assert_ne!(peripherals.interrupt_flag & 0x01, 0);
    }

    #[test]
    fn test_peripherals_joypad() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // JOYP書き込み：方向キー選択
        peripherals.write(0xFF00, 0x20);

        // ジョイパッドにボタン押下
        peripherals.joypad.press(crate::joypad::JoypadButton::Right);

        // 読み取り: Rightが押されている
        let joyp = peripherals.read(0xFF00);
        assert_eq!(joyp & 0x01, 0x00); // bit0=0 (Right押下)
    }

    #[test]
    fn test_peripherals_dma() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // WRAMにテストデータを配置
        for i in 0..160u8 {
            peripherals.write(0xC000 + i as u16, i);
        }

        // DMA開始
        peripherals.write(0xFF46, 0xC0); // 転送元: 0xC000

        // DMA完了まで実行
        for _ in 0..700 {
            peripherals.tick(1);
        }

        // PPUをHBlankモードに変更してOAM読み取りを許可
        peripherals.ppu.mode = crate::ppu::PpuMode::HBlank;

        // OAMにデータがコピーされているか確認
        assert_eq!(peripherals.read(0xFE00), 0);
        assert_eq!(peripherals.read(0xFE01), 1);
        assert_eq!(peripherals.read(0xFE9F), 159);
    }

    #[test]
    fn test_peripherals_obp_registers() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // OBP0/OBP1書き込み・読み取り
        peripherals.write(0xFF48, 0xE4);
        peripherals.write(0xFF49, 0x1B);
        assert_eq!(peripherals.read(0xFF48), 0xE4);
        assert_eq!(peripherals.read(0xFF49), 0x1B);

        // WY/WX書き込み・読み取り
        peripherals.write(0xFF4A, 0x10);
        peripherals.write(0xFF4B, 0x07);
        assert_eq!(peripherals.read(0xFF4A), 0x10);
        assert_eq!(peripherals.read(0xFF4B), 0x07);
    }

    #[test]
    fn test_peripherals_cartridge_rom() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // BootROMを無効化
        peripherals.write(0xFF50, 0x01);

        // カートリッジ未装着時は0xFF
        assert_eq!(peripherals.read(0x0100), 0xFF);

        // テスト用ROMを作成してセット
        let mut rom_data = vec![0u8; 0x8000];
        rom_data[0x0100] = 0x42;
        rom_data[0x4000] = 0x99;
        let cart = crate::cartridge::Cartridge::new_rom_only(rom_data);
        peripherals.load_cartridge(cart);

        // カートリッジROMが読める
        assert_eq!(peripherals.read(0x0100), 0x42);
        assert_eq!(peripherals.read(0x4000), 0x99);
    }

    #[test]
    fn test_peripherals_joypad_interrupt() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();
        peripherals.joypad.write(0x20); // 方向キー選択

        // ボタン押下で割り込み発生
        peripherals.joypad.press(crate::joypad::JoypadButton::Right);
        peripherals.tick(1);

        // Joypad割り込みがIFに反映
        assert_ne!(peripherals.interrupt_flag & 0x10, 0);
    }

    #[test]
    fn test_peripherals_serial() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // SBレジスタ書き込み・読み取り
        peripherals.write(0xFF01, 0x42);
        assert_eq!(peripherals.read(0xFF01), 0x42);

        // SCレジスタ書き込み・読み取り
        peripherals.write(0xFF02, 0x81);
        assert_eq!(peripherals.read(0xFF02) & 0x81, 0x81);
    }

    #[test]
    fn test_peripherals_serial_interrupt() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        peripherals.write(0xFF01, 0xAB);
        peripherals.write(0xFF02, 0x81); // 内部クロックで転送開始

        // 転送完了まで（4096サイクル）
        for _ in 0..4096 {
            peripherals.tick(1);
        }

        // Serial割り込みがIFに反映
        assert_ne!(peripherals.interrupt_flag & 0x08, 0);
    }

    #[test]
    fn test_peripherals_apu_registers() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // NR52でAPU電源オン
        peripherals.write(0xFF26, 0x80);
        assert_eq!(peripherals.read(0xFF26) & 0x80, 0x80);

        // NR50 マスター音量
        peripherals.write(0xFF24, 0x77);
        assert_eq!(peripherals.read(0xFF24), 0x77);

        // NR51 パニング
        peripherals.write(0xFF25, 0xFF);
        assert_eq!(peripherals.read(0xFF25), 0xFF);
    }

    #[test]
    fn test_peripherals_apu_wave_ram() {
        let mut peripherals = Peripherals::new_with_dummy_bootrom();

        // Wave RAM書き込み（APU電源に関係なく可能）
        peripherals.write(0xFF30, 0x12);
        peripherals.write(0xFF3F, 0xAB);
        assert_eq!(peripherals.read(0xFF30), 0x12);
        assert_eq!(peripherals.read(0xFF3F), 0xAB);
    }
}
