// src/apu/noise.rs
// GameBoy APU ノイズチャンネル (Channel 4)
//
// NR41 (0xFF20): 長さ
//   Bit 5-0: 長さデータ (t1: 0-63)
// NR42 (0xFF21): エンベロープ
//   Bit 7-4: 初期ボリューム
//   Bit 3:   方向 (0=減少, 1=増加)
//   Bit 2-0: 周期
// NR43 (0xFF22): 多項式カウンタ
//   Bit 7-4: シフトクロック周波数 (s)
//   Bit 3:   カウンタ幅 (0=15ビット, 1=7ビット)
//   Bit 2-0: 分周比 (r)
//   周波数 = 524288 Hz / r / 2^(s+1)  (r=0は0.5として扱う)
// NR44 (0xFF23): 制御
//   Bit 7:   トリガー
//   Bit 6:   長さ有効

/// ノイズチャンネル
pub struct NoiseChannel {
    /// チャンネル有効フラグ
    pub enabled: bool,
    /// DAC有効フラグ
    pub dac_enabled: bool,

    /// 長さカウンタ
    pub length_counter: u16,
    /// 長さ有効フラグ
    pub length_enabled: bool,

    // エンベロープ
    /// エンベロープ初期ボリューム
    pub envelope_initial: u8,
    /// エンベロープ方向
    pub envelope_direction: bool,
    /// エンベロープ周期
    pub envelope_period: u8,
    /// 現在のボリューム
    pub volume: u8,
    /// エンベロープタイマー
    envelope_timer: u8,

    // 多項式カウンタ
    /// シフトクロック
    pub clock_shift: u8,
    /// カウンタ幅 (true=7ビット, false=15ビット)
    pub width_mode: bool,
    /// 分周比
    pub divisor_code: u8,

    /// LFSR (線形フィードバックシフトレジスタ)
    lfsr: u16,
    /// 周波数タイマー
    frequency_timer: u16,
}

