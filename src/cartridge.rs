// src/cartridge.rs
// GameBoy カートリッジ・MBCシステム
//
// カートリッジヘッダ (0x0100-0x014F):
//   0x0100-0x0103: エントリポイント
//   0x0104-0x0133: Nintendoロゴ
//   0x0134-0x0143: タイトル
//   0x0147: カートリッジタイプ (MBC種別)
//   0x0148: ROMサイズ
//   0x0149: RAMサイズ
//
// MBC種別:
//   0x00: ROM ONLY (MBCなし)
//   0x01: MBC1
//   0x02: MBC1+RAM
//   0x03: MBC1+RAM+BATTERY

/// カートリッジタイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Unknown(u8),
}

impl CartridgeType {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            other => CartridgeType::Unknown(other),
        }
    }

    fn has_mbc1(&self) -> bool {
        matches!(self, CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery)
    }

    fn has_ram(&self) -> bool {
        matches!(self, CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery)
    }
}

/// ROMサイズ (バンク数)
fn rom_banks_from_byte(byte: u8) -> usize {
    match byte {
        0x00 => 2,   // 32KB
        0x01 => 4,   // 64KB
        0x02 => 8,   // 128KB
        0x03 => 16,  // 256KB
        0x04 => 32,  // 512KB
        0x05 => 64,  // 1MB
        0x06 => 128, // 2MB
        0x07 => 256, // 4MB
        0x08 => 512, // 8MB
        _ => 2,
    }
}

/// RAMサイズ (バイト数)
fn ram_size_from_byte(byte: u8) -> usize {
    match byte {
        0x00 => 0,
        0x01 => 2 * 1024,    // 2KB (未使用だが定義)
        0x02 => 8 * 1024,    // 8KB
        0x03 => 32 * 1024,   // 32KB (4バンク)
        0x04 => 128 * 1024,  // 128KB (16バンク)
        0x05 => 64 * 1024,   // 64KB (8バンク)
        _ => 0,
    }
}

/// カートリッジヘッダ情報
#[derive(Debug)]
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: CartridgeType,
    pub rom_banks: usize,
    pub ram_size: usize,
}

/// MBC1バンキングモード
#[derive(Debug, Clone, Copy, PartialEq)]
enum Mbc1Mode {
    Rom,  // モード0: ROMバンキング (デフォルト)
    Ram,  // モード1: RAM バンキング
}

/// カートリッジ
pub struct Cartridge {
    /// ROMデータ
    rom: Vec<u8>,
    /// 外部RAM
    ram: Vec<u8>,
    /// ヘッダ情報
    pub header: CartridgeHeader,

    // MBC1レジスタ
    /// RAM有効フラグ
    ram_enabled: bool,
    /// ROMバンク番号 (下位5bit)
    rom_bank: u8,
    /// RAM バンク番号 / ROMバンク上位2bit
    ram_bank: u8,
    /// バンキングモード
    banking_mode: Mbc1Mode,
}

impl Cartridge {
    /// ROMデータからカートリッジを作成
    pub fn new(rom_data: Vec<u8>) -> Result<Self, String> {
        if rom_data.len() < 0x150 {
            return Err("ROMデータが小さすぎます（ヘッダが不足）".to_string());
        }

        let header = Self::parse_header(&rom_data);
        let ram_size = header.ram_size;

        // MBC1+RAMの場合、最低8KBのRAMを確保
        let actual_ram_size = if header.cartridge_type.has_ram() && ram_size == 0 {
            8 * 1024
        } else {
            ram_size
        };

        Ok(Self {
            rom: rom_data,
            ram: vec![0; actual_ram_size],
            header,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: Mbc1Mode::Rom,
        })
    }

    /// ROM ONLYカートリッジを作成（テスト用）
    pub fn new_rom_only(rom_data: Vec<u8>) -> Self {
        let len = rom_data.len();
        let mut padded = rom_data;
        if len < 0x8000 {
            padded.resize(0x8000, 0xFF);
        }

        Self {
            rom: padded,
            ram: vec![0; 0],
            header: CartridgeHeader {
                title: "TEST".to_string(),
                cartridge_type: CartridgeType::RomOnly,
                rom_banks: 2,
                ram_size: 0,
            },
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: Mbc1Mode::Rom,
        }
    }

