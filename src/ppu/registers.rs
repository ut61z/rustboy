// PPU関連のレジスタ

pub struct PpuRegisters {
    pub lcdc: u8,    // 0xFF40 - LCD制御
    pub stat: u8,    // 0xFF41 - LCDステータス
    pub scy: u8,     // 0xFF42 - スクロールY
    pub scx: u8,     // 0xFF43 - スクロールX
    pub ly: u8,      // 0xFF44 - LCD Y座標
    pub lyc: u8,     // 0xFF45 - LY比較
    pub bgp: u8,     // 0xFF47 - BGパレット
    pub obp0: u8,    // 0xFF48 - オブジェクトパレット0
    pub obp1: u8,    // 0xFF49 - オブジェクトパレット1
    pub wy: u8,      // 0xFF4A - ウィンドウY
    pub wx: u8,      // 0xFF4B - ウィンドウX
}

impl PpuRegisters {
    pub fn new() -> Self {
        Self {
            lcdc: 0x91,  // デフォルトでLCD有効、BG有効
            stat: 0x00,
            scy: 0x00,
            scx: 0x00,
            ly: 0x00,
            lyc: 0x00,
            bgp: 0xFC,   // デフォルトパレット (11111100)
            obp0: 0xFF,  // デフォルトスプライトパレット0
            obp1: 0xFF,  // デフォルトスプライトパレット1
            wy: 0x00,    // ウィンドウY位置
            wx: 0x00,    // ウィンドウX位置
        }
    }
    
    // LCDC レジスタのビットフラグ
    pub fn is_lcd_enabled(&self) -> bool {
        (self.lcdc & 0x80) != 0
    }
    
    pub fn is_window_tilemap_high(&self) -> bool {
        (self.lcdc & 0x40) != 0
    }
    
    pub fn is_window_enabled(&self) -> bool {
        (self.lcdc & 0x20) != 0
    }
    
    pub fn is_bg_window_tiledata_high(&self) -> bool {
        (self.lcdc & 0x10) != 0
    }
    
    pub fn is_bg_tilemap_high(&self) -> bool {
        (self.lcdc & 0x08) != 0
    }
    
    pub fn is_sprite_size_16(&self) -> bool {
        (self.lcdc & 0x04) != 0
    }
    
    pub fn is_sprite_enabled(&self) -> bool {
        (self.lcdc & 0x02) != 0
    }
    
    pub fn is_bg_enabled(&self) -> bool {
        (self.lcdc & 0x01) != 0
    }
    
    // STAT レジスタのビットフラグ
    pub fn is_lyc_interrupt_enabled(&self) -> bool {
        (self.stat & 0x40) != 0
    }
    
    pub fn is_oam_interrupt_enabled(&self) -> bool {
        (self.stat & 0x20) != 0
    }
    
    pub fn is_vblank_interrupt_enabled(&self) -> bool {
        (self.stat & 0x10) != 0
    }
    
    pub fn is_hblank_interrupt_enabled(&self) -> bool {
        (self.stat & 0x08) != 0
    }
    
    pub fn is_lyc_equal(&self) -> bool {
        (self.stat & 0x04) != 0
    }
    
    pub fn get_mode(&self) -> u8 {
        self.stat & 0x03
    }
    
    // BGP パレット変換 (2ビット -> 2ビット)
    pub fn get_bg_palette_color(&self, color_id: u8) -> u8 {
        match color_id & 0x03 {
            0 => self.bgp & 0x03,
            1 => (self.bgp >> 2) & 0x03,
            2 => (self.bgp >> 4) & 0x03,
            3 => (self.bgp >> 6) & 0x03,
            _ => unreachable!(),
        }
    }

    // OBP0 パレット変換 (スプライトパレット0)
    pub fn get_obp0_color(&self, color_id: u8) -> u8 {
        match color_id & 0x03 {
            0 => self.obp0 & 0x03,         // 透明（使用されない）
            1 => (self.obp0 >> 2) & 0x03,
            2 => (self.obp0 >> 4) & 0x03,
            3 => (self.obp0 >> 6) & 0x03,
            _ => unreachable!(),
        }
    }

    // OBP1 パレット変換 (スプライトパレット1)
    pub fn get_obp1_color(&self, color_id: u8) -> u8 {
        match color_id & 0x03 {
            0 => self.obp1 & 0x03,         // 透明（使用されない）
            1 => (self.obp1 >> 2) & 0x03,
            2 => (self.obp1 >> 4) & 0x03,
            3 => (self.obp1 >> 6) & 0x03,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lcdc_flags() {
        let mut registers = PpuRegisters::new();
        
        // デフォルト値のテスト
        assert!(registers.is_lcd_enabled());
        assert!(registers.is_bg_enabled());
        
        // LCD無効にする
        registers.lcdc = 0x00;
        assert!(!registers.is_lcd_enabled());
        assert!(!registers.is_bg_enabled());
    }
    
    #[test]
    fn test_bg_palette() {
        let mut registers = PpuRegisters::new();
        registers.bgp = 0b11100100;  // 3,2,1,0 の順

        assert_eq!(registers.get_bg_palette_color(0), 0);  // 00
        assert_eq!(registers.get_bg_palette_color(1), 1);  // 01
        assert_eq!(registers.get_bg_palette_color(2), 2);  // 10
        assert_eq!(registers.get_bg_palette_color(3), 3);  // 11
    }

    #[test]
    fn test_sprite_palette() {
        let mut registers = PpuRegisters::new();
        registers.obp0 = 0b11100100;
        registers.obp1 = 0b00011011; // 反転パレット

        // OBP0
        assert_eq!(registers.get_obp0_color(0), 0);
        assert_eq!(registers.get_obp0_color(1), 1);
        assert_eq!(registers.get_obp0_color(2), 2);
        assert_eq!(registers.get_obp0_color(3), 3);

        // OBP1
        assert_eq!(registers.get_obp1_color(0), 3);
        assert_eq!(registers.get_obp1_color(1), 2);
        assert_eq!(registers.get_obp1_color(2), 1);
        assert_eq!(registers.get_obp1_color(3), 0);
    }

    #[test]
    fn test_window_registers() {
        let mut registers = PpuRegisters::new();
        registers.wy = 0x10;
        registers.wx = 0x07; // WX=7 はウィンドウ左端

        assert_eq!(registers.wy, 0x10);
        assert_eq!(registers.wx, 0x07);
    }
}