// src/apu/wave.rs
// GameBoy APU ウェーブチャンネル (Channel 3)
//
// NR30 (0xFF1A): チャンネルオン/オフ
//   Bit 7: DAC電源 (1=オン)
// NR31 (0xFF1B): 長さ
//   Bit 7-0: 長さデータ (0-255)
// NR32 (0xFF1C): 出力レベル
//   Bit 6-5: 出力レベル (0=無音, 1=100%, 2=50%, 3=25%)
// NR33 (0xFF1D): 周波数下位
// NR34 (0xFF1E): 周波数上位/制御
//
// Wave RAM (0xFF30-0xFF3F): 16バイト = 32サンプル (各4ビット)

/// ウェーブチャンネル
pub struct WaveChannel {
    /// チャンネル有効フラグ
    pub enabled: bool,
    /// DAC有効フラグ
    pub dac_enabled: bool,

    /// 長さカウンタ
    pub length_counter: u16,
    /// 長さ有効フラグ
    pub length_enabled: bool,

    /// 出力レベル (0-3)
    pub output_level: u8,

    /// 周波数値 (11ビット)
    pub frequency: u16,
    /// 周波数タイマー
    frequency_timer: u16,

    /// Wave RAM (16バイト = 32サンプル)
    pub wave_ram: [u8; 16],
    /// 現在のサンプル位置 (0-31)
    sample_position: u8,
    /// 現在のサンプルバッファ
    sample_buffer: u8,
}

