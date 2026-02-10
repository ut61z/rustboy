// src/cpu/interrupts.rs
// GameBoy 割り込みシステム

/// 割り込み種別
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Interrupt {
    VBlank  = 0, // bit 0: 0x0040
    Stat    = 1, // bit 1: 0x0048
    Timer   = 2, // bit 2: 0x0050
    Serial  = 3, // bit 3: 0x0058
    Joypad  = 4, // bit 4: 0x0060
}

impl Interrupt {
    /// 割り込みのビットマスク
    pub fn mask(self) -> u8 {
        1 << (self as u8)
    }

    /// 割り込みハンドラのアドレス
    pub fn handler_address(self) -> u16 {
        match self {
            Interrupt::VBlank => 0x0040,
            Interrupt::Stat   => 0x0048,
            Interrupt::Timer  => 0x0050,
            Interrupt::Serial => 0x0058,
            Interrupt::Joypad => 0x0060,
        }
    }

    /// 優先順位順（VBlankが最高）にすべての割り込みを返す
    pub fn all_by_priority() -> &'static [Interrupt] {
        &[
            Interrupt::VBlank,
            Interrupt::Stat,
            Interrupt::Timer,
            Interrupt::Serial,
            Interrupt::Joypad,
        ]
    }
}

/// 保留中の割り込みのうち最高優先度のものを取得
/// IF & IE で有効かつ要求されている割り込みを優先順位順にチェック
pub fn get_pending_interrupt(interrupt_flag: u8, interrupt_enable: u8) -> Option<Interrupt> {
    let pending = interrupt_flag & interrupt_enable & 0x1F;
    if pending == 0 {
        return None;
    }

    for &interrupt in Interrupt::all_by_priority() {
        if pending & interrupt.mask() != 0 {
            return Some(interrupt);
        }
    }

    None
}

/// 保留中の割り込みがあるかどうかだけを確認（HALTからの復帰判定用）
pub fn has_pending_interrupt(interrupt_flag: u8, interrupt_enable: u8) -> bool {
    (interrupt_flag & interrupt_enable & 0x1F) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupt_masks() {
        assert_eq!(Interrupt::VBlank.mask(), 0x01);
        assert_eq!(Interrupt::Stat.mask(), 0x02);
        assert_eq!(Interrupt::Timer.mask(), 0x04);
        assert_eq!(Interrupt::Serial.mask(), 0x08);
        assert_eq!(Interrupt::Joypad.mask(), 0x10);
    }

    #[test]
    fn test_interrupt_handler_addresses() {
        assert_eq!(Interrupt::VBlank.handler_address(), 0x0040);
        assert_eq!(Interrupt::Stat.handler_address(), 0x0048);
        assert_eq!(Interrupt::Timer.handler_address(), 0x0050);
        assert_eq!(Interrupt::Serial.handler_address(), 0x0058);
        assert_eq!(Interrupt::Joypad.handler_address(), 0x0060);
    }

    #[test]
    fn test_get_pending_interrupt_priority() {
        // VBlankとTimerが同時に保留 → VBlankが先
        let interrupt_flag = 0x05; // VBlank + Timer
        let interrupt_enable = 0x05;
        assert_eq!(get_pending_interrupt(interrupt_flag, interrupt_enable), Some(Interrupt::VBlank));
    }

    #[test]
    fn test_get_pending_interrupt_none() {
        // 割り込み無効
        assert_eq!(get_pending_interrupt(0x01, 0x00), None);
        // 割り込みフラグなし
        assert_eq!(get_pending_interrupt(0x00, 0x01), None);
    }

    #[test]
    fn test_get_pending_interrupt_ie_mask() {
        // IF=VBlank+Timer, IE=Timerのみ → Timerが返る
        assert_eq!(get_pending_interrupt(0x05, 0x04), Some(Interrupt::Timer));
    }

    #[test]
    fn test_has_pending_interrupt() {
        assert!(has_pending_interrupt(0x01, 0x01));
        assert!(!has_pending_interrupt(0x01, 0x00));
        assert!(!has_pending_interrupt(0x00, 0x01));
    }
}
