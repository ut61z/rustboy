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
//   0x05: MBC2
//   0x06: MBC2+BATTERY
//   0x0F: MBC3+TIMER+BATTERY
//   0x10: MBC3+TIMER+RAM+BATTERY
//   0x11: MBC3
//   0x12: MBC3+RAM
//   0x13: MBC3+RAM+BATTERY
//   0x19: MBC5
//   0x1A: MBC5+RAM
//   0x1B: MBC5+RAM+BATTERY
//   0x1C: MBC5+RUMBLE
//   0x1D: MBC5+RUMBLE+RAM
//   0x1E: MBC5+RUMBLE+RAM+BATTERY

/// カートリッジタイプ
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CartridgeType {
    RomOnly,
    Mbc1,
    Mbc1Ram,
    Mbc1RamBattery,
    Mbc2,
    Mbc2Battery,
    Mbc3TimerBattery,
    Mbc3TimerRamBattery,
    Mbc3,
    Mbc3Ram,
    Mbc3RamBattery,
    Mbc5,
    Mbc5Ram,
    Mbc5RamBattery,
    Mbc5Rumble,
    Mbc5RumbleRam,
    Mbc5RumbleRamBattery,
    Unknown(u8),
}

impl CartridgeType {
    fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => CartridgeType::RomOnly,
            0x01 => CartridgeType::Mbc1,
            0x02 => CartridgeType::Mbc1Ram,
            0x03 => CartridgeType::Mbc1RamBattery,
            0x05 => CartridgeType::Mbc2,
            0x06 => CartridgeType::Mbc2Battery,
            0x0F => CartridgeType::Mbc3TimerBattery,
            0x10 => CartridgeType::Mbc3TimerRamBattery,
            0x11 => CartridgeType::Mbc3,
            0x12 => CartridgeType::Mbc3Ram,
            0x13 => CartridgeType::Mbc3RamBattery,
            0x19 => CartridgeType::Mbc5,
            0x1A => CartridgeType::Mbc5Ram,
            0x1B => CartridgeType::Mbc5RamBattery,
            0x1C => CartridgeType::Mbc5Rumble,
            0x1D => CartridgeType::Mbc5RumbleRam,
            0x1E => CartridgeType::Mbc5RumbleRamBattery,
            other => CartridgeType::Unknown(other),
        }
    }

    /// MBCコントローラの種別を返す
    fn mbc_kind(&self) -> MbcKind {
        match self {
            CartridgeType::RomOnly => MbcKind::None,
            CartridgeType::Mbc1 | CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery => MbcKind::Mbc1,
            CartridgeType::Mbc2 | CartridgeType::Mbc2Battery => MbcKind::Mbc2,
            CartridgeType::Mbc3 | CartridgeType::Mbc3Ram | CartridgeType::Mbc3RamBattery
            | CartridgeType::Mbc3TimerBattery | CartridgeType::Mbc3TimerRamBattery => MbcKind::Mbc3,
            CartridgeType::Mbc5 | CartridgeType::Mbc5Ram | CartridgeType::Mbc5RamBattery
            | CartridgeType::Mbc5Rumble | CartridgeType::Mbc5RumbleRam | CartridgeType::Mbc5RumbleRamBattery => MbcKind::Mbc5,
            CartridgeType::Unknown(_) => MbcKind::None,
        }
    }

    fn has_ram(&self) -> bool {
        matches!(self,
            CartridgeType::Mbc1Ram | CartridgeType::Mbc1RamBattery
            | CartridgeType::Mbc2 | CartridgeType::Mbc2Battery
            | CartridgeType::Mbc3Ram | CartridgeType::Mbc3RamBattery
            | CartridgeType::Mbc3TimerRamBattery
            | CartridgeType::Mbc5Ram | CartridgeType::Mbc5RamBattery
            | CartridgeType::Mbc5RumbleRam | CartridgeType::Mbc5RumbleRamBattery
        )
    }

    fn has_timer(&self) -> bool {
        matches!(self,
            CartridgeType::Mbc3TimerBattery | CartridgeType::Mbc3TimerRamBattery
        )
    }
}