impl WaveChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            length_counter: 0,
            length_enabled: false,
            output_level: 0,
            frequency: 0,
            frequency_timer: 0,
            wave_ram: [0; 16],
            sample_position: 0,
            sample_buffer: 0,
        }
    }

    /// NR30 DAC電源レジスタの読み取り
    pub fn read_dac(&self) -> u8 {
        0x7F | if self.dac_enabled { 0x80 } else { 0x00 }
    }

    /// NR30 DAC電源レジスタへの書き込み
    pub fn write_dac(&mut self, value: u8) {
        self.dac_enabled = value & 0x80 != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    /// NR31 長さレジスタへの書き込み (書き込みのみ)
    pub fn write_length(&mut self, value: u8) {
        self.length_counter = 256 - value as u16;
    }

    /// NR32 出力レベルレジスタの読み取り
    pub fn read_output_level(&self) -> u8 {
        0x9F | (self.output_level << 5)
    }

    /// NR32 出力レベルレジスタへの書き込み
    pub fn write_output_level(&mut self, value: u8) {
        self.output_level = (value >> 5) & 0x03;
    }

    /// NR33 周波数下位レジスタへの書き込み (書き込みのみ)
    pub fn write_frequency_low(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x700) | value as u16;
    }

    /// NR34 周波数上位/制御レジスタの読み取り
    pub fn read_frequency_high(&self) -> u8 {
        0xBF | if self.length_enabled { 0x40 } else { 0x00 }
    }

    /// NR34 周波数上位/制御レジスタへの書き込み
    pub fn write_frequency_high(&mut self, value: u8) {
        self.length_enabled = value & 0x40 != 0;
        self.frequency = (self.frequency & 0x00FF) | ((value as u16 & 0x07) << 8);

        if value & 0x80 != 0 {
            self.trigger();
        }
    }

    /// Wave RAMの読み取り
    pub fn read_wave_ram(&self, addr: u16) -> u8 {
        let index = (addr - 0xFF30) as usize;
        if index < 16 {
            self.wave_ram[index]
        } else {
            0xFF
        }
    }

    /// Wave RAMへの書き込み
    pub fn write_wave_ram(&mut self, addr: u16, value: u8) {
        let index = (addr - 0xFF30) as usize;
        if index < 16 {
            self.wave_ram[index] = value;
        }
    }

    /// チャンネルトリガー
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;

        if self.length_counter == 0 {
            self.length_counter = 256;
        }

        // 周波数タイマーリロード
        self.frequency_timer = (2048 - self.frequency) * 2;
        self.sample_position = 0;
    }

    /// 長さカウンタをクロック
    pub fn clock_length(&mut self) {
        if self.length_enabled && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.enabled = false;
            }
        }
    }

    /// 周波数タイマーを1サイクル進める
    pub fn tick(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }

        if self.frequency_timer == 0 {
            self.frequency_timer = (2048 - self.frequency) * 2;
            self.sample_position = (self.sample_position + 1) & 0x1F;

            // Wave RAMから現在のサンプルを読み出し
            let byte_index = (self.sample_position / 2) as usize;
            if self.sample_position & 1 == 0 {
                // 上位ニブル
                self.sample_buffer = (self.wave_ram[byte_index] >> 4) & 0x0F;
            } else {
                // 下位ニブル
                self.sample_buffer = self.wave_ram[byte_index] & 0x0F;
            }
        }
    }

    /// 現在の出力サンプル (0-15)
    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }

        let sample = self.sample_buffer;
        match self.output_level {
            0 => 0,                  // 無音
            1 => sample,             // 100%
            2 => sample >> 1,        // 50%
            3 => sample >> 2,        // 25%
            _ => 0,
        }
    }

    /// DAC出力 (-1.0 ~ 1.0)
    pub fn dac_output(&self) -> f32 {
        if !self.dac_enabled {
            return 0.0;
        }
        let digital = self.output();
        (digital as f32 / 7.5) - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_channel_creation() {
        let ch = WaveChannel::new();
        assert!(!ch.enabled);
        assert!(!ch.dac_enabled);
        assert_eq!(ch.output(), 0);
        assert_eq!(ch.wave_ram, [0; 16]);
    }

    #[test]
    fn test_wave_dac_register() {
        let mut ch = WaveChannel::new();
        ch.write_dac(0x80);
        assert!(ch.dac_enabled);
        assert_eq!(ch.read_dac(), 0xFF);

        ch.write_dac(0x00);
        assert!(!ch.dac_enabled);
        assert_eq!(ch.read_dac(), 0x7F);
    }

    #[test]
    fn test_wave_output_level() {
        let mut ch = WaveChannel::new();
        ch.write_output_level(0x40); // level=2 (50%)
        assert_eq!(ch.output_level, 2);
        assert_eq!(ch.read_output_level() & 0x60, 0x40);
    }

    #[test]
    fn test_wave_ram_read_write() {
        let mut ch = WaveChannel::new();
        ch.write_wave_ram(0xFF30, 0x12);
        ch.write_wave_ram(0xFF31, 0x34);
        ch.write_wave_ram(0xFF3F, 0xAB);

        assert_eq!(ch.read_wave_ram(0xFF30), 0x12);
        assert_eq!(ch.read_wave_ram(0xFF31), 0x34);
        assert_eq!(ch.read_wave_ram(0xFF3F), 0xAB);
    }

    #[test]
    fn test_wave_trigger() {
        let mut ch = WaveChannel::new();
        ch.write_dac(0x80); // DAC有効
        ch.write_frequency_low(0x00);
        ch.write_frequency_high(0x80); // トリガー

        assert!(ch.enabled);
        assert_eq!(ch.sample_position, 0);
    }

    #[test]
    fn test_wave_length_counter() {
        let mut ch = WaveChannel::new();
        ch.write_dac(0x80);
        ch.write_length(254); // counter = 256 - 254 = 2
        ch.write_frequency_high(0xC0); // トリガー + 長さ有効

        assert!(ch.enabled);
        ch.clock_length();
        assert!(ch.enabled);
        ch.clock_length();
        assert!(!ch.enabled);
    }

    #[test]
    fn test_wave_output_levels() {
        let mut ch = WaveChannel::new();
        ch.write_dac(0x80);
        ch.enabled = true;
        ch.sample_buffer = 0x0C; // サンプル値12

        ch.write_output_level(0x20); // 100%
        assert_eq!(ch.output(), 12);

        ch.write_output_level(0x40); // 50%
        assert_eq!(ch.output(), 6);

        ch.write_output_level(0x60); // 25%
        assert_eq!(ch.output(), 3);

        ch.write_output_level(0x00); // 無音
        assert_eq!(ch.output(), 0);
    }

    #[test]
    fn test_wave_disabled_output() {
        let ch = WaveChannel::new();
        assert_eq!(ch.output(), 0);
        assert_eq!(ch.dac_output(), 0.0);
    }
}
