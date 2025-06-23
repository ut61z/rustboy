// 背景描画システム

use super::vram::{Vram, TileAddressingMode, TileMapSelect};
use super::tiles::{TileRenderer, ColorConverter};
use super::registers::PpuRegisters;

pub struct BackgroundRenderer {
    tile_renderer: TileRenderer,
}

impl BackgroundRenderer {
    pub fn new() -> Self {
        Self {
            tile_renderer: TileRenderer::new(),
        }
    }
    
    // 背景スキャンライン（160ピクセル）を描画
    pub fn render_scanline(&mut self, 
                          vram: &Vram, 
                          registers: &PpuRegisters, 
                          scanline: u8) -> [u8; 160 * 3] {
        let mut line_buffer = [0u8; 160 * 3];
        
        if !registers.is_bg_enabled() {
            // BG無効時は白で塗りつぶし
            for i in (0..480).step_by(3) {
                let (r, g, b) = ColorConverter::dmg_to_rgb888(0);
                line_buffer[i] = r;
                line_buffer[i + 1] = g;
                line_buffer[i + 2] = b;
            }
            return line_buffer;
        }
        
        // スクロール補正されたY座標
        let bg_y = scanline.wrapping_add(registers.scy);
        let tile_y = bg_y / 8;          // タイル行
        let pixel_y = bg_y % 8;         // タイル内Y座標
        
        // タイルマップ選択
        let tilemap_select = if registers.is_bg_tilemap_high() {
            TileMapSelect::Map1
        } else {
            TileMapSelect::Map0
        };
        
        // タイルデータアドレス指定モード
        let addressing_mode = if registers.is_bg_window_tiledata_high() {
            TileAddressingMode::Unsigned
        } else {
            TileAddressingMode::Signed
        };
        
        // 160ピクセルを描画
        for pixel_x in 0..160 {
            // スクロール補正されたX座標
            let bg_x = (pixel_x as u8).wrapping_add(registers.scx);
            let tile_x = bg_x / 8;          // タイル列
            let pixel_x_in_tile = bg_x % 8; // タイル内X座標
            
            // タイルIDを取得
            let tile_id = vram.read_tile_map(tilemap_select, tile_x, tile_y);
            
            // タイル描画（キャッシュ活用）
            let tile_pixels = self.tile_renderer.render_tile(
                vram, 
                tile_id, 
                addressing_mode, 
                registers.bgp
            );
            
            // ピクセル値を取得
            let pixel_index = (pixel_y as usize) * 8 + (pixel_x_in_tile as usize);
            let color_id = tile_pixels[pixel_index];
            
            // RGB変換
            let (r, g, b) = ColorConverter::dmg_to_rgb888(color_id);
            let buffer_index = pixel_x * 3;
            line_buffer[buffer_index] = r;
            line_buffer[buffer_index + 1] = g;
            line_buffer[buffer_index + 2] = b;
        }
        
        line_buffer
    }
    
    // 背景全体を描画（デバッグ用）
    pub fn render_full_background(&mut self, 
                                 vram: &Vram, 
                                 registers: &PpuRegisters) -> [u8; 256 * 256 * 3] {
        let mut buffer = [0u8; 256 * 256 * 3];
        
        if !registers.is_bg_enabled() {
            return buffer;
        }
        
        let tilemap_select = if registers.is_bg_tilemap_high() {
            TileMapSelect::Map1
        } else {
            TileMapSelect::Map0
        };
        
        let addressing_mode = if registers.is_bg_window_tiledata_high() {
            TileAddressingMode::Unsigned
        } else {
            TileAddressingMode::Signed
        };
        
        // 32x32タイルマップを描画
        for tile_y in 0..32 {
            for tile_x in 0..32 {
                let tile_id = vram.read_tile_map(tilemap_select, tile_x, tile_y);
                let tile_pixels = self.tile_renderer.render_tile(
                    vram, 
                    tile_id, 
                    addressing_mode, 
                    registers.bgp
                );
                
                // 8x8ピクセルのタイルを256x256バッファに転送
                for y in 0..8 {
                    for x in 0..8 {
                        let pixel_x = tile_x as usize * 8 + x;
                        let pixel_y = tile_y as usize * 8 + y;
                        let buffer_index = (pixel_y * 256 + pixel_x) * 3;
                        let tile_index = y * 8 + x;
                        
                        let color_id = tile_pixels[tile_index];
                        let (r, g, b) = ColorConverter::dmg_to_rgb888(color_id);
                        
                        buffer[buffer_index] = r;
                        buffer[buffer_index + 1] = g;
                        buffer[buffer_index + 2] = b;
                    }
                }
            }
        }
        
        buffer
    }
    