/// MBCコントローラ種別
#[derive(Debug, Clone, Copy, PartialEq)]
enum MbcKind {
    None,
    Mbc1,
    Mbc2,
    Mbc3,
    Mbc5,
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

/// MBC3 RTCレジスタ
#[derive(Debug, Clone)]
struct RtcRegisters {
    /// 秒 (0-59)
    seconds: u8,
    /// 分 (0-59)
    minutes: u8,
    /// 時 (0-23)
    hours: u8,
    /// 日 下位8ビット
    days_low: u8,
    /// 日 上位ビット + 制御フラグ
    /// Bit 0: 日カウンタ上位ビット (bit8)
    /// Bit 6: 停止フラグ (0=動作中, 1=停止)
    /// Bit 7: 日カウンタオーバーフロー
    days_high: u8,
}

impl RtcRegisters {
    fn new() -> Self {
        Self {
            seconds: 0,
            minutes: 0,
            hours: 0,
            days_low: 0,
            days_high: 0,
        }
    }

    /// RTCを1秒進める
    fn tick_second(&mut self) {
        // 停止中は進めない
        if self.days_high & 0x40 != 0 {
            return;
        }

        self.seconds += 1;
        if self.seconds >= 60 {
            self.seconds = 0;
            self.minutes += 1;
            if self.minutes >= 60 {
                self.minutes = 0;
                self.hours += 1;
                if self.hours >= 24 {
                    self.hours = 0;
                    let days = self.day_counter() + 1;
                    self.days_low = days as u8;
                    if days > 0x1FF {
                        // オーバーフロー
                        self.days_high = (self.days_high & 0xFE) | 0x80; // bit7=1, bit0=0
                        self.days_low = 0;
                    } else {
                        self.days_high = (self.days_high & 0xFE) | ((days >> 8) as u8 & 0x01);
                    }
                }
            }
        }
    }

    /// 日カウンタ値 (0-511)
    fn day_counter(&self) -> u16 {
        self.days_low as u16 | ((self.days_high as u16 & 0x01) << 8)
    }
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
    /// ROMバンク番号 (下位5bit for MBC1, 4bit for MBC2, 7bit for MBC3, 9bit for MBC5)
    rom_bank: u16,
    /// RAM バンク番号 / ROMバンク上位2bit
    ram_bank: u8,
    /// MBC1バンキングモード
    banking_mode: Mbc1Mode,

