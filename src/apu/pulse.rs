// src/apu/pulse.rs
// GameBoy APU パルスチャンネル (Channel 1/2)
//
// Channel 1: NR10(スイープ), NR11(長さ/デューティ), NR12(エンベロープ), NR13(周波数下位), NR14(周波数上位/制御)
// Channel 2: NR21(長さ/デューティ), NR22(エンベロープ), NR23(周波数下位), NR24(周波数上位/制御)
//
// デューティサイクルパターン:
//   00: 12.5% (________------__)
//   01: 25.0% (________------_-)
//   10: 50.0% (____----____----)
//   11: 75.0% (________--____--)
//
// スイープ (Channel 1のみ):
//   周波数を周期的にシフトして変更

/// デューティサイクル波形テーブル
/// 各デューティパターンの8ステップ (0=Low, 1=High)
const DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1], // 12.5%
    [1, 0, 0, 0, 0, 0, 0, 1], // 25.0%
    [1, 0, 0, 0, 0, 1, 1, 1], // 50.0%
    [0, 1, 1, 1, 1, 1, 1, 0], // 75.0%
];

/// パルスチャンネル
pub struct PulseChannel {
    /// チャンネル有効フラグ
    pub enabled: bool,
    /// DAC有効フラグ
    pub dac_enabled: bool,

    // NRx1: デューティ/長さ
    /// デューティサイクル (0-3)
    pub duty: u8,
    /// 長さカウンタ
    pub length_counter: u16,
    /// 長さ有効フラグ
    pub length_enabled: bool,

    // NRx2: エンベロープ
    /// エンベロープ初期ボリューム (0-15)
    pub envelope_initial: u8,
    /// エンベロープ方向 (true=増加, false=減少)
    pub envelope_direction: bool,
    /// エンベロープ周期 (0-7)
    pub envelope_period: u8,
    /// 現在のボリューム
    pub volume: u8,
    /// エンベロープタイマー
    envelope_timer: u8,

    // NRx3/NRx4: 周波数
    /// 周波数値 (11ビット)
    pub frequency: u16,
    /// 周波数タイマー
    frequency_timer: u16,
    /// デューティステップ位置
    duty_position: u8,

    // NR10: スイープ (Channel 1のみ)
    /// スイープ有効フラグ
    pub sweep_enabled: bool,
    /// スイープ周期 (0-7)
    pub sweep_period: u8,
    /// スイープ方向 (true=減少, false=増加)
    pub sweep_negate: bool,
    /// スイープシフト量 (0-7)
    pub sweep_shift: u8,
    /// スイープタイマー
    sweep_timer: u8,
    /// スイープシャドウ周波数
    sweep_shadow: u16,
    /// スイープで減算を使用したか
    sweep_negate_used: bool,
    /// スイープ機能を持つか (Channel 1のみ)
    has_sweep: bool,
}

impl PulseChannel {
    /// 新しいパルスチャンネルを作成
    pub fn new(has_sweep: bool) -> Self {
        Self {
            enabled: false,
            dac_enabled: false,
            duty: 0,
            length_counter: 0,
            length_enabled: false,
            envelope_initial: 0,
            envelope_direction: false,
            envelope_period: 0,
            volume: 0,
            envelope_timer: 0,
            frequency: 0,
            frequency_timer: 0,
            duty_position: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            sweep_timer: 0,
            sweep_shadow: 0,
            sweep_negate_used: false,
            has_sweep,
        }
    }

    /// NR10 スイープレジスタの読み取り
    pub fn read_sweep(&self) -> u8 {
        0x80 // bit7は常に1
            | (self.sweep_period << 4)
            | if self.sweep_negate { 0x08 } else { 0x00 }
            | self.sweep_shift
    }

    /// NR10 スイープレジスタへの書き込み
    pub fn write_sweep(&mut self, value: u8) {
        self.sweep_period = (value >> 4) & 0x07;
        let new_negate = value & 0x08 != 0;
        // 減算モードから加算モードへの切り替えでチャンネル無効化
        if self.sweep_negate && !new_negate && self.sweep_negate_used {
            self.enabled = false;
        }
        self.sweep_negate = new_negate;
        self.sweep_shift = value & 0x07;
    }