    /// ヘッダを解析
    fn parse_header(rom: &[u8]) -> CartridgeHeader {
        // タイトル (0x0134-0x0143)
        let title_bytes = &rom[0x0134..=0x0143];
        let title = title_bytes.iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect::<String>();

        let cartridge_type = CartridgeType::from_byte(rom[0x0147]);
        let rom_banks = rom_banks_from_byte(rom[0x0148]);
        let ram_size = ram_size_from_byte(rom[0x0149]);

        CartridgeHeader {
            title,
            cartridge_type,
            rom_banks,
            ram_size,
        }
    }

    /// ROM領域の読み取り (0x0000-0x7FFF)
    pub fn read_rom(&self, addr: u16) -> u8 {
        match addr {
            // Bank 0 (0x0000-0x3FFF) — 常にバンク0
            0x0000..=0x3FFF => {
                if self.header.cartridge_type == CartridgeType::RomOnly {
                    self.rom.get(addr as usize).copied().unwrap_or(0xFF)
                } else {
                    // MBC1 モード1ではBank 0の代わりに0x20/0x40/0x60バンク
                    let bank = if self.banking_mode == Mbc1Mode::Ram {
                        (self.ram_bank as usize) << 5
                    } else {
                        0
                    };
                    let offset = bank * 0x4000 + addr as usize;
                    self.rom.get(offset).copied().unwrap_or(0xFF)
                }
            }
            // Bank N (0x4000-0x7FFF)
            0x4000..=0x7FFF => {
                if self.header.cartridge_type == CartridgeType::RomOnly {
                    self.rom.get(addr as usize).copied().unwrap_or(0xFF)
                } else {
                    let bank = self.effective_rom_bank();
                    let offset = bank * 0x4000 + (addr as usize - 0x4000);
                    self.rom.get(offset).copied().unwrap_or(0xFF)
                }
            }
            _ => 0xFF,
        }
    }