    // MBC3 RTC
    /// RTCレジスタ (現在値)
    rtc: RtcRegisters,
    /// RTCレジスタ (ラッチ値)
    rtc_latched: RtcRegisters,
    /// RTCラッチ前回値 (0x00→0x01のシーケンスで検出)
    rtc_latch_pending: bool,
    /// RTCマッピング (0x08-0x0C でRAMの代わりにRTCレジスタを選択)
    rtc_mapped: bool,
    /// RTC秒カウンタ (CPUサイクル→秒への変換)
    rtc_cycle_counter: u32,
}

/// CPUサイクル→1秒 (4,194,304サイクル)
const CYCLES_PER_SECOND: u32 = 4_194_304;

impl Cartridge {
    /// ROMデータからカートリッジを作成
    pub fn new(rom_data: Vec<u8>) -> Result<Self, String> {
        if rom_data.len() < 0x150 {
            return Err("ROMデータが小さすぎます（ヘッダが不足）".to_string());
        }

        let header = Self::parse_header(&rom_data);
        let ram_size = header.ram_size;

        // MBC種別に応じたRAMサイズ決定
        let actual_ram_size = match header.cartridge_type.mbc_kind() {
            MbcKind::Mbc2 => 512, // MBC2: 512×4ビット内蔵RAM
            _ => {
                if header.cartridge_type.has_ram() && ram_size == 0 {
                    8 * 1024 // 最低8KB
                } else {
                    ram_size
                }
            }
        };

        Ok(Self {
            rom: rom_data,
            ram: vec![0; actual_ram_size],
            header,
            ram_enabled: false,
            rom_bank: 1,
            ram_bank: 0,
            banking_mode: Mbc1Mode::Rom,
            rtc: RtcRegisters::new(),
            rtc_latched: RtcRegisters::new(),
            rtc_latch_pending: false,
            rtc_mapped: false,
            rtc_cycle_counter: 0,
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
            rtc: RtcRegisters::new(),
            rtc_latched: RtcRegisters::new(),
            rtc_latch_pending: false,
            rtc_mapped: false,
            rtc_cycle_counter: 0,
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

    /// カートリッジを1 CPUサイクル進める (RTC用)
    pub fn tick(&mut self) {
        if !self.header.cartridge_type.has_timer() {
            return;
        }

        self.rtc_cycle_counter += 1;
        if self.rtc_cycle_counter >= CYCLES_PER_SECOND {
            self.rtc_cycle_counter = 0;
            self.rtc.tick_second();
        }
    }

    /// ROM領域の読み取り (0x0000-0x7FFF)
    pub fn read_rom(&self, addr: u16) -> u8 {
        match self.header.cartridge_type.mbc_kind() {
            MbcKind::None => self.read_rom_none(addr),
            MbcKind::Mbc1 => self.read_rom_mbc1(addr),
            MbcKind::Mbc2 => self.read_rom_mbc2(addr),
            MbcKind::Mbc3 => self.read_rom_mbc3(addr),
            MbcKind::Mbc5 => self.read_rom_mbc5(addr),
        }
    }

    /// ROM領域への書き込み (MBCレジスタ操作)
    pub fn write_rom(&mut self, addr: u16, value: u8) {
        match self.header.cartridge_type.mbc_kind() {
            MbcKind::None => {} // ROM ONLYは書き込み不可
            MbcKind::Mbc1 => self.write_rom_mbc1(addr, value),
            MbcKind::Mbc2 => self.write_rom_mbc2(addr, value),
            MbcKind::Mbc3 => self.write_rom_mbc3(addr, value),
            MbcKind::Mbc5 => self.write_rom_mbc5(addr, value),
        }
    }

    /// 外部RAM読み取り (0xA000-0xBFFF)
    pub fn read_ram(&self, addr: u16) -> u8 {
        match self.header.cartridge_type.mbc_kind() {
            MbcKind::None => 0xFF,
            MbcKind::Mbc1 => self.read_ram_mbc1(addr),
            MbcKind::Mbc2 => self.read_ram_mbc2(addr),
            MbcKind::Mbc3 => self.read_ram_mbc3(addr),
            MbcKind::Mbc5 => self.read_ram_mbc5(addr),
        }
    }

    /// 外部RAM書き込み (0xA000-0xBFFF)
    pub fn write_ram(&mut self, addr: u16, value: u8) {
        match self.header.cartridge_type.mbc_kind() {
            MbcKind::None => {}
            MbcKind::Mbc1 => self.write_ram_mbc1(addr, value),
            MbcKind::Mbc2 => self.write_ram_mbc2(addr, value),
            MbcKind::Mbc3 => self.write_ram_mbc3(addr, value),
            MbcKind::Mbc5 => self.write_ram_mbc5(addr, value),
        }
    }

    // ===== ROM ONLY =====

    fn read_rom_none(&self, addr: u16) -> u8 {
        self.rom.get(addr as usize).copied().unwrap_or(0xFF)
    }

    // ===== MBC1 =====

    fn read_rom_mbc1(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                let bank = if self.banking_mode == Mbc1Mode::Ram {
                    (self.ram_bank as usize) << 5
                } else {
                    0
                };
                let offset = bank * 0x4000 + addr as usize;
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let bank = self.effective_rom_bank_mbc1();
                let offset = bank * 0x4000 + (addr as usize - 0x4000);
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_rom_mbc1(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                let bank = value & 0x1F;
                self.rom_bank = if bank == 0 { 1 } else { bank as u16 };
            }
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x03;
            }
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

    fn read_ram_mbc1(&self, addr: u16) -> u8 {
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

    fn write_ram_mbc1(&mut self, addr: u16, value: u8) {
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

    fn effective_rom_bank_mbc1(&self) -> usize {
        let bank = (self.ram_bank as usize) << 5 | (self.rom_bank as usize);
        bank % self.header.rom_banks
    }

    // ===== MBC2 =====

    fn read_rom_mbc2(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                self.rom.get(addr as usize).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let bank = (self.rom_bank as usize) % self.header.rom_banks;
                let offset = bank * 0x4000 + (addr as usize - 0x4000);
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_rom_mbc2(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM有効/無効 — アドレスのbit8が0
            0x0000..=0x3FFF => {
                if addr & 0x0100 == 0 {
                    // RAM Enable/Disable
                    self.ram_enabled = (value & 0x0F) == 0x0A;
                } else {
                    // ROM Bank Number (下位4ビット)
                    let bank = value & 0x0F;
                    self.rom_bank = if bank == 0 { 1 } else { bank as u16 };
                }
            }
            _ => {}
        }
    }

    fn read_ram_mbc2(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }
        // MBC2 RAM: 512×4ビット、アドレスの下位9ビットでアクセス
        let offset = (addr as usize - 0xA000) & 0x01FF;
        if offset < self.ram.len() {
            self.ram[offset] | 0xF0 // 上位4ビットは常に1
        } else {
            0xFF
        }
    }

    fn write_ram_mbc2(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }
        let offset = (addr as usize - 0xA000) & 0x01FF;
        if offset < self.ram.len() {
            self.ram[offset] = value & 0x0F; // 下位4ビットのみ
        }
    }

    // ===== MBC3 =====

    fn read_rom_mbc3(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                self.rom.get(addr as usize).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let bank = (self.rom_bank as usize) % self.header.rom_banks;
                let offset = bank * 0x4000 + (addr as usize - 0x4000);
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_rom_mbc3(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM/RTC有効
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            // ROMバンク番号 (7ビット, 0→1にリダイレクト)
            0x2000..=0x3FFF => {
                let bank = value & 0x7F;
                self.rom_bank = if bank == 0 { 1 } else { bank as u16 };
            }
            // RAMバンク番号 / RTCレジスタ選択
            0x4000..=0x5FFF => {
                self.ram_bank = value;
                self.rtc_mapped = value >= 0x08 && value <= 0x0C;
            }
            // RTCラッチ
            0x6000..=0x7FFF => {
                if value == 0x00 {
                    self.rtc_latch_pending = true;
                } else if value == 0x01 && self.rtc_latch_pending {
                    // ラッチ: 現在のRTC値をコピー
                    self.rtc_latched = self.rtc.clone();
                    self.rtc_latch_pending = false;
                } else {
                    self.rtc_latch_pending = false;
                }
            }
            _ => {}
        }
    }

    fn read_ram_mbc3(&self, addr: u16) -> u8 {
        if !self.ram_enabled {
            return 0xFF;
        }

        // RTCレジスタマッピング
        if self.rtc_mapped {
            return match self.ram_bank {
                0x08 => self.rtc_latched.seconds,
                0x09 => self.rtc_latched.minutes,
                0x0A => self.rtc_latched.hours,
                0x0B => self.rtc_latched.days_low,
                0x0C => self.rtc_latched.days_high,
                _ => 0xFF,
            };
        }

        // 通常RAM
        if self.ram.is_empty() {
            return 0xFF;
        }
        let bank = (self.ram_bank as usize) & 0x03;
        let offset = bank * 0x2000 + (addr as usize - 0xA000);
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_ram_mbc3(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled {
            return;
        }

        // RTCレジスタ書き込み
        if self.rtc_mapped {
            match self.ram_bank {
                0x08 => self.rtc.seconds = value & 0x3F,
                0x09 => self.rtc.minutes = value & 0x3F,
                0x0A => self.rtc.hours = value & 0x1F,
                0x0B => self.rtc.days_low = value,
                0x0C => self.rtc.days_high = value & 0xC1, // bit0,6,7のみ
                _ => {}
            }
            return;
        }

        // 通常RAM
        if self.ram.is_empty() {
            return;
        }
        let bank = (self.ram_bank as usize) & 0x03;
        let offset = bank * 0x2000 + (addr as usize - 0xA000);
        if offset < self.ram.len() {
            self.ram[offset] = value;
        }
    }

    // ===== MBC5 =====

    fn read_rom_mbc5(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => {
                self.rom.get(addr as usize).copied().unwrap_or(0xFF)
            }
            0x4000..=0x7FFF => {
                let bank = (self.rom_bank as usize) % self.header.rom_banks;
                let offset = bank * 0x4000 + (addr as usize - 0x4000);
                self.rom.get(offset).copied().unwrap_or(0xFF)
            }
            _ => 0xFF,
        }
    }

    fn write_rom_mbc5(&mut self, addr: u16, value: u8) {
        match addr {
            // RAM有効
            0x0000..=0x1FFF => {
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            // ROMバンク番号 下位8ビット
            0x2000..=0x2FFF => {
                self.rom_bank = (self.rom_bank & 0x100) | value as u16;
            }
            // ROMバンク番号 上位1ビット (bit8)
            0x3000..=0x3FFF => {
                self.rom_bank = (self.rom_bank & 0x0FF) | ((value as u16 & 0x01) << 8);
            }
            // RAMバンク番号 (0-15)
            0x4000..=0x5FFF => {
                self.ram_bank = value & 0x0F;
            }
            _ => {}
        }
    }

    fn read_ram_mbc5(&self, addr: u16) -> u8 {
        if !self.ram_enabled || self.ram.is_empty() {
            return 0xFF;
        }
        let bank = self.ram_bank as usize;
        let offset = bank * 0x2000 + (addr as usize - 0xA000);
        self.ram.get(offset).copied().unwrap_or(0xFF)
    }

    fn write_ram_mbc5(&mut self, addr: u16, value: u8) {
        if !self.ram_enabled || self.ram.is_empty() {
            return;
        }
        let bank = self.ram_bank as usize;
        let offset = bank * 0x2000 + (addr as usize - 0xA000);
        if offset < self.ram.len() {
            self.ram[offset] = value;
        }
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

    fn create_test_rom_with_ram(size: usize, cart_type: u8, rom_size_byte: u8, ram_size_byte: u8) -> Vec<u8> {
        let mut rom = create_test_rom(size, cart_type);
        rom[0x0148] = rom_size_byte;
        rom[0x0149] = ram_size_byte;
        rom
    }

    // ===== ROM ONLY テスト =====

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

    // ===== MBC1 テスト =====

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

    // ===== MBC2 テスト =====

    #[test]
    fn test_mbc2_rom_bank_switching() {
        let mut rom = create_test_rom(0x10000, 0x05); // MBC2
        rom[0x0148] = 0x01; // 64KB

        rom[0x4000] = 0x11; // Bank 1
        rom[0x8000] = 0x22; // Bank 2
        rom[0xC000] = 0x33; // Bank 3

        let mut cart = Cartridge::new(rom).unwrap();

        // デフォルトはバンク1
        assert_eq!(cart.read_rom(0x4000), 0x11);

        // バンク切り替え (bit8=1 のアドレス)
        cart.write_rom(0x2100, 0x02);
        assert_eq!(cart.read_rom(0x4000), 0x22);

        cart.write_rom(0x2100, 0x03);
        assert_eq!(cart.read_rom(0x4000), 0x33);
    }

    #[test]
    fn test_mbc2_bank0_redirect() {
        let rom = create_test_rom(0x8000, 0x05);
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x2100, 0x00); // バンク0
        assert_eq!(cart.rom_bank, 1); // バンク1にリダイレクト
    }

    #[test]
    fn test_mbc2_ram() {
        let rom = create_test_rom(0x8000, 0x05);
        let mut cart = Cartridge::new(rom).unwrap();

        // MBC2 RAMは512×4ビット
        assert_eq!(cart.ram.len(), 512);

        // RAM有効化 (bit8=0のアドレス)
        cart.write_rom(0x0000, 0x0A);
        assert!(cart.ram_enabled);

        // 下位4ビットのみ書き込み可能
        cart.write_ram(0xA000, 0xFF);
        assert_eq!(cart.read_ram(0xA000) & 0x0F, 0x0F);
        // 上位4ビットは読み取り時に1
        assert_eq!(cart.read_ram(0xA000), 0xFF);

        // 4ビットデータ確認
        cart.write_ram(0xA001, 0x35);
        assert_eq!(cart.read_ram(0xA001) & 0x0F, 0x05); // 下位4ビットのみ
    }

    #[test]
    fn test_mbc2_ram_enable_address_bit8() {
        let rom = create_test_rom(0x8000, 0x05);
        let mut cart = Cartridge::new(rom).unwrap();

        // bit8=1 のアドレス → ROMバンク番号
        cart.write_rom(0x0100, 0x0A);
        assert!(!cart.ram_enabled); // RAM有効化されない

        // bit8=0 のアドレス → RAM有効化
        cart.write_rom(0x0000, 0x0A);
        assert!(cart.ram_enabled);
    }

    #[test]
    fn test_mbc2_ram_wrapping() {
        let rom = create_test_rom(0x8000, 0x05);
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x0000, 0x0A); // RAM有効化

        // 0xA200 は 0xA000と同じオフセット（下位9ビットでラップ）
        cart.write_ram(0xA000, 0x07);
        assert_eq!(cart.read_ram(0xA200) & 0x0F, 0x07); // 同じアドレスにラップ
    }

    // ===== MBC3 テスト =====

    #[test]
    fn test_mbc3_rom_bank_switching() {
        let mut rom = create_test_rom_with_ram(0x20000, 0x13, 0x02, 0x03); // MBC3+RAM+BATTERY, 128KB ROM, 32KB RAM

        rom[0x4000] = 0xAA; // Bank 1
        rom[0x8000] = 0xBB; // Bank 2
        rom[0x1C000] = 0xCC; // Bank 7

        let mut cart = Cartridge::new(rom).unwrap();

        assert_eq!(cart.read_rom(0x4000), 0xAA);

        cart.write_rom(0x2000, 0x02);
        assert_eq!(cart.read_rom(0x4000), 0xBB);

        cart.write_rom(0x2000, 0x07);
        assert_eq!(cart.read_rom(0x4000), 0xCC);
    }

    #[test]
    fn test_mbc3_bank0_redirect() {
        let rom = create_test_rom(0x8000, 0x11);
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x2000, 0x00);
        assert_eq!(cart.rom_bank, 1);
    }

    #[test]
    fn test_mbc3_ram_banking() {
        let rom = create_test_rom_with_ram(0x8000, 0x13, 0x00, 0x03); // 32KB RAM (4バンク)
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x0000, 0x0A); // RAM有効化

        // バンク0に書き込み
        cart.write_rom(0x4000, 0x00);
        cart.write_ram(0xA000, 0x11);

        // バンク1に書き込み
        cart.write_rom(0x4000, 0x01);
        cart.write_ram(0xA000, 0x22);

        // バンク0の読み取り
        cart.write_rom(0x4000, 0x00);
        assert_eq!(cart.read_ram(0xA000), 0x11);

        // バンク1の読み取り
        cart.write_rom(0x4000, 0x01);
        assert_eq!(cart.read_ram(0xA000), 0x22);
    }

    #[test]
    fn test_mbc3_rtc_latch() {
        let rom = create_test_rom(0x8000, 0x0F); // MBC3+TIMER+BATTERY
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x0000, 0x0A); // RAM/RTC有効化

        // RTCに値をセット
        cart.write_rom(0x4000, 0x08); // 秒レジスタ選択
        cart.write_ram(0xA000, 30); // 30秒

        cart.write_rom(0x4000, 0x09); // 分レジスタ
        cart.write_ram(0xA000, 45); // 45分

        // ラッチ実行 (0x00 → 0x01)
        cart.write_rom(0x6000, 0x00);
        cart.write_rom(0x6000, 0x01);

        // ラッチされた値を読み取り
        cart.write_rom(0x4000, 0x08);
        assert_eq!(cart.read_ram(0xA000), 30);

        cart.write_rom(0x4000, 0x09);
        assert_eq!(cart.read_ram(0xA000), 45);
    }