    // スキャンライン上の特定ピクセルの色を取得
    pub fn get_pixel_color(&mut self, 
                          vram: &Vram, 
                          registers: &PpuRegisters, 
                          screen_x: u8, 
                          screen_y: u8) -> u8 {
        if !registers.is_bg_enabled() {
            return 0;
        }
        
        let bg_x = screen_x.wrapping_add(registers.scx);
        let bg_y = screen_y.wrapping_add(registers.scy);
        
        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;
        
        let tilemap_select = if registers.is_bg_tilemap_high() {
            TileMapSelect::Map1
        } else {
            TileMapSelect::Map0
        };
        
        let addressing_mode = if registers.is_bg_window_tiledata_high() {
            TileAddressingMode::Unsigned
        } else {
            TileAddressingMode::Signed
        };
        
        let tile_id = vram.read_tile_map(tilemap_select, tile_x, tile_y);
        let tile_pixels = self.tile_renderer.render_tile(
            vram, 
            tile_id, 
            addressing_mode, 
            registers.bgp
        );
        
        tile_pixels[pixel_y as usize * 8 + pixel_x as usize]
    }
    
    // キャッシュクリア
    pub fn clear_cache(&mut self) {
        self.tile_renderer.clear_cache();
    }
}

// 背景スクロール情報
#[derive(Debug, Clone, Copy)]
pub struct ScrollInfo {
    pub scx: u8,
    pub scy: u8,
}

impl ScrollInfo {
    pub fn new(scx: u8, scy: u8) -> Self {
        Self { scx, scy }
    }
    
    // スクリーン座標を背景座標に変換
    pub fn screen_to_bg(&self, screen_x: u8, screen_y: u8) -> (u8, u8) {
        (
            screen_x.wrapping_add(self.scx),
            screen_y.wrapping_add(self.scy)
        )
    }
    
    // 背景座標をタイル座標に変換
    pub fn bg_to_tile(&self, bg_x: u8, bg_y: u8) -> (u8, u8, u8, u8) {
        let tile_x = bg_x / 8;
        let tile_y = bg_y / 8;
        let pixel_x = bg_x % 8;
        let pixel_y = bg_y % 8;
        
        (tile_x, tile_y, pixel_x, pixel_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vram::Vram;
    use super::super::registers::PpuRegisters;
    
    #[test]
    fn test_background_renderer_creation() {
        let _renderer = BackgroundRenderer::new();
    }
    
    #[test]
    fn test_bg_disabled_rendering() {
        let mut renderer = BackgroundRenderer::new();
        let vram = Vram::new();
        let mut registers = PpuRegisters::new();
        
        // BG無効
        registers.lcdc = 0x80;  // LCD有効、BG無効
        
        let line = renderer.render_scanline(&vram, &registers, 0);
        
        // 全て白色（色0）になることを確認
        let (r, g, b) = ColorConverter::dmg_to_rgb888(0);
        assert_eq!(line[0], r);
        assert_eq!(line[1], g);
        assert_eq!(line[2], b);
    }
    
    #[test]
    fn test_scroll_info() {
        let scroll = ScrollInfo::new(8, 16);
        
        let (bg_x, bg_y) = scroll.screen_to_bg(0, 0);
        assert_eq!(bg_x, 8);
        assert_eq!(bg_y, 16);
        
        let (tile_x, tile_y, pixel_x, pixel_y) = scroll.bg_to_tile(bg_x, bg_y);
        assert_eq!(tile_x, 1);    // 8 / 8
        assert_eq!(tile_y, 2);    // 16 / 8
        assert_eq!(pixel_x, 0);   // 8 % 8
        assert_eq!(pixel_y, 0);   // 16 % 8
    }
    
    #[test]
    fn test_scrolling_wrap_around() {
        let scroll = ScrollInfo::new(255, 255);
        
        // オーバーフローのテスト
        let (bg_x, bg_y) = scroll.screen_to_bg(1, 1);
        assert_eq!(bg_x, 0);  // 1 + 255 = 256 -> 0 (u8のラップアラウンド)
        assert_eq!(bg_y, 0);  // 1 + 255 = 256 -> 0 (u8のラップアラウンド)
    }
}