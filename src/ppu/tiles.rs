// タイルシステム実装

use super::vram::{Vram, TileData, TileAddressingMode, TileMapSelect};

pub struct TileRenderer {
    cache: TileCache,
}

impl TileRenderer {
    pub fn new() -> Self {
        Self {
            cache: TileCache::new(),
        }
    }
    
    // タイルを描画してピクセルデータを取得
    pub fn render_tile(&mut self, 
                      vram: &Vram, 
                      tile_id: u8, 
                      addressing_mode: TileAddressingMode,
                      palette: u8) -> [u8; 8 * 8] {
        
        // キャッシュから取得を試行
        if let Some(cached) = self.cache.get(tile_id, addressing_mode) {
            return self.apply_palette(cached, palette);
        }
        
        // VRAMからタイルデータを読み取り
        let tile_data = vram.read_tile_data(tile_id, addressing_mode);
        
        // ピクセルデータに変換
        let mut pixels = [0u8; 64];
        for y in 0..8 {
            for x in 0..8 {
                pixels[y * 8 + x] = tile_data.pixels[y][x];
            }
        }
        
        // キャッシュに保存
        self.cache.put(tile_id, addressing_mode, pixels);
        
        // パレット適用
        self.apply_palette(pixels, palette)
    }
    
    // パレットを適用してピクセル値を変換
    fn apply_palette(&self, pixels: [u8; 64], palette: u8) -> [u8; 64] {
        let mut result = [0u8; 64];
        
        for (i, &pixel) in pixels.iter().enumerate() {
            result[i] = match pixel & 0x03 {
                0 => palette & 0x03,
                1 => (palette >> 2) & 0x03,
                2 => (palette >> 4) & 0x03,
                3 => (palette >> 6) & 0x03,
                _ => unreachable!(),
            };
        }
        
        result
    }
    
    // キャッシュをクリア
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

// タイルキャッシュ（パフォーマンス向上のため）
struct TileCache {
    entries: Vec<TileCacheEntry>,
    max_entries: usize,
}

#[derive(Clone)]
struct TileCacheEntry {
    tile_id: u8,
    addressing_mode: TileAddressingMode,
    pixels: [u8; 64],
    access_count: u32,
}

impl TileCache {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: 64,  // 最大64タイルをキャッシュ
        }
    }
    
    fn get(&mut self, tile_id: u8, addressing_mode: TileAddressingMode) -> Option<[u8; 64]> {
        for entry in &mut self.entries {
            if entry.tile_id == tile_id && 
               std::mem::discriminant(&entry.addressing_mode) == std::mem::discriminant(&addressing_mode) {
                entry.access_count += 1;
                return Some(entry.pixels);
            }
        }
        None
    }
    
    fn put(&mut self, tile_id: u8, addressing_mode: TileAddressingMode, pixels: [u8; 64]) {
        // 既存エントリがあるか確認
        for entry in &mut self.entries {
            if entry.tile_id == tile_id && 
               std::mem::discriminant(&entry.addressing_mode) == std::mem::discriminant(&addressing_mode) {
                entry.pixels = pixels;
                entry.access_count += 1;
                return;
            }
        }
        
        // 新しいエントリを追加
        if self.entries.len() >= self.max_entries {
            // LRU方式で最も使用頻度の低いエントリを削除
            if let Some(min_index) = self.entries.iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.access_count)
                .map(|(index, _)| index) {
                self.entries.remove(min_index);
            }
        }
        
        self.entries.push(TileCacheEntry {
            tile_id,
            addressing_mode,
            pixels,
            access_count: 1,
        });
    }
    
    fn clear(&mut self) {
        self.entries.clear();
    }
}

// 色変換ユーティリティ
pub struct ColorConverter;

impl ColorConverter {
    // GameBoyの4色グレースケールをRGB888に変換
    pub fn dmg_to_rgb888(color_id: u8) -> (u8, u8, u8) {
        match color_id & 0x03 {
            0 => (0x9B, 0xBC, 0x0F),  // 最明色（緑系）
            1 => (0x8B, 0xAC, 0x0F),  // 明
            2 => (0x30, 0x62, 0x30),  // 暗
            3 => (0x0F, 0x38, 0x0F),  // 最暗色
            _ => unreachable!(),
        }
    }
    