    /// ROM領域への書き込み (MBCレジスタ操作)
    pub fn write_rom(&mut self, addr: u16, value: u8) {
        if !self.header.cartridge_type.has_mbc1() {
            return; // ROM ONLYは書き込み不可
        }

        match addr {
            // RAM Enable (0x0000-0x1FFF)
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            // ROM Bank Number (0x2000-0x3FFF) — 下位5bit
            0x2000..=0x3FFF => {
                let bank = value & 0x1F;
                self.rom_bank = if bank == 0 { 1 } else { bank };
            }
            // RAM Bank Number / Upper ROM Bank (0x4000-0x5FFF)
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x03;
            }
            // Banking Mode (0x6000-0x7FFF)
            0x6000..=0x7FFF => {
                self.banking_mode = if value & 0x01 == 0 {
                    Mbc1Mode::Rom
                } else {
                    Mbc1Mode::Ram
                };
            }
            _ => {}
        }
    }

    /// 外部RAM読み取り (0xA000-0xBFFF)
    pub fn read_ram(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }

        let bank = if self.banking_mode == Mbc1Mode::Ram {
            self.ram_bank as usize
        } else {
            0
        };
        let offset = bank * 0x2000 + (addr as usize - 0xA000);
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    /// 外部RAM書き込み (0xA000-0xBFFF)
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }

        let bank = if self.banking_mode == Mbc1Mode::Ram {
            self.ram_bank as usize
        } else {
            0
        };
        let offset = bank * 0x2000 + (addr as usize - 0xA000);
        if offset < self.ram.len() {
            self.ram[offset] = value;
        }
    }

    /// 実効ROMバンク番号を計算
    fn effective_rom_bank(&self) -> usize {
        let bank = (self.ram_bank as usize) << 5 | (self.rom_bank as usize);
        bank % self.header.rom_banks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom(size: usize, cart_type: u8) -> Vec<u8> {
        let mut rom = vec![0u8; size];
        // エントリポイント
        rom[0x0100] = 0x00; // NOP
        // タイトル
        let title = b"TEST";
        rom[0x0134..0x0134 + title.len()].copy_from_slice(title);
        // カートリッジタイプ
        rom[0x0147] = cart_type;
        // ROMサイズ (32KB = 0x00)
        rom[0x0148] = 0x00;
        // RAMサイズ
        rom[0x0149] = 0x00;
        rom
    }

    #[test]
    fn test_rom_only_cartridge() {
        let mut rom = create_test_rom(0x8000, 0x00);
        rom[0x0000] = 0x31; // テスト用データ
        rom[0x7FFF] = 0x42;

        let cart = Cartridge::new(rom).unwrap();
        assert_eq!(cart.header.cartridge_type, CartridgeType::RomOnly);
        assert_eq!(cart.read_rom(0x0000), 0x31);
        assert_eq!(cart.read_rom(0x7FFF), 0x42);
    }

    #[test]
    fn test_cartridge_header_parse() {
        let rom = create_test_rom(0x8000, 0x01);
        let cart = Cartridge::new(rom).unwrap();
        assert_eq!(cart.header.title, "TEST");
        assert_eq!(cart.header.cartridge_type, CartridgeType::Mbc1);
        assert_eq!(cart.header.rom_banks, 2);
    }

    #[test]
    fn test_mbc1_rom_bank_switching() {
        // 64KB ROM (4バンク)
        let mut rom = create_test_rom(0x10000, 0x01);
        rom[0x0148] = 0x01; // 64KB

        // 各バンクの先頭にテスト用データ配置
        rom[0x4000] = 0x11; // Bank 1
        rom[0x8000] = 0x22; // Bank 2
        rom[0xC000] = 0x33; // Bank 3

        let mut cart = Cartridge::new(rom).unwrap();

        // デフォルトはバンク1
        assert_eq!(cart.read_rom(0x4000), 0x11);

        // バンク2に切り替え
        cart.write_rom(0x2000, 0x02);
        assert_eq!(cart.read_rom(0x4000), 0x22);

        // バンク3に切り替え
        cart.write_rom(0x2000, 0x03);
        assert_eq!(cart.read_rom(0x4000), 0x33);
    }

    #[test]
    fn test_mbc1_bank0_redirect() {
        // バンク0への書き込みは自動的にバンク1にリダイレクト
        let rom = create_test_rom(0x8000, 0x01);
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x2000, 0x00); // バンク0を指定
        assert_eq!(cart.rom_bank, 1); // バンク1にリダイレクト
    }

    #[test]
    fn test_mbc1_ram() {
        let mut rom = create_test_rom(0x8000, 0x02); // MBC1+RAM
        rom[0x0149] = 0x02; // 8KB RAM

        let mut cart = Cartridge::new(rom).unwrap();

        // RAMが無効な場合は0xFFを返す
        assert_eq!(cart.read_ram(0xA000), 0xFF);

        // RAM有効化
        cart.write_rom(0x0000, 0x0A);
        cart.write_ram(0xA000, 0x42);
        assert_eq!(cart.read_ram(0xA000), 0x42);

        // RAM無効化
        cart.write_rom(0x0000, 0x00);
        assert_eq!(cart.read_ram(0xA000), 0xFF);
    }

    #[test]
    fn test_rom_too_small() {
        let rom = vec![0u8; 0x100]; // ヘッダが不足
        assert!(Cartridge::new(rom).is_err());
    }

    #[test]
    fn test_new_rom_only_convenience() {
        let rom = vec![0x00; 0x100]; // 小さなROM
        let cart = Cartridge::new_rom_only(rom);
        assert_eq!(cart.header.cartridge_type, CartridgeType::RomOnly);
        // 32KBにパディングされている
        assert_eq!(cart.rom.len(), 0x8000);
    }
}