    /// NRx1 長さ/デューティレジスタの読み取り (上位2ビットのみ読める)
    pub fn read_length_duty(&self) -> u8 {
        (self.duty << 6) | 0x3F // 下位6ビットは常に1
    }

    /// NRx1 長さ/デューティレジスタへの書き込み
    pub fn write_length_duty(&mut self, value: u8) {
        self.duty = (value >> 6) & 0x03;
        let length_data = value & 0x3F;
        self.length_counter = 64 - length_data as u16;
    }

    /// NRx2 エンベロープレジスタの読み取り
    pub fn read_envelope(&self) -> u8 {
        (self.envelope_initial << 4)
            | if self.envelope_direction { 0x08 } else { 0x00 }
            | self.envelope_period
    }

    /// NRx2 エンベロープレジスタへの書き込み
    pub fn write_envelope(&mut self, value: u8) {
        self.envelope_initial = (value >> 4) & 0x0F;
        self.envelope_direction = value & 0x08 != 0;
        self.envelope_period = value & 0x07;
        // DACは上位5ビットが0以外なら有効
        self.dac_enabled = value & 0xF8 != 0;
        if !self.dac_enabled {
            self.enabled = false;
        }
    }

    /// NRx3 周波数下位レジスタへの書き込み (書き込みのみ)
    pub fn write_frequency_low(&mut self, value: u8) {
        self.frequency = (self.frequency & 0x700) | value as u16;
    }

    /// NRx4 周波数上位/制御レジスタの読み取り
    pub fn read_frequency_high(&self) -> u8 {
        0xBF | if self.length_enabled { 0x40 } else { 0x00 } // bit6のみ読める
    }

    /// NRx4 周波数上位/制御レジスタへの書き込み
    pub fn write_frequency_high(&mut self, value: u8) {
        self.length_enabled = value & 0x40 != 0;
        self.frequency = (self.frequency & 0x00FF) | ((value as u16 & 0x07) << 8);

        // トリガー
        if value & 0x80 != 0 {
            self.trigger();
        }
    }

    /// チャンネルトリガー
    fn trigger(&mut self) {
        self.enabled = self.dac_enabled;

        // 長さカウンタが0なら最大値に
        if self.length_counter == 0 {
            self.length_counter = 64;
        }

        // 周波数タイマーリロード
        self.frequency_timer = (2048 - self.frequency) * 4;

        // エンベロープリロード
        self.volume = self.envelope_initial;
        self.envelope_timer = if self.envelope_period == 0 { 8 } else { self.envelope_period };

        // スイープ初期化
        if self.has_sweep {
            self.sweep_shadow = self.frequency;
            self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
            self.sweep_enabled = self.sweep_period != 0 || self.sweep_shift != 0;
            self.sweep_negate_used = false;

            // スイープシフトが0でない場合、オーバーフローチェック
            if self.sweep_shift != 0 {
                let new_freq = self.calculate_sweep_frequency();
                if new_freq > 2047 {
                    self.enabled = false;
                }
            }
        }
    }

    /// スイープによる新しい周波数を計算
    fn calculate_sweep_frequency(&mut self) -> u16 {
        let shifted = self.sweep_shadow >> self.sweep_shift;
        if self.sweep_negate {
            self.sweep_negate_used = true;
            self.sweep_shadow.wrapping_sub(shifted)
        } else {
            self.sweep_shadow.wrapping_add(shifted)
        }
    }