    // GameBoyの4色グレースケールをグレー値に変換
    pub fn dmg_to_gray(color_id: u8) -> u8 {
        match color_id & 0x03 {
            0 => 0xFF,  // 白
            1 => 0xAA,  // 明るいグレー
            2 => 0x55,  // 暗いグレー
            3 => 0x00,  // 黒
            _ => unreachable!(),
        }
    }
}

// デバッグ用タイルビューア
pub struct TileViewer;

impl TileViewer {
    // タイルデータをコンソールに表示
    pub fn print_tile(tile_data: &TileData) {
        let chars = [' ', '░', '▒', '█'];
        println!("┌────────┐");
        for row in &tile_data.pixels {
            print!("│");
            for &pixel in row {
                print!("{}", chars[pixel as usize]);
            }
            println!("│");
        }
        println!("└────────┘");
    }
    
    // パレット情報を表示
    pub fn print_palette(palette: u8) {
        println!("パレット: 0b{:08b}", palette);
        println!("  色0: {} → {}", 0, palette & 0x03);
        println!("  色1: {} → {}", 1, (palette >> 2) & 0x03);
        println!("  色2: {} → {}", 2, (palette >> 4) & 0x03);
        println!("  色3: {} → {}", 3, (palette >> 6) & 0x03);
    }
    
    // タイルマップの一部を表示
    pub fn print_tilemap_region(vram: &Vram, map_select: TileMapSelect, start_x: u8, start_y: u8, width: u8, height: u8) {
        println!("タイルマップ表示 ({:?}, {}x{} @ ({},{}))", map_select, width, height, start_x, start_y);
        
        for y in 0..height {
            for x in 0..width {
                let tile_id = vram.read_tile_map(map_select, start_x + x, start_y + y);
                print!("{:02X} ", tile_id);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::vram::*;
    
    #[test]
    fn test_tile_renderer() {
        let mut vram = Vram::new();
        let mut renderer = TileRenderer::new();
        
        // 簡単なタイルパターンを作成
        let pattern = [
            0b11111111, 0b00000000,  // 行0: 全て色1 (low=1, high=0)
            0b00000000, 0b11111111,  // 行1: 全て色2 (low=0, high=1)
            0b11111111, 0b11111111,  // 行2: 全て色3 (low=1, high=1)
            0b00000000, 0b00000000,  // 行3: 全て色0 (low=0, high=0)
            0b00000000, 0b00000000,  // 行4: 全て色0
            0b00000000, 0b00000000,  // 行5: 全て色0
            0b00000000, 0b00000000,  // 行6: 全て色0
            0b00000000, 0b00000000,  // 行7: 全て色0
        ];
        
        for (i, &byte) in pattern.iter().enumerate() {
            vram.write(i as u16, byte);
        }
        
        // タイルを描画
        let palette = 0b11100100; // 色3→3, 色2→2, 色1→1, 色0→0
        let pixels = renderer.render_tile(&vram, 0, TileAddressingMode::Unsigned, palette);
        
        // パレット適用の確認
        assert_eq!(pixels[0], 1);  // 行0の色1 → パレット値1
        assert_eq!(pixels[8], 2);  // 行1の色2 → パレット値2
        assert_eq!(pixels[16], 3); // 行2の色3 → パレット値3
        assert_eq!(pixels[24], 0); // 行3の色0 → パレット値0
    }
    
    #[test]
    fn test_color_converter() {
        let (r, g, b) = ColorConverter::dmg_to_rgb888(0);
        assert_eq!((r, g, b), (0x9B, 0xBC, 0x0F));
        
        assert_eq!(ColorConverter::dmg_to_gray(0), 0xFF);
        assert_eq!(ColorConverter::dmg_to_gray(3), 0x00);
    }
    
    #[test]
    fn test_tile_cache() {
        let mut cache = TileCache::new();
        let pixels = [42u8; 64];
        
        // キャッシュにエントリなし
        assert!(cache.get(0, TileAddressingMode::Unsigned).is_none());
        
        // エントリを追加
        cache.put(0, TileAddressingMode::Unsigned, pixels);
        
        // キャッシュから取得
        let cached = cache.get(0, TileAddressingMode::Unsigned).unwrap();
        assert_eq!(cached[0], 42);
    }
}