/// 分周比テーブル
const DIVISOR_TABLE: [u16; 8] = [8, 16, 32, 48, 64, 80, 96, 112];

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            length_counter: 0,
            length_enabled: false,
            envelope_initial: 0,
            envelope_direction: false,
            envelope_period: 0,
            volume: 0,
            envelope_timer: 0,
            clock_shift: 0,
            width_mode: false,
            divisor_code: 0,
            lfsr: 0x7FFF, // 15ビット全て1で初期化
            frequency_timer: 0,
        }
    }

    /// NR41 長さレジスタへの書き込み (書き込みのみ)
    pub fn write_length(&mut self, value: u8) {
        let length_data = value & 0x3F;
        self.length_counter = 64 - length_data as u16;
    }

    /// NR42 エンベロープレジスタの読み取り
    pub fn read_envelope(&self) -> u8 {
        (self.envelope_initial << 4)
            | if self.envelope_direction { 0x08 } else { 0x00 }
            | self.envelope_period
    }

    /// NR42 エンベロープレジスタへの書き込み
    pub fn write_envelope(&mut self, value: u8) {
        self.envelope_initial = (value >> 4) & 0x0F;
        self.envelope_direction = value & 0x08 != 0;
        self.envelope_period = value & 0x07;
        self.dac_enabled = value & 0xF8 != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    /// NR43 多項式カウンタレジスタの読み取り
    pub fn read_polynomial(&self) -> u8 {
        (self.clock_shift << 4)
            | if self.width_mode { 0x08 } else { 0x00 }
            | self.divisor_code
    }

    /// NR43 多項式カウンタレジスタへの書き込み
    pub fn write_polynomial(&mut self, value: u8) {
        self.clock_shift = (value >> 4) & 0x0F;
        self.width_mode = value & 0x08 != 0;
        self.divisor_code = value & 0x07;
    }

    /// NR44 制御レジスタの読み取り
    pub fn read_control(&self) -> u8 {
        0xBF | if self.length_enabled { 0x40 } else { 0x00 }
    }

    /// NR44 制御レジスタへの書き込み
    pub fn write_control(&mut self, value: u8) {
        self.length_enabled = value & 0x40 != 0;

        if value & 0x80 != 0 {
            self.trigger();
        }
    }

    /// チャンネルトリガー
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;

        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        // 周波数タイマーリロード
        self.frequency_timer = self.get_period();

        // LFSR初期化
        self.lfsr = 0x7FFF;

        // エンベロープリロード
        self.volume = self.envelope_initial;
        self.envelope_timer = if self.envelope_period == 0 { 8 } else { self.envelope_period };
    }

    /// 周波数タイマー周期を計算
    fn get_period(&self) -> u16 {
        DIVISOR_TABLE[self.divisor_code as usize] << self.clock_shift
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

    /// エンベロープをクロック
    pub fn clock_envelope(&mut self) {
        if self.envelope_period == 0 {
            return;
        }

        self.envelope_timer = self.envelope_timer.saturating_sub(1);
        if self.envelope_timer == 0 {
            self.envelope_timer = if self.envelope_period == 0 { 8 } else { self.envelope_period };

            if self.envelope_direction && self.volume < 15 {
                self.volume += 1;
            } else if !self.envelope_direction && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }

    /// 周波数タイマーを1サイクル進める
    pub fn tick(&mut self) {
        if self.frequency_timer > 0 {
            self.frequency_timer -= 1;
        }

        if self.frequency_timer == 0 {
            self.frequency_timer = self.get_period();

            // LFSRクロック
            let xor_result = (self.lfsr & 0x01) ^ ((self.lfsr >> 1) & 0x01);
            self.lfsr = (self.lfsr >> 1) | (xor_result << 14);

            // 7ビットモードではbit6にもセット
            if self.width_mode {
                self.lfsr = (self.lfsr & !0x0040) | (xor_result << 6);
            }
        }
    }

    /// 現在の出力サンプル (0-15)
    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        // LFSRのbit0が0ならHigh出力
        if self.lfsr & 0x01 == 0 {
            self.volume
        } else {
            0
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
    fn test_noise_channel_creation() {
        let ch = NoiseChannel::new();
        assert!(!ch.enabled);
        assert!(!ch.dac_enabled);
        assert_eq!(ch.lfsr, 0x7FFF);
        assert_eq!(ch.output(), 0);
    }

    #[test]
    fn test_noise_length() {
        let mut ch = NoiseChannel::new();
        ch.write_length(62); // counter = 64 - 62 = 2
        assert_eq!(ch.length_counter, 2);
    }

    #[test]
    fn test_noise_envelope_register() {
        let mut ch = NoiseChannel::new();
        ch.write_envelope(0xA5); // vol=10, down, period=5
        assert_eq!(ch.envelope_initial, 10);
        assert!(!ch.envelope_direction);
        assert_eq!(ch.envelope_period, 5);
        assert!(ch.dac_enabled);
        assert_eq!(ch.read_envelope(), 0xA5);
    }

    #[test]
    fn test_noise_polynomial_register() {
        let mut ch = NoiseChannel::new();
        ch.write_polynomial(0x63); // shift=6, 15bit, divisor=3
        assert_eq!(ch.clock_shift, 6);
        assert!(!ch.width_mode);
        assert_eq!(ch.divisor_code, 3);
        assert_eq!(ch.read_polynomial(), 0x63);
    }

    #[test]
    fn test_noise_width_mode() {
        let mut ch = NoiseChannel::new();
        ch.write_polynomial(0x08); // 7ビットモード
        assert!(ch.width_mode);
    }

    #[test]
    fn test_noise_trigger() {
        let mut ch = NoiseChannel::new();
        ch.write_envelope(0xF0); // DAC有効
        ch.write_control(0x80); // トリガー

        assert!(ch.enabled);
        assert_eq!(ch.volume, 15);
        assert_eq!(ch.lfsr, 0x7FFF);
    }

    #[test]
    fn test_noise_length_counter() {
        let mut ch = NoiseChannel::new();
        ch.write_envelope(0xF0);
        ch.write_length(63); // counter = 1
        ch.write_control(0xC0); // トリガー + 長さ有効

        assert!(ch.enabled);
        ch.clock_length();
        assert!(!ch.enabled);
    }

    #[test]
    fn test_noise_lfsr_shift() {
        let mut ch = NoiseChannel::new();
        ch.write_envelope(0xF0);
        ch.write_polynomial(0x00); // shift=0, 15bit, divisor=0
        ch.write_control(0x80); // トリガー

        let initial_lfsr = ch.lfsr;

        // LFSRを数回クロック
        let period = 8u16; // divisor_code=0 → 8サイクル
        for _ in 0..period {
            ch.tick();
        }

        // LFSRが変化しているはず
        assert_ne!(ch.lfsr, initial_lfsr);
    }

    #[test]
    fn test_noise_disabled_output() {
        let ch = NoiseChannel::new();
        assert_eq!(ch.output(), 0);
        assert_eq!(ch.dac_output(), 0.0);
    }
}