    #[test]
    fn test_mbc3_rtc_tick() {
        let rom = create_test_rom(0x8000, 0x0F); // MBC3+TIMER+BATTERY
        let mut cart = Cartridge::new(rom).unwrap();

        // 1秒分のサイクルを進める
        for _ in 0..CYCLES_PER_SECOND {
            cart.tick();
        }

        // ラッチしてRTC値を確認
        cart.write_rom(0x0000, 0x0A);
        cart.write_rom(0x6000, 0x00);
        cart.write_rom(0x6000, 0x01);

        cart.write_rom(0x4000, 0x08);
        assert_eq!(cart.read_ram(0xA000), 1); // 1秒経過
    }

    #[test]
    fn test_mbc3_rtc_halt() {
        let rom = create_test_rom(0x8000, 0x0F);
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x0000, 0x0A);

        // RTCを停止
        cart.write_rom(0x4000, 0x0C); // days_high選択
        cart.write_ram(0xA000, 0x40); // bit6=1: 停止

        // サイクルを進めてもRTCは変化しない
        for _ in 0..CYCLES_PER_SECOND * 2 {
            cart.tick();
        }

        // ラッチして確認
        cart.write_rom(0x6000, 0x00);
        cart.write_rom(0x6000, 0x01);

        cart.write_rom(0x4000, 0x08);
        assert_eq!(cart.read_ram(0xA000), 0); // 0秒のまま
    }

