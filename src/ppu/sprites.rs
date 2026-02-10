// src/ppu/sprites.rs
// スプライト（OAM）描画システム
//
// OAMエントリ (4バイト × 40スプライト = 160バイト):
//   Byte 0: Y座標 (画面Y + 16)
//   Byte 1: X座標 (画面X + 8)
//   Byte 2: タイルID
//   Byte 3: フラグ
//     Bit 7: BG/ウィンドウより後ろ (0=前, 1=BGカラー1-3の後ろ)
//     Bit 6: Y反転
//     Bit 5: X反転
//     Bit 4: パレット番号 (0=OBP0, 1=OBP1)
//
// 制約:
//   1スキャンラインあたり最大10スプライト
//   X座標が小さいスプライトが優先（同じ場合はOAMインデックスが小さい方）

use super::vram::Vram;
use super::registers::PpuRegisters;
use super::tiles::ColorConverter;

/// OAMスプライトエントリ
#[derive(Debug, Clone, Copy)]
pub struct SpriteEntry {
    pub y: u8,        // Y座標 (画面Y + 16)
    pub x: u8,        // X座標 (画面X + 8)
    pub tile_id: u8,  // タイルID
    pub flags: u8,    // フラグ
    pub oam_index: u8, // OAMテーブル内のインデックス
}

impl SpriteEntry {
    /// OAMデータからスプライトエントリを作成
    pub fn from_oam(oam: &[u8], index: usize) -> Self {
        let base = index * 4;
        Self {
            y: oam[base],
            x: oam[base + 1],
            tile_id: oam[base + 2],
            flags: oam[base + 3],
            oam_index: index as u8,
        }
    }

    /// 画面上のY座標を取得
    pub fn screen_y(&self) -> i16 {
        self.y as i16 - 16
    }

    /// 画面上のX座標を取得
    pub fn screen_x(&self) -> i16 {
        self.x as i16 - 8
    }

    /// BG/ウィンドウより後ろか
    pub fn is_behind_bg(&self) -> bool {
        self.flags & 0x80 != 0
    }

    /// Y反転
    pub fn is_y_flipped(&self) -> bool {
        self.flags & 0x40 != 0
    }

    /// X反転
    pub fn is_x_flipped(&self) -> bool {
        self.flags & 0x20 != 0
    }

    /// パレット番号 (0=OBP0, 1=OBP1)
    pub fn palette_number(&self) -> u8 {
        (self.flags >> 4) & 0x01
    }

    /// 指定スキャンラインに表示されるか
    pub fn is_on_scanline(&self, scanline: u8, sprite_height: u8) -> bool {
        let screen_y = self.screen_y();
        let ly = scanline as i16;
        ly >= screen_y && ly < screen_y + sprite_height as i16
    }
}

/// スプライトレンダラ
pub struct SpriteRenderer;

impl SpriteRenderer {
    /// OAMスキャン: 指定スキャンラインに表示されるスプライトを収集（最大10個）
    pub fn scan_oam(oam: &[u8; 160], scanline: u8, sprite_height: u8) -> Vec<SpriteEntry> {
        let mut sprites: Vec<SpriteEntry> = Vec::with_capacity(10);

        for i in 0..40 {
            let sprite = SpriteEntry::from_oam(oam, i);
            if sprite.is_on_scanline(scanline, sprite_height) {
                sprites.push(sprite);
                if sprites.len() >= 10 {
                    break; // 1スキャンラインあたり最大10スプライト
                }
            }
        }

        // X座標でソート（小さい方が優先、同じならOAMインデックスが小さい方が優先）
        sprites.sort_by(|a, b| {
            a.x.cmp(&b.x).then(a.oam_index.cmp(&b.oam_index))
        });

        sprites
    }