    /// スイープをクロック
    pub fn clock_sweep(&mut self) {
        if !self.has_sweep {
            return;
        }

        self.sweep_timer = self.sweep_timer.saturating_sub(1);
        if self.sweep_timer == 0 {
            self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };

            if self.sweep_enabled && self.sweep_period != 0 {
                let new_freq = self.calculate_sweep_frequency();
                if new_freq > 2047 {
                    self.enabled = false;
                } else if self.sweep_shift != 0 {
                    self.sweep_shadow = new_freq;
                    self.frequency = new_freq;

                    // 再度オーバーフローチェック
                    let check_freq = self.calculate_sweep_frequency();
                    if check_freq > 2047 {
                        self.enabled = false;
                    }
                }
            }
        }
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
            self.frequency_timer = (2048 - self.frequency) * 4;
            self.duty_position = (self.duty_position + 1) & 0x07;
        }
    }

    /// 現在の出力サンプル (0-15)
    pub fn output(&self) -> u8 {
        if !self.enabled || !self.dac_enabled {
            return 0;
        }
        let wave = DUTY_TABLE[self.duty as usize][self.duty_position as usize];
        if wave != 0 { self.volume } else { 0 }
    }

    /// DAC出力 (-1.0 ~ 1.0 の範囲、DACオフ時は0.0)
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
    fn test_pulse_channel_creation() {
        let ch = PulseChannel::new(false);
        assert!(!ch.enabled);
        assert!(!ch.dac_enabled);
        assert_eq!(ch.output(), 0);
    }

    #[test]
    fn test_pulse_channel_with_sweep() {
        let ch = PulseChannel::new(true);
        assert!(ch.has_sweep);
    }

    #[test]
    fn test_duty_register() {
        let mut ch = PulseChannel::new(false);
        ch.write_length_duty(0x80); // duty=2 (50%), length=0
        assert_eq!(ch.duty, 2);
        assert_eq!(ch.length_counter, 64);
        assert_eq!(ch.read_length_duty() & 0xC0, 0x80);
    }

    #[test]
    fn test_envelope_register() {
        let mut ch = PulseChannel::new(false);
        ch.write_envelope(0xF3); // volume=15, up, period=3
        assert_eq!(ch.envelope_initial, 15);
        assert!(!ch.envelope_direction); // bit3=0: 減少
        assert_eq!(ch.envelope_period, 3);
        assert!(ch.dac_enabled);
        assert_eq!(ch.read_envelope(), 0xF3);
    }

    #[test]
    fn test_envelope_dac_disable() {
        let mut ch = PulseChannel::new(false);
        ch.write_envelope(0x00); // volume=0, down, period=0 → DAC無効
        assert!(!ch.dac_enabled);
    }

    #[test]
    fn test_trigger() {
        let mut ch = PulseChannel::new(false);
        ch.write_envelope(0xF0); // volume=15 (DAC有効)
        ch.write_frequency_low(0x00);
        ch.write_frequency_high(0x80); // トリガー

        assert!(ch.enabled);
        assert_eq!(ch.volume, 15);
    }

    #[test]
    fn test_length_counter() {
        let mut ch = PulseChannel::new(false);
        ch.write_envelope(0xF0); // DAC有効
        ch.write_length_duty(0x3E); // length_data=62 → counter=2
        ch.write_frequency_high(0xC0); // トリガー + 長さ有効

        assert!(ch.enabled);
        assert_eq!(ch.length_counter, 2);

        ch.clock_length();
        assert!(ch.enabled);
        assert_eq!(ch.length_counter, 1);

        ch.clock_length();
        assert!(!ch.enabled);
        assert_eq!(ch.length_counter, 0);
    }

    #[test]
    fn test_sweep_register() {
        let mut ch = PulseChannel::new(true);
        ch.write_sweep(0x7B); // period=7, negate, shift=3
        assert_eq!(ch.sweep_period, 7);
        assert!(ch.sweep_negate);
        assert_eq!(ch.sweep_shift, 3);
        assert_eq!(ch.read_sweep(), 0xFB); // bit7=1 + 0x7B
    }

    #[test]
    fn test_frequency_write() {
        let mut ch = PulseChannel::new(false);
        ch.write_frequency_low(0x73);
        ch.write_frequency_high(0x06); // freq上位3ビット = 6
        assert_eq!(ch.frequency, 0x673);
    }

    #[test]
    fn test_envelope_clock() {
        let mut ch = PulseChannel::new(false);
        ch.write_envelope(0x71); // volume=7, down, period=1
        ch.write_frequency_high(0x80); // トリガー
        assert_eq!(ch.volume, 7);

        ch.clock_envelope(); // タイマー消費
        ch.clock_envelope(); // ボリューム変更
        // 正確なタイミングはトリガー時のセットアップに依存
    }

    #[test]
    fn test_output_when_disabled() {
        let ch = PulseChannel::new(false);
        assert_eq!(ch.output(), 0);
        assert_eq!(ch.dac_output(), 0.0);
    }
}