    // ===== MBC5 テスト =====

    #[test]
    fn test_mbc5_rom_bank_switching() {
        let mut rom = create_test_rom_with_ram(0x20000, 0x19, 0x02, 0x00); // MBC5, 128KB ROM

        rom[0x4000] = 0x11; // Bank 1
        rom[0x8000] = 0x22; // Bank 2
        rom[0xC000] = 0x33; // Bank 3

        let mut cart = Cartridge::new(rom).unwrap();

        assert_eq!(cart.read_rom(0x4000), 0x11);

        cart.write_rom(0x2000, 0x02);
        assert_eq!(cart.read_rom(0x4000), 0x22);

        cart.write_rom(0x2000, 0x03);
        assert_eq!(cart.read_rom(0x4000), 0x33);
    }

    #[test]
    fn test_mbc5_bank0_allowed() {
        let mut rom = create_test_rom(0x10000, 0x19);
        rom[0x0148] = 0x01; // 64KB
        rom[0x0000] = 0xAA; // Bank 0のデータ

        let mut cart = Cartridge::new(rom).unwrap();

        // MBC5ではバンク0を選択可能（MBC1と異なる）
        cart.write_rom(0x2000, 0x00);
        assert_eq!(cart.rom_bank, 0); // バンク0のまま
        // Bank 0のデータ（0x0000-0x3FFFと同じ）
        assert_eq!(cart.read_rom(0x4000), cart.read_rom(0x0000));
    }