    /// スキャンラインにスプライトを描画
    /// bg_color_ids: BGの色ID配列（BG優先判定用、160ピクセル）
    /// line_buffer: 出力ラインバッファ (160 * 3 RGB)
    pub fn render_scanline(
        oam: &[u8; 160],
        vram: &Vram,
        registers: &PpuRegisters,
        scanline: u8,
        bg_color_ids: &[u8; 160],
        line_buffer: &mut [u8],
    ) {
        if !registers.is_sprite_enabled() {
            return;
        }

        let sprite_height: u8 = if registers.is_sprite_size_16() { 16 } else { 8 };
        let sprites = Self::scan_oam(oam, scanline, sprite_height);

        // 逆順で描画（低優先度のスプライトから先に描画し、高優先度で上書き）
        for sprite in sprites.iter().rev() {
            let screen_x = sprite.screen_x();
            let screen_y = sprite.screen_y();
            let line_in_sprite = (scanline as i16 - screen_y) as u8;

            // Y反転の処理
            let tile_line = if sprite.is_y_flipped() {
                sprite_height - 1 - line_in_sprite
            } else {
                line_in_sprite
            };

            // 8x16モード時のタイルID調整
            let tile_id = if sprite_height == 16 {
                if tile_line < 8 {
                    sprite.tile_id & 0xFE // 上半分
                } else {
                    sprite.tile_id | 0x01 // 下半分
                }
            } else {
                sprite.tile_id
            };

            let tile_line_in_tile = tile_line % 8;

            // タイルデータ読み出し（スプライトは常に0x8000ベース）
            let tile_addr = (tile_id as u16) * 16 + (tile_line_in_tile as u16) * 2;
            let byte1 = vram.read(tile_addr);
            let byte2 = vram.read(tile_addr + 1);

            // 8ピクセルを描画
            for pixel_x in 0..8 {
                let screen_pixel_x = screen_x + pixel_x as i16;

                // 画面外チェック
                if screen_pixel_x < 0 || screen_pixel_x >= 160 {
                    continue;
                }
                let sx = screen_pixel_x as usize;

                // X反転の処理
                let bit = if sprite.is_x_flipped() {
                    pixel_x
                } else {
                    7 - pixel_x
                };

                let pixel_low = (byte1 >> bit) & 1;
                let pixel_high = (byte2 >> bit) & 1;
                let color_id = pixel_low | (pixel_high << 1);

                // 色ID 0は透明
                if color_id == 0 {
                    continue;
                }

                // BG優先フラグチェック
                if sprite.is_behind_bg() && bg_color_ids[sx] != 0 {
                    continue;
                }

                // パレット適用
                let palette_color = if sprite.palette_number() == 0 {
                    registers.get_obp0_color(color_id)
                } else {
                    registers.get_obp1_color(color_id)
                };

                // RGB変換
                let (r, g, b) = ColorConverter::dmg_to_rgb888(palette_color);
                let idx = sx * 3;
                line_buffer[idx] = r;
                line_buffer[idx + 1] = g;
                line_buffer[idx + 2] = b;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_entry_from_oam() {
        let mut oam = [0u8; 160];
        // スプライト0: Y=32, X=16, TileID=1, Flags=0x00
        oam[0] = 32;
        oam[1] = 16;
        oam[2] = 1;
        oam[3] = 0x00;

        let entry = SpriteEntry::from_oam(&oam, 0);
        assert_eq!(entry.screen_y(), 16);  // 32 - 16
        assert_eq!(entry.screen_x(), 8);   // 16 - 8
        assert_eq!(entry.tile_id, 1);
        assert!(!entry.is_behind_bg());
        assert!(!entry.is_y_flipped());
        assert!(!entry.is_x_flipped());
        assert_eq!(entry.palette_number(), 0);
    }

    #[test]
    fn test_sprite_flags() {
        let mut oam = [0u8; 160];
        oam[3] = 0xF0; // BG優先=1, Y反転=1, X反転=1, パレット1

        let entry = SpriteEntry::from_oam(&oam, 0);
        assert!(entry.is_behind_bg());
        assert!(entry.is_y_flipped());
        assert!(entry.is_x_flipped());
        assert_eq!(entry.palette_number(), 1);
    }

    #[test]
    fn test_sprite_on_scanline_8x8() {
        let mut oam = [0u8; 160];
        oam[0] = 24; // screen_y = 8

        let entry = SpriteEntry::from_oam(&oam, 0);
        assert!(!entry.is_on_scanline(7, 8));  // 7 < 8
        assert!(entry.is_on_scanline(8, 8));   // 8 >= 8 && 8 < 16
        assert!(entry.is_on_scanline(15, 8));  // 15 >= 8 && 15 < 16
        assert!(!entry.is_on_scanline(16, 8)); // 16 >= 16
    }

    #[test]
    fn test_sprite_on_scanline_8x16() {
        let mut oam = [0u8; 160];
        oam[0] = 24; // screen_y = 8

        let entry = SpriteEntry::from_oam(&oam, 0);
        assert!(entry.is_on_scanline(8, 16));  // 8 >= 8 && 8 < 24
        assert!(entry.is_on_scanline(23, 16)); // 23 >= 8 && 23 < 24
        assert!(!entry.is_on_scanline(24, 16));
    }

    #[test]
    fn test_oam_scan_max_10() {
        let mut oam = [0u8; 160];
        // 15個のスプライトをスキャンライン0に配置
        for i in 0..15 {
            oam[i * 4] = 16;     // screen_y = 0
            oam[i * 4 + 1] = (i as u8 + 1) * 8 + 8;
        }

        let sprites = SpriteRenderer::scan_oam(&oam, 0, 8);
        assert_eq!(sprites.len(), 10); // 最大10個
    }

    #[test]
    fn test_oam_scan_sorting() {
        let mut oam = [0u8; 160];
        // スプライト0: X=40
        oam[0] = 16; oam[1] = 40;
        // スプライト1: X=20
        oam[4] = 16; oam[5] = 20;
        // スプライト2: X=30
        oam[8] = 16; oam[9] = 30;

        let sprites = SpriteRenderer::scan_oam(&oam, 0, 8);
        assert_eq!(sprites.len(), 3);
        assert_eq!(sprites[0].x, 20); // X座標順
        assert_eq!(sprites[1].x, 30);
        assert_eq!(sprites[2].x, 40);
    }

    #[test]
    fn test_sprite_transparent_color() {
        let mut oam = [0u8; 160];
        let vram = Vram::new(); // 全て0 → 色ID=0（透明）
        let mut registers = PpuRegisters::new();
        registers.lcdc = 0x93; // スプライト有効
        registers.obp0 = 0xE4;

        // スプライトをスキャンライン0に配置
        oam[0] = 16; // screen_y = 0
        oam[1] = 8;  // screen_x = 0
        oam[2] = 0;  // tile_id = 0

        let bg_colors = [0u8; 160];
        let mut line_buffer = [0u8; 160 * 3];

        // 全タイルデータが0なので、色IDは全て0（透明）で描画されない
        SpriteRenderer::render_scanline(
            &oam, &vram, &registers, 0, &bg_colors, &mut line_buffer,
        );

        // 透明なので変更されない
        assert_eq!(line_buffer[0], 0);
    }

    #[test]
    fn test_sprite_rendering_basic() {
        let mut oam = [0u8; 160];
        let mut vram = Vram::new();
        let mut registers = PpuRegisters::new();
        registers.lcdc = 0x93; // スプライト有効
        registers.obp0 = 0xE4; // パレット: 3,2,1,0

        // スプライトをスキャンライン0に配置
        oam[0] = 16; // screen_y = 0
        oam[1] = 8;  // screen_x = 0
        oam[2] = 0;  // tile_id = 0

        // タイル0のデータ: 最初のライン = 0xFF, 0x00 → 色ID 1（全ピクセル）
        vram.write(0x0000, 0xFF);
        vram.write(0x0001, 0x00);

        let bg_colors = [0u8; 160];
        let mut line_buffer = [0u8; 160 * 3];

        SpriteRenderer::render_scanline(
            &oam, &vram, &registers, 0, &bg_colors, &mut line_buffer,
        );

        // 色ID=1、OBP0=0xE4 → パレット色1
        let (r, g, b) = ColorConverter::dmg_to_rgb888(1);
        assert_eq!(line_buffer[0], r);
        assert_eq!(line_buffer[1], g);
        assert_eq!(line_buffer[2], b);
    }
}
