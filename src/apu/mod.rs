// src/apu/mod.rs
// GameBoy APU (Audio Processing Unit) コア
//
// 4チャンネル構成:
//   Channel 1: パルス波 + 周波数スイープ (NR10-NR14)
//   Channel 2: パルス波 (NR21-NR24)
//   Channel 3: ウェーブテーブル (NR30-NR34 + Wave RAM)
//   Channel 4: ノイズ (NR41-NR44)
//
// マスター制御:
//   NR50 (0xFF24): マスター音量/VINパニング
//   NR51 (0xFF25): 音声出力選択（各チャンネルの左右パニング）
//   NR52 (0xFF26): APU電源・チャンネル状態
//
// フレームシーケンサ (512Hz):
//   Step 0: 長さカウンタ
//   Step 1: (なし)
//   Step 2: 長さカウンタ、スイープ
//   Step 3: (なし)
//   Step 4: 長さカウンタ
//   Step 5: (なし)
//   Step 6: 長さカウンタ、スイープ
//   Step 7: エンベロープ

pub mod pulse;
pub mod wave;
pub mod noise;

use pulse::PulseChannel;
use wave::WaveChannel;
use noise::NoiseChannel;
use crate::memory_map::io_registers::*;

/// フレームシーケンサの周期 (CPUサイクル: 4,194,304 / 512 = 8192)
const FRAME_SEQUENCER_PERIOD: u16 = 8192;

/// APU (Audio Processing Unit)
pub struct Apu {
    /// Channel 1: パルス + スイープ
    pub channel1: PulseChannel,
    /// Channel 2: パルス
    pub channel2: PulseChannel,
    /// Channel 3: ウェーブ
    pub channel3: WaveChannel,
    /// Channel 4: ノイズ
    pub channel4: NoiseChannel,

    // NR50: マスター音量
    /// VIN→左出力 (未使用だがレジスタとして保持)
    pub vin_left: bool,
    /// 左ボリューム (0-7)
    pub left_volume: u8,
    /// VIN→右出力
    pub vin_right: bool,
    /// 右ボリューム (0-7)
    pub right_volume: u8,

    // NR51: パニング
    /// 各チャンネルの左右出力設定
    pub panning: u8,

    // NR52: APU電源
    /// APU有効フラグ
    pub power: bool,

    /// フレームシーケンサタイマー
    frame_sequencer_timer: u16,
    /// フレームシーケンサステップ (0-7)
    frame_sequencer_step: u8,

