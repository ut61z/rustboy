// src/joypad.rs
// GameBoy ジョイパッド入力システム
//
// JOYP (0xFF00) レジスタ:
//   Bit 5 - P15: ボタンキー選択 (0=選択)
//   Bit 4 - P14: 方向キー選択  (0=選択)
//   Bit 3 - P13: Down  or Start  (0=押下)
//   Bit 2 - P12: Up    or Select (0=押下)
//   Bit 1 - P11: Left  or B      (0=押下)
//   Bit 0 - P10: Right or A      (0=押下)
//
// 読み取り時: 選択されたグループのボタン状態が下位4bitに反映
// 未選択時は0xF（全ボタン離し）を返す

/// ジョイパッドボタン
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoypadButton {
    // 方向キー (P14)
    Right,
    Left,
    Up,
    Down,
    // ボタンキー (P15)
    A,
    B,
    Select,
    Start,
}

/// ジョイパッドコントローラ
pub struct Joypad {
    /// ボタンキー状態 (bit0=A, bit1=B, bit2=Select, bit3=Start, 0=押下)
    button_keys: u8,
    /// 方向キー状態 (bit0=Right, bit1=Left, bit2=Up, bit3=Down, 0=押下)
    direction_keys: u8,
    /// 選択レジスタ (bit4=方向キー選択, bit5=ボタンキー選択)
    select: u8,
    /// 割り込み要求フラグ
    pub interrupt_request: bool,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            button_keys: 0x0F,    // 全ボタン離し
            direction_keys: 0x0F, // 全方向キー離し
            select: 0x30,         // 初期値: 両方未選択
            interrupt_request: false,
        }
    }

    /// JOYPレジスタの読み取り
    pub fn read(&self) -> u8 {
        let mut result = self.select | 0xC0; // 上位2bitは常に1

        // P14 (bit4) が0なら方向キー選択
        if self.select & 0x10 == 0 {
            result = (result & 0xF0) | (self.direction_keys & 0x0F);
        }
        // P15 (bit5) が0ならボタンキー選択
        if self.select & 0x20 == 0 {
            result = (result & 0xF0) | (self.button_keys & 0x0F);
        }
        // 両方未選択の場合は0x0F（全ボタン離し）
        if self.select & 0x30 == 0x30 {
            result |= 0x0F;
        }

        result
    }

    /// JOYPレジスタへの書き込み（上位2bitの選択のみ有効）
    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }

    /// ボタン押下
    pub fn press(&mut self, button: JoypadButton) {
        let old_state = self.get_current_input();

        match button {
            JoypadButton::Right  => self.direction_keys &= !0x01,
            JoypadButton::Left   => self.direction_keys &= !0x02,
            JoypadButton::Up     => self.direction_keys &= !0x04,
            JoypadButton::Down   => self.direction_keys &= !0x08,
            JoypadButton::A      => self.button_keys &= !0x01,
            JoypadButton::B      => self.button_keys &= !0x02,
            JoypadButton::Select => self.button_keys &= !0x04,
            JoypadButton::Start  => self.button_keys &= !0x08,
        }

        let new_state = self.get_current_input();

        // High→Low遷移で割り込み要求
        if old_state & !new_state != 0 {
            self.interrupt_request = true;
        }
    }

    /// ボタン離し
    pub fn release(&mut self, button: JoypadButton) {
        match button {
            JoypadButton::Right  => self.direction_keys |= 0x01,
            JoypadButton::Left   => self.direction_keys |= 0x02,
            JoypadButton::Up     => self.direction_keys |= 0x04,
            JoypadButton::Down   => self.direction_keys |= 0x08,
            JoypadButton::A      => self.button_keys |= 0x01,
            JoypadButton::B      => self.button_keys |= 0x02,
            JoypadButton::Select => self.button_keys |= 0x04,
            JoypadButton::Start  => self.button_keys |= 0x08,
        }
    }

    /// 現在選択されているグループの入力状態を取得
    fn get_current_input(&self) -> u8 {
        let mut input = 0x0F;
        if self.select & 0x10 == 0 {
            input &= self.direction_keys;
        }
        if self.select & 0x20 == 0 {
            input &= self.button_keys;
        }
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joypad_creation() {
        let joypad = Joypad::new();
        // 初期状態: 全ボタン離し、両グループ未選択
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }

    #[test]
    fn test_direction_key_select() {
        let mut joypad = Joypad::new();

        // 方向キーグループ選択
        joypad.write(0x20); // P14=0 (方向キー選択), P15=1

        // 全キー離し状態
        assert_eq!(joypad.read() & 0x0F, 0x0F);

        // Rightを押下
        joypad.press(JoypadButton::Right);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // bit0=0

        // Upも押下
        joypad.press(JoypadButton::Up);
        assert_eq!(joypad.read() & 0x0F, 0x0A); // bit0=0, bit2=0
    }

    #[test]
    fn test_button_key_select() {
        let mut joypad = Joypad::new();

        // ボタンキーグループ選択
        joypad.write(0x10); // P14=1, P15=0 (ボタンキー選択)

        // Aを押下
        joypad.press(JoypadButton::A);
        assert_eq!(joypad.read() & 0x0F, 0x0E); // bit0=0

        // Startも押下
        joypad.press(JoypadButton::Start);
        assert_eq!(joypad.read() & 0x0F, 0x06); // bit0=0, bit3=0
    }

    #[test]
    fn test_button_release() {
        let mut joypad = Joypad::new();
        joypad.write(0x20); // 方向キー選択

        joypad.press(JoypadButton::Right);
        assert_eq!(joypad.read() & 0x01, 0x00);

        joypad.release(JoypadButton::Right);
        assert_eq!(joypad.read() & 0x01, 0x01);
    }

    #[test]
    fn test_group_isolation() {
        let mut joypad = Joypad::new();

        // ボタンキーAを押下
        joypad.press(JoypadButton::A);

        // 方向キーグループ選択時にはAの状態が見えない
        joypad.write(0x20); // 方向キー選択
        assert_eq!(joypad.read() & 0x0F, 0x0F);

        // ボタンキーグループ選択時にはAが見える
        joypad.write(0x10); // ボタンキー選択
        assert_eq!(joypad.read() & 0x0F, 0x0E);
    }

    #[test]
    fn test_interrupt_on_press() {
        let mut joypad = Joypad::new();
        joypad.write(0x20); // 方向キー選択

        assert!(!joypad.interrupt_request);
        joypad.press(JoypadButton::Right);
        assert!(joypad.interrupt_request);

        // フラグクリア後、離しでは割り込みが発生しない
        joypad.interrupt_request = false;
        joypad.release(JoypadButton::Right);
        assert!(!joypad.interrupt_request);
    }

    #[test]
    fn test_both_groups_unselected() {
        let mut joypad = Joypad::new();
        joypad.write(0x30); // 両方未選択

        joypad.press(JoypadButton::A);
        joypad.press(JoypadButton::Right);

        // 両グループ未選択時は全ボタン離し状態
        assert_eq!(joypad.read() & 0x0F, 0x0F);
    }
}
