// src/cpu/timer.rs
// GameBoy タイマーシステム

/// GameBoy タイマー
/// DIV: 16bit内部カウンタ（上位8bitを0xFF04で読み出し）
/// TIMA: タイマーカウンタ（0xFF05、オーバーフローで割り込み）
/// TMA: タイマーモジュロ（0xFF06、TIMAオーバーフロー時のリロード値）
/// TAC: タイマー制御（0xFF07、有効/無効・周波数選択）
pub struct Timer {
    /// 内部16bitカウンタ（DIVは上位8bit）
    pub internal_counter: u16,
    /// タイマーカウンタ
    pub tima: u8,
    /// タイマーモジュロ（リロード値）
    pub tma: u8,
    /// タイマー制御
    pub tac: u8,
    /// Timer割り込み要求フラグ
    pub interrupt_request: bool,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            internal_counter: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            interrupt_request: false,
        }
    }

    /// 1 Tサイクル分タイマーを進める
    pub fn tick(&mut self) {
        let old_counter = self.internal_counter;
        self.internal_counter = self.internal_counter.wrapping_add(1);

        // タイマー有効時のみTIMAを更新
        if self.is_enabled() {
            let bit = self.get_clock_bit();
            // falling edge検出: 旧カウンタのbitが1→新カウンタのbitが0
            let old_bit = (old_counter >> bit) & 1;
            let new_bit = (self.internal_counter >> bit) & 1;

            if old_bit == 1 && new_bit == 0 {
                // TIMAをインクリメント
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                if overflow {
                    self.tima = self.tma; // TMAからリロード
                    self.interrupt_request = true;
                } else {
                    self.tima = new_tima;
                }
            }
        }
    }

    /// DIVレジスタ読み出し（内部カウンタの上位8bit）
    pub fn read_div(&self) -> u8 {
        (self.internal_counter >> 8) as u8
    }

    /// DIVレジスタ書き込み（任意の値でカウンタをゼロリセット）
    pub fn write_div(&mut self) {
        self.internal_counter = 0;
    }

    /// タイマーが有効かどうか
    fn is_enabled(&self) -> bool {
        self.tac & 0x04 != 0
    }

    /// TACの周波数選択ビットに応じた内部カウンタのbit位置
    /// クロック速度:
    ///   00: CPU/1024 (4096 Hz)   → bit 9
    ///   01: CPU/16   (262144 Hz) → bit 3
    ///   10: CPU/64   (65536 Hz)  → bit 5
    ///   11: CPU/256  (16384 Hz)  → bit 7
    fn get_clock_bit(&self) -> u16 {
        match self.tac & 0x03 {
            0 => 9,
            1 => 3,
            2 => 5,
            3 => 7,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new();
        assert_eq!(timer.read_div(), 0);
        assert_eq!(timer.tima, 0);
        assert_eq!(timer.tma, 0);
        assert_eq!(timer.tac, 0);
    }

    #[test]
    fn test_div_increment() {
        let mut timer = Timer::new();
        // 256 Tサイクルで DIV が1増える
        for _ in 0..256 {
            timer.tick();
        }
        assert_eq!(timer.read_div(), 1);
    }

    #[test]
    fn test_div_write_resets() {
        let mut timer = Timer::new();
        for _ in 0..512 {
            timer.tick();
        }
        assert_eq!(timer.read_div(), 2);
        timer.write_div();
        assert_eq!(timer.read_div(), 0);
    }

    #[test]
    fn test_timer_disabled() {
        let mut timer = Timer::new();
        timer.tac = 0x00; // 無効
        timer.tima = 0;
        for _ in 0..10000 {
            timer.tick();
        }
        assert_eq!(timer.tima, 0); // TIMAは変化しない
    }

    #[test]
    fn test_timer_overflow_interrupt() {
        let mut timer = Timer::new();
        timer.tac = 0x05; // 有効、CPU/16 (bit 3)
        timer.tima = 0xFF;
        timer.tma = 0x42;

        // CPU/16 = 16 Tサイクルで1回TIMAインクリメント
        // bit 3のfalling edgeは内部カウンタが8→0の遷移（16サイクル毎）
        for _ in 0..16 {
            timer.tick();
        }

        // TIMAがオーバーフロー → TMAリロード + 割り込み
        assert_eq!(timer.tima, 0x42);
        assert!(timer.interrupt_request);
    }

    #[test]
    fn test_timer_frequency_selection() {
        // CPU/16モード: 16 Tサイクルで1回TIMAインクリメント
        let mut timer = Timer::new();
        timer.tac = 0x05; // 有効、CPU/16
        timer.tima = 0x00;

        for _ in 0..16 {
            timer.tick();
        }
        assert_eq!(timer.tima, 1);

        for _ in 0..16 {
            timer.tick();
        }
        assert_eq!(timer.tima, 2);
    }
}
