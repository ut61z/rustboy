// src/serial.rs
// GameBoy シリアル通信コントローラ
//
// SB (0xFF01): シリアル転送データ — 8ビットデータレジスタ
// SC (0xFF02): シリアル転送制御
//   Bit 7: 転送開始フラグ (1=転送要求/実行中)
//   Bit 1: クロック速度 (CGBのみ、DMGでは無視)
//   Bit 0: シフトクロック (0=外部クロック, 1=内部クロック)
//
// 内部クロック使用時: 8192Hz (512 CPUサイクル/bit、4096サイクル/バイト)
// 転送完了時(8ビットシフト後): SC bit7をクリアし、シリアル割り込みを要求

/// シリアル通信コントローラ
pub struct Serial {
    /// シリアル転送データ (SB: 0xFF01)
    pub sb: u8,
    /// シリアル転送制御 (SC: 0xFF02)
    pub sc: u8,
    /// 転送サイクルカウンタ
    transfer_counter: u16,
    /// 転送ビットカウンタ
    bit_counter: u8,
    /// 割り込み要求フラグ
    pub interrupt_request: bool,
}

/// 内部クロック: 1ビットあたり512 CPUサイクル (4,194,304 Hz / 8192 Hz)
const CYCLES_PER_BIT: u16 = 512;

impl Serial {
    pub fn new() -> Self {
        Self {
            sb: 0x00,
            sc: 0x7E, // 初期値: 転送停止、未使用ビットは1
            transfer_counter: 0,
            bit_counter: 0,
            interrupt_request: false,
        }
    }

    /// SBレジスタの読み取り
    pub fn read_sb(&self) -> u8 {
        self.sb
    }

    /// SBレジスタへの書き込み
    pub fn write_sb(&mut self, value: u8) {
        self.sb = value;
    }

    /// SCレジスタの読み取り (未使用ビットは1)
    pub fn read_sc(&self) -> u8 {
        self.sc | 0x7E // Bit 1-6は常に1 (DMG)
    }

    /// SCレジスタへの書き込み
    pub fn write_sc(&mut self, value: u8) {
        self.sc = value;
        // 転送開始 (bit7=1, bit0=1: 内部クロック)
        if value & 0x81 == 0x81 {
            self.transfer_counter = 0;
            self.bit_counter = 0;
        }
    }

    /// 転送がアクティブかどうか
    pub fn is_transferring(&self) -> bool {
        self.sc & 0x80 != 0 && self.sc & 0x01 != 0
    }

    /// シリアル通信を1サイクル進める
    pub fn tick(&mut self) {
        if !self.is_transferring() {
            return;
        }

        self.transfer_counter += 1;

        if self.transfer_counter >= CYCLES_PER_BIT {
            self.transfer_counter = 0;
            self.bit_counter += 1;

            // データをシフト（外部デバイスなし→0xFFを受信）
            self.sb = (self.sb << 1) | 0x01;

            if self.bit_counter >= 8 {
                // 転送完了
                self.sc &= !0x80; // 転送フラグをクリア
                self.bit_counter = 0;
                self.interrupt_request = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_creation() {
        let serial = Serial::new();
        assert_eq!(serial.read_sb(), 0x00);
        assert_eq!(serial.read_sc() & 0x81, 0x00); // 転送停止、外部クロック
        assert!(!serial.is_transferring());
        assert!(!serial.interrupt_request);
    }

    #[test]
    fn test_serial_sb_read_write() {
        let mut serial = Serial::new();
        serial.write_sb(0x42);
        assert_eq!(serial.read_sb(), 0x42);
        serial.write_sb(0xFF);
        assert_eq!(serial.read_sb(), 0xFF);
    }

    #[test]
    fn test_serial_sc_unused_bits() {
        let serial = Serial::new();
        // SCの未使用ビット(1-6)は読み取り時に1
        assert_eq!(serial.read_sc() & 0x7E, 0x7E);
    }

    #[test]
    fn test_serial_transfer_start() {
        let mut serial = Serial::new();
        serial.write_sb(0xAB);
        serial.write_sc(0x81); // 転送開始（内部クロック）
        assert!(serial.is_transferring());
    }

    #[test]
    fn test_serial_transfer_external_clock_no_transfer() {
        let mut serial = Serial::new();
        serial.write_sc(0x80); // 転送開始だが外部クロック
        // 外部クロックでは内部tickで進まない
        assert!(!serial.is_transferring());
    }

    #[test]
    fn test_serial_transfer_complete() {
        let mut serial = Serial::new();
        serial.write_sb(0xAB);
        serial.write_sc(0x81); // 内部クロックで転送開始

        // 8ビット転送 = 8 × 512 = 4096サイクル
        for _ in 0..4096 {
            serial.tick();
        }

        // 転送完了: SC bit7クリア、割り込み要求
        assert_eq!(serial.read_sc() & 0x80, 0x00);
        assert!(serial.interrupt_request);
    }

    #[test]
    fn test_serial_receive_ff_without_connection() {
        let mut serial = Serial::new();
        serial.write_sb(0x00);
        serial.write_sc(0x81);

        // 接続なし → 全ビット1を受信
        for _ in 0..4096 {
            serial.tick();
        }

        assert_eq!(serial.read_sb(), 0xFF);
    }

    #[test]
    fn test_serial_inactive_tick() {
        let mut serial = Serial::new();
        serial.write_sb(0x42);

        // 転送を開始していない場合、tickは何もしない
        for _ in 0..5000 {
            serial.tick();
        }

        assert_eq!(serial.read_sb(), 0x42);
        assert!(!serial.interrupt_request);
    }
}