    #[test]
    fn test_mbc5_9bit_rom_bank() {
        let rom = create_test_rom_with_ram(0x80000, 0x19, 0x04, 0x00); // MBC5, 512KB ROM (32バンク)

        // バンク番号の上位ビットテスト
        let mut cart = Cartridge::new(rom).unwrap();

        // 下位8ビット
        cart.write_rom(0x2000, 0xFF);
        assert_eq!(cart.rom_bank & 0xFF, 0xFF);

        // 上位1ビット
        cart.write_rom(0x3000, 0x01);
        assert_eq!(cart.rom_bank, 0x1FF); // 9ビット
    }

    #[test]
    fn test_mbc5_ram() {
        let rom = create_test_rom_with_ram(0x8000, 0x1A, 0x00, 0x02); // MBC5+RAM, 8KB RAM
        let mut cart = Cartridge::new(rom).unwrap();

        cart.write_rom(0x0000, 0x0A); // RAM有効化
        cart.write_ram(0xA000, 0x42);
        assert_eq!(cart.read_ram(0xA000), 0x42);
    }

    #[test]
    fn test_mbc5_ram_banking() {
        let rom = create_test_rom_with_ram(0x8000, 0x1A, 0x00, 0x03); // 32KB RAM (4バンク)
        let mut cart = Cartridge::new(rom).unwrap();
        cart.write_rom(0x0000, 0x0A);

        // バンク0
        cart.write_rom(0x4000, 0x00);
        cart.write_ram(0xA000, 0x11);

        // バンク1
        cart.write_rom(0x4000, 0x01);
        cart.write_ram(0xA000, 0x22);

        // バンク0の読み取り
        cart.write_rom(0x4000, 0x00);
        assert_eq!(cart.read_ram(0xA000), 0x11);

        // バンク1の読み取り
        cart.write_rom(0x4000, 0x01);
        assert_eq!(cart.read_ram(0xA000), 0x22);
    }

    // ===== カートリッジタイプ検出テスト =====

    #[test]
    fn test_cartridge_type_detection() {
        assert_eq!(CartridgeType::from_byte(0x00).mbc_kind(), MbcKind::None);
        assert_eq!(CartridgeType::from_byte(0x01).mbc_kind(), MbcKind::Mbc1);
        assert_eq!(CartridgeType::from_byte(0x05).mbc_kind(), MbcKind::Mbc2);
        assert_eq!(CartridgeType::from_byte(0x13).mbc_kind(), MbcKind::Mbc3);
        assert_eq!(CartridgeType::from_byte(0x19).mbc_kind(), MbcKind::Mbc5);
    }

    #[test]
    fn test_cartridge_has_timer() {
        assert!(CartridgeType::Mbc3TimerBattery.has_timer());
        assert!(CartridgeType::Mbc3TimerRamBattery.has_timer());
        assert!(!CartridgeType::Mbc3.has_timer());
        assert!(!CartridgeType::Mbc1.has_timer());
    }
}