    /// オーディオサンプルバッファ（左右インターリーブ、-1.0〜1.0）
    pub sample_buffer: Vec<f32>,
    /// サンプル生成用ダウンサンプルカウンタ
    downsample_counter: u32,
    /// サンプリングレート (デフォルト: 44100Hz)
    pub sample_rate: u32,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            channel1: PulseChannel::new(true),
            channel2: PulseChannel::new(false),
            channel3: WaveChannel::new(),
            channel4: NoiseChannel::new(),
            vin_left: false,
            left_volume: 0,
            vin_right: false,
            right_volume: 0,
            panning: 0x00,
            power: false,
            frame_sequencer_timer: FRAME_SEQUENCER_PERIOD,
            frame_sequencer_step: 0,
            sample_buffer: Vec::new(),
            downsample_counter: 0,
            sample_rate: 44100,
        }
    }

    /// APUを1 CPUサイクル進める
    pub fn tick(&mut self) {
        if !self.power {
            return;
        }

        // 各チャンネルの周波数タイマーを進める
        self.channel1.tick();
        self.channel2.tick();
        self.channel3.tick();
        self.channel4.tick();

        // フレームシーケンサ
        self.frame_sequencer_timer = self.frame_sequencer_timer.saturating_sub(1);
        if self.frame_sequencer_timer == 0 {
            self.frame_sequencer_timer = FRAME_SEQUENCER_PERIOD;
            self.clock_frame_sequencer();
        }

        // ダウンサンプリング (CPUクロック→サンプリングレート)
        self.downsample_counter += self.sample_rate;
        if self.downsample_counter >= 4_194_304 {
            self.downsample_counter -= 4_194_304;
            self.generate_sample();
        }
    }

    /// フレームシーケンサのクロック
    fn clock_frame_sequencer(&mut self) {
        match self.frame_sequencer_step {
            0 => {
                // 長さカウンタ
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
            }
            1 => {} // なし
            2 => {
                // 長さカウンタ + スイープ
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
                self.channel1.clock_sweep();
            }
            3 => {} // なし
            4 => {
                // 長さカウンタ
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
            }
            5 => {} // なし
            6 => {
                // 長さカウンタ + スイープ
                self.channel1.clock_length();
                self.channel2.clock_length();
                self.channel3.clock_length();
                self.channel4.clock_length();
                self.channel1.clock_sweep();
            }
            7 => {
                // エンベロープ
                self.channel1.clock_envelope();
                self.channel2.clock_envelope();
                self.channel4.clock_envelope();
            }
            _ => {}
        }

        self.frame_sequencer_step = (self.frame_sequencer_step + 1) & 0x07;
    }

    /// オーディオサンプルを生成してバッファに追加
    fn generate_sample(&mut self) {
        let ch1 = self.channel1.dac_output();
        let ch2 = self.channel2.dac_output();
        let ch3 = self.channel3.dac_output();
        let ch4 = self.channel4.dac_output();

        // ミキシング（パニング適用）
        let mut left: f32 = 0.0;
        let mut right: f32 = 0.0;

        if self.panning & 0x10 != 0 { left += ch1; }
        if self.panning & 0x20 != 0 { left += ch2; }
        if self.panning & 0x40 != 0 { left += ch3; }
        if self.panning & 0x80 != 0 { left += ch4; }

        if self.panning & 0x01 != 0 { right += ch1; }
        if self.panning & 0x02 != 0 { right += ch2; }
        if self.panning & 0x04 != 0 { right += ch3; }
        if self.panning & 0x08 != 0 { right += ch4; }

        // マスター音量適用 (0-7 → 1/8-8/8)
        left *= (self.left_volume as f32 + 1.0) / 8.0;
        right *= (self.right_volume as f32 + 1.0) / 8.0;

        // 4チャンネル分の正規化
        left /= 4.0;
        right /= 4.0;

        self.sample_buffer.push(left);
        self.sample_buffer.push(right);
    }

    /// サンプルバッファを取り出す（取り出し後はクリア）
    pub fn drain_samples(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.sample_buffer)
    }

    /// I/Oレジスタの読み取り
    pub fn read(&self, addr: u16) -> u8 {
        if !self.power && addr != NR52 {
            // APU電源オフ時はNR52以外は読めない（Wave RAM除く）
            if (WAVE_RAM_START..=WAVE_RAM_END).contains(&addr) {
                return self.channel3.read_wave_ram(addr);
            }
            return 0xFF;
        }

        match addr {
            // Channel 1
            NR10 => self.channel1.read_sweep(),
            NR11 => self.channel1.read_length_duty(),
            NR12 => self.channel1.read_envelope(),
            NR13 => 0xFF, // 書き込みのみ
            NR14 => self.channel1.read_frequency_high(),

            // Channel 2
            NR21 => self.channel2.read_length_duty(),
            NR22 => self.channel2.read_envelope(),
            NR23 => 0xFF, // 書き込みのみ
            NR24 => self.channel2.read_frequency_high(),

            // Channel 3
            NR30 => self.channel3.read_dac(),
            NR31 => 0xFF, // 書き込みのみ
            NR32 => self.channel3.read_output_level(),
            NR33 => 0xFF, // 書き込みのみ
            NR34 => self.channel3.read_frequency_high(),

            // Channel 4
            NR41 => 0xFF, // 書き込みのみ
            NR42 => self.channel4.read_envelope(),
            NR43 => self.channel4.read_polynomial(),
            NR44 => self.channel4.read_control(),

            // Master
            NR50 => self.read_nr50(),
            NR51 => self.panning,
            NR52 => self.read_nr52(),

            // Wave RAM
            WAVE_RAM_START..=WAVE_RAM_END => self.channel3.read_wave_ram(addr),

            _ => 0xFF,
        }
    }

    /// I/Oレジスタへの書き込み
    pub fn write(&mut self, addr: u16, value: u8) {
        // Wave RAMはAPU電源に関係なく書き込み可能
        if (WAVE_RAM_START..=WAVE_RAM_END).contains(&addr) {
            self.channel3.write_wave_ram(addr, value);
            return;
        }

        // NR52の電源ビットはいつでも書き込み可能
        if addr == NR52 {
            self.write_nr52(value);
            return;
        }

        // APU電源オフ時は書き込み無視 (NR11/NR21/NR31/NR41の長さ除く)
        if !self.power {
            match addr {
                NR11 => self.channel1.write_length_duty(value),
                NR21 => self.channel2.write_length_duty(value),
                NR31 => self.channel3.write_length(value),
                NR41 => self.channel4.write_length(value),
                _ => {}
            }
            return;
        }

        match addr {
            // Channel 1
            NR10 => self.channel1.write_sweep(value),
            NR11 => self.channel1.write_length_duty(value),
            NR12 => self.channel1.write_envelope(value),
            NR13 => self.channel1.write_frequency_low(value),
            NR14 => self.channel1.write_frequency_high(value),

            // Channel 2
            NR21 => self.channel2.write_length_duty(value),
            NR22 => self.channel2.write_envelope(value),
            NR23 => self.channel2.write_frequency_low(value),
            NR24 => self.channel2.write_frequency_high(value),

            // Channel 3
            NR30 => self.channel3.write_dac(value),
            NR31 => self.channel3.write_length(value),
            NR32 => self.channel3.write_output_level(value),
            NR33 => self.channel3.write_frequency_low(value),
            NR34 => self.channel3.write_frequency_high(value),

            // Channel 4
            NR41 => self.channel4.write_length(value),
            NR42 => self.channel4.write_envelope(value),
            NR43 => self.channel4.write_polynomial(value),
            NR44 => self.channel4.write_control(value),

            // Master
            NR50 => self.write_nr50(value),
            NR51 => self.panning = value,

            _ => {}
        }
    }

    /// NR50レジスタの読み取り
    fn read_nr50(&self) -> u8 {
        (if self.vin_left { 0x80 } else { 0x00 })
            | (self.left_volume << 4)
            | (if self.vin_right { 0x08 } else { 0x00 })
            | self.right_volume
    }

    /// NR50レジスタへの書き込み
    fn write_nr50(&mut self, value: u8) {
        self.vin_left = value & 0x80 != 0;
        self.left_volume = (value >> 4) & 0x07;
        self.vin_right = value & 0x08 != 0;
        self.right_volume = value & 0x07;
    }

    /// NR52レジスタの読み取り
    fn read_nr52(&self) -> u8 {
        0x70 // bit 4-6は常に1
            | if self.power { 0x80 } else { 0x00 }
            | if self.channel4.enabled { 0x08 } else { 0x00 }
            | if self.channel3.enabled { 0x04 } else { 0x00 }
            | if self.channel2.enabled { 0x02 } else { 0x00 }
            | if self.channel1.enabled { 0x01 } else { 0x00 }
    }

    /// NR52レジスタへの書き込み
    fn write_nr52(&mut self, value: u8) {
        let new_power = value & 0x80 != 0;

        if self.power && !new_power {
            // APU電源オフ: 全レジスタをクリア
            self.power_off();
        } else if !self.power && new_power {
            // APU電源オン
            self.frame_sequencer_step = 0;
        }

        self.power = new_power;
    }

    /// APU電源オフ時の全レジスタクリア
    fn power_off(&mut self) {
        self.channel1 = PulseChannel::new(true);
        self.channel2 = PulseChannel::new(false);
        // Wave RAMは保持
        let wave_ram_backup = self.channel3.wave_ram;
        self.channel3 = WaveChannel::new();
        self.channel3.wave_ram = wave_ram_backup;
        self.channel4 = NoiseChannel::new();

        self.vin_left = false;
        self.left_volume = 0;
        self.vin_right = false;
        self.right_volume = 0;
        self.panning = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apu_creation() {
        let apu = Apu::new();
        assert!(!apu.power);
        assert!(!apu.channel1.enabled);
        assert!(!apu.channel2.enabled);
        assert!(!apu.channel3.enabled);
        assert!(!apu.channel4.enabled);
    }

    #[test]
    fn test_apu_power_on_off() {
        let mut apu = Apu::new();

        // 電源オン
        apu.write(NR52, 0x80);
        assert!(apu.power);

        // レジスタ書き込み可能
        apu.write(NR50, 0x77); // 左右ボリューム最大
        assert_eq!(apu.left_volume, 7);
        assert_eq!(apu.right_volume, 7);

        // 電源オフ
        apu.write(NR52, 0x00);
        assert!(!apu.power);
        // レジスタがクリアされる
        assert_eq!(apu.left_volume, 0);
        assert_eq!(apu.right_volume, 0);
    }

    #[test]
    fn test_apu_nr52_read() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        // 初期状態: 電源オン、全チャンネル無効
        let nr52 = apu.read(NR52);
        assert_eq!(nr52 & 0x80, 0x80); // 電源オン
        assert_eq!(nr52 & 0x0F, 0x00); // 全チャンネル無効
        assert_eq!(nr52 & 0x70, 0x70); // 未使用ビットは1
    }

    #[test]
    fn test_apu_channel1_trigger() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        apu.write(NR12, 0xF0); // ボリューム15、DAC有効
        apu.write(NR14, 0x80); // トリガー

        assert!(apu.channel1.enabled);
        let nr52 = apu.read(NR52);
        assert_eq!(nr52 & 0x01, 0x01); // Channel 1 有効
    }

    #[test]
    fn test_apu_nr50_register() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        apu.write(NR50, 0xA5);
        assert!(apu.vin_left);
        assert_eq!(apu.left_volume, 2);
        assert!(!apu.vin_right);
        assert_eq!(apu.right_volume, 5);
        assert_eq!(apu.read(NR50), 0xA5);
    }

    #[test]
    fn test_apu_nr51_panning() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        apu.write(NR51, 0x12);
        assert_eq!(apu.panning, 0x12);
        assert_eq!(apu.read(NR51), 0x12);
    }

    #[test]
    fn test_apu_power_off_preserves_wave_ram() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        // Wave RAMにデータを書き込み
        apu.write(WAVE_RAM_START, 0x12);
        apu.write(WAVE_RAM_START + 1, 0x34);

        // 電源オフ
        apu.write(NR52, 0x00);

        // Wave RAMは保持される
        assert_eq!(apu.read(WAVE_RAM_START), 0x12);
        assert_eq!(apu.read(WAVE_RAM_START + 1), 0x34);
    }

    #[test]
    fn test_apu_wave_ram_accessible_when_off() {
        let mut apu = Apu::new();
        // 電源オフでもWave RAMは読み書き可能
        assert!(!apu.power);

        apu.write(WAVE_RAM_START, 0xAB);
        assert_eq!(apu.read(WAVE_RAM_START), 0xAB);
    }

    #[test]
    fn test_apu_registers_locked_when_off() {
        let mut apu = Apu::new();
        // APU電源オフ時はレジスタに書き込めない
        apu.write(NR50, 0x77);
        assert_eq!(apu.left_volume, 0); // 変更されない

        // NR52は書き込み可能
        apu.write(NR52, 0x80);
        assert!(apu.power);
    }

    #[test]
    fn test_apu_tick_generates_samples() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        // しばらくtick
        for _ in 0..44100 {
            apu.tick();
        }

        // サンプルが生成されているはず
        let samples = apu.drain_samples();
        assert!(!samples.is_empty());
        // ステレオなので偶数
        assert_eq!(samples.len() % 2, 0);
    }

    #[test]
    fn test_apu_no_tick_when_off() {
        let mut apu = Apu::new();
        // 電源オフではtickしてもサンプルが生成されない
        for _ in 0..1000 {
            apu.tick();
        }
        let samples = apu.drain_samples();
        assert!(samples.is_empty());
    }

    #[test]
    fn test_apu_read_write_only_registers() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        // 書き込みのみのレジスタは0xFFを返す
        assert_eq!(apu.read(NR13), 0xFF);
        assert_eq!(apu.read(NR23), 0xFF);
        assert_eq!(apu.read(NR31), 0xFF);
        assert_eq!(apu.read(NR33), 0xFF);
        assert_eq!(apu.read(NR41), 0xFF);
    }

    #[test]
    fn test_frame_sequencer_length() {
        let mut apu = Apu::new();
        apu.write(NR52, 0x80);

        // Channel 1に短い長さカウンタを設定
        apu.write(NR12, 0xF0); // DAC有効
        apu.write(NR11, 0x3F); // length_data=63 → counter=1
        apu.write(NR14, 0xC0); // トリガー + 長さ有効

        assert!(apu.channel1.enabled);

        // フレームシーケンサのstep 0まで進める (8192サイクル)
        for _ in 0..8192 {
            apu.tick();
        }

        // 長さカウンタが消費されてチャンネル無効化
        assert!(!apu.channel1.enabled);
    }
}
