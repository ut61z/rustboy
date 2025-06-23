#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub y: u8,          // Y座標 (実際の表示位置は y - 16)
    pub x: u8,          // X座標 (実際の表示位置は x - 8)
    pub tile_index: u8, // タイルインデックス
    pub flags: u8,      // 属性フラグ
}

impl Sprite {
    /// OAMの4バイトからスプライトを作成
    pub fn from_oam_bytes(bytes: &[u8; 4]) -> Self {
        Self {
            y: bytes[0],
            x: bytes[1],
            tile_index: bytes[2],
            flags: bytes[3],
        }
    }
    
    /// スプライトが画面上に表示されるかチェック
    pub fn is_visible(&self) -> bool {
        self.y != 0 && self.x != 0 && self.y < 160 && self.x < 168
    }
    
    /// 実際の画面座標を取得（オフセット適用済み）
    pub fn screen_x(&self) -> i16 {
        self.x as i16 - 8
    }
    
    pub fn screen_y(&self) -> i16 {
        self.y as i16 - 16
    }
    
    /// 指定されたスキャンラインに表示されるかチェック
    pub fn is_on_scanline(&self, scanline: u8, sprite_height: u8) -> bool {
        if !self.is_visible() {
            return false;
        }
        
        let sprite_y = self.screen_y();
        let scanline_i16 = scanline as i16;
        
        sprite_y <= scanline_i16 && scanline_i16 < sprite_y + sprite_height as i16
    }
    
    /// 属性フラグの解析
    pub fn has_bg_priority(&self) -> bool {
        (self.flags & 0x80) != 0
    }
    
    pub fn is_y_flipped(&self) -> bool {
        (self.flags & 0x40) != 0
    }
    
    pub fn is_x_flipped(&self) -> bool {
        (self.flags & 0x20) != 0
    }
    
    pub fn palette_number(&self) -> u8 {
        if (self.flags & 0x10) != 0 { 1 } else { 0 }
    }
}

pub struct SpriteRenderer {
    sprites: [Sprite; 40],
}

impl SpriteRenderer {
    pub fn new() -> Self {
        Self {
            sprites: [Sprite {
                y: 0,
                x: 0,
                tile_index: 0,
                flags: 0,
            }; 40],
        }
    }
    
    /// OAMデータから全スプライトを解析
    pub fn parse_oam(&mut self, oam: &[u8; 160]) {
        for i in 0..40 {
            let base_addr = i * 4;
            let sprite_bytes = [
                oam[base_addr],
                oam[base_addr + 1],
                oam[base_addr + 2],
                oam[base_addr + 3],
            ];
            self.sprites[i] = Sprite::from_oam_bytes(&sprite_bytes);
        }
    }
    
    /// 指定されたスキャンラインに表示されるスプライトを検索
    /// GameBoyハードウェア仕様に従った優先度付け
    pub fn find_sprites_on_scanline(&self, scanline: u8, sprite_height: u8) -> Vec<(usize, Sprite)> {
        let mut line_sprites = Vec::new();
        
        // OAMインデックス順で検索（ハードウェア動作に合わせる）
        for (index, &sprite) in self.sprites.iter().enumerate() {
            if sprite.is_on_scanline(scanline, sprite_height) {
                line_sprites.push((index, sprite));
                
                // GameBoyは1ラインに最大10スプライトまで
                if line_sprites.len() >= 10 {
                    break;
                }
            }
        }
        
        // X座標でソート（優先度決定）
        // GameBoy DMGでは、X座標が小さいほど高優先度
        // 同じX座標の場合はOAMインデックスが小さいほど高優先度
        line_sprites.sort_by(|a, b| {
            let x_cmp = a.1.x.cmp(&b.1.x);
            if x_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                x_cmp
            }
        });
        
        line_sprites
    }
    
    /// より効率的なスプライト検索（キャッシュ最適化版）
    pub fn find_sprites_on_scanline_optimized(&self, scanline: u8, sprite_height: u8) -> Vec<(usize, Sprite)> {
        let mut line_sprites = Vec::with_capacity(10); // 最大10スプライト
        
        for (index, &sprite) in self.sprites.iter().enumerate() {
            // 早期終了条件
            if line_sprites.len() >= 10 {
                break;
            }
            
            // 可視性チェック（最適化）
            if sprite.y == 0 || sprite.x == 0 {
                continue;
            }
            
            let sprite_y = sprite.screen_y();
            let scanline_i16 = scanline as i16;
            
            // スキャンライン範囲チェック
            if sprite_y <= scanline_i16 && scanline_i16 < sprite_y + sprite_height as i16 {
                line_sprites.push((index, sprite));
            }
        }
        
        // 優先度ソート
        line_sprites.sort_unstable_by(|a, b| {
            let x_cmp = a.1.x.cmp(&b.1.x);
            if x_cmp == std::cmp::Ordering::Equal {
                a.0.cmp(&b.0)
            } else {
                x_cmp
            }
        });
        
        line_sprites
    }
    
    /// スプライトタイルの1行分のピクセルデータを取得
    /// Returns: [color_id; 8] (0=透明, 1-3=パレット色)
    pub fn render_sprite_line(&self, sprite: &Sprite, scanline: u8, sprite_height: u8, vram: &crate::ppu::vram::Vram) -> [u8; 8] {
        let mut pixels = [0u8; 8];
        
        let sprite_y = sprite.screen_y();
        let line_in_sprite = scanline as i16 - sprite_y;
        
        // スプライト範囲外チェック
        if line_in_sprite < 0 || line_in_sprite >= sprite_height as i16 {
            return pixels;
        }
        
        // Y flip処理
        let actual_line = if sprite.is_y_flipped() {
            (sprite_height - 1) - line_in_sprite as u8
        } else {
            line_in_sprite as u8
        };
        
        // タイルインデックス計算（8x16モード対応）
        let tile_index = if sprite_height == 16 {
            // 8x16モード: 偶数インデックス（上半分）、奇数インデックス（下半分）
            if actual_line < 8 {
                sprite.tile_index & 0xFE  // 偶数にする
            } else {
                sprite.tile_index | 0x01  // 奇数にする
            }
        } else {
            // 8x8モード
            sprite.tile_index
        };
        
        // タイル内の行計算
        let tile_line = if sprite_height == 16 {
            actual_line % 8
        } else {
            actual_line
        };
        
        // タイルデータアドレス計算（スプライトは常に$8000-$8FFFから読み込み）
        let tile_addr = (tile_index as u16) * 16 + (tile_line as u16) * 2;
        
        // 2bppタイルデータ読み込み
        let byte1 = vram.read(tile_addr);
        let byte2 = vram.read(tile_addr + 1);
        
        // 8ピクセル分のデータを展開
        for x in 0..8 {
            let bit = 7 - x;
            let pixel_low = (byte1 >> bit) & 1;
            let pixel_high = (byte2 >> bit) & 1;
            let color_id = pixel_low | (pixel_high << 1);
            
            // X flip処理
            let actual_x = if sprite.is_x_flipped() {
                7 - x
            } else {
                x
            };
            
            pixels[actual_x] = color_id;
        }
        
        pixels
    }
    
    /// スプライトの1ピクセルをフレームバッファに描画
    /// Returns: true if pixel was drawn (not transparent)
    pub fn draw_sprite_pixel(&self, 
                           framebuffer: &mut [u8], 
                           screen_x: usize, 
                           screen_y: usize, 
                           color_id: u8, 
                           palette_number: u8,
                           obp0: u8, 
                           obp1: u8) -> bool {
        // 透明ピクセル（色0）はスキップ
        if color_id == 0 {
            return false;
        }
        
        // 画面範囲チェック
        if screen_x >= 160 || screen_y >= 144 {
            return false;
        }
        
        // パレット選択
        let palette = if palette_number == 0 { obp0 } else { obp1 };
        
        // パレット色を取得
        let palette_color = match color_id {
            1 => (palette >> 2) & 0x03,
            2 => (palette >> 4) & 0x03,
            3 => (palette >> 6) & 0x03,
            _ => 0,
        };
        
        // RGB変換
        let (r, g, b) = match palette_color {
            0 => (0x9B, 0xBC, 0x0F),  // 最明色（緑系）
            1 => (0x8B, 0xAC, 0x0F),  // 明
            2 => (0x30, 0x62, 0x30),  // 暗
            3 => (0x0F, 0x38, 0x0F),  // 最暗色
            _ => (0x9B, 0xBC, 0x0F),
        };
        
        // フレームバッファに書き込み
        let pixel_index = (screen_y * 160 + screen_x) * 3;
        if pixel_index + 2 < framebuffer.len() {
            framebuffer[pixel_index] = r;
            framebuffer[pixel_index + 1] = g;
            framebuffer[pixel_index + 2] = b;
            return true;
        }
        
        false
    }
    
    /// スキャンライン全体のスプライト描画
    pub fn render_sprites_on_scanline(&self, 
                                    scanline: u8, 
                                    sprite_height: u8,
                                    framebuffer: &mut [u8],
                                    vram: &crate::ppu::vram::Vram,
                                    obp0: u8,
                                    obp1: u8,
                                    bg_pixels: Option<&[u8; 160]>) -> u8 {
        let mut sprites_drawn = 0;
        
        // 現在のスキャンラインのスプライトを取得
        let line_sprites = self.find_sprites_on_scanline(scanline, sprite_height);
        
        // 逆順で描画（優先度の低いスプライトから先に描画）
        for (sprite_index, sprite) in line_sprites.iter().rev() {
            let sprite_pixels = self.render_sprite_line(sprite, scanline, sprite_height, vram);
            let sprite_screen_x = sprite.screen_x();
            
            // スプライトの8ピクセルを描画
            for (pixel_x, &color_id) in sprite_pixels.iter().enumerate() {
                let screen_x = sprite_screen_x + pixel_x as i16;
                
                // 画面範囲チェック
                if screen_x < 0 || screen_x >= 160 {
                    continue;
                }
                
                let screen_x_usize = screen_x as usize;
                
                // BG優先度チェック
                if sprite.has_bg_priority() {
                    if let Some(bg_pixels) = bg_pixels {
                        // BG色が0でない場合、スプライトを描画しない
                        if bg_pixels[screen_x_usize] != 0 {
                            continue;
                        }
                    }
                }
                
                // ピクセル描画
                if self.draw_sprite_pixel(
                    framebuffer,
                    screen_x_usize,
                    scanline as usize,
                    color_id,
                    sprite.palette_number(),
                    obp0,
                    obp1
                ) {
                    sprites_drawn += 1;
                }
            }
        }
        
        sprites_drawn
    }
    
    /// デバッグ用：アクティブなスプライトを表示
    pub fn debug_active_sprites(&self) {
        println!("=== Active Sprites ===");
        for (i, sprite) in self.sprites.iter().enumerate() {
            if sprite.is_visible() {
                println!("Sprite {}: X={}, Y={}, Tile={:02X}, Flags={:02X}", 
                         i, sprite.x, sprite.y, sprite.tile_index, sprite.flags);
                println!("  Screen pos: ({}, {}), BG Priority: {}, Flips: (X:{}, Y:{}), Palette: {}", 
                         sprite.screen_x(), sprite.screen_y(),
                         sprite.has_bg_priority(),
                         sprite.is_x_flipped(), sprite.is_y_flipped(),
                         sprite.palette_number());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sprite_creation() {
        let sprite_data = [80, 88, 0x01, 0x00]; // Y=80, X=88, Tile=1, Flags=0
        let sprite = Sprite::from_oam_bytes(&sprite_data);
        
        assert_eq!(sprite.y, 80);
        assert_eq!(sprite.x, 88);
        assert_eq!(sprite.tile_index, 0x01);
        assert_eq!(sprite.flags, 0x00);
    }
    
    #[test]
    fn test_sprite_visibility() {
        // 可視スプライト
        let visible_sprite = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x00]);
        assert!(visible_sprite.is_visible());
        
        // 非可視スプライト (Y=0)
        let invisible_sprite = Sprite::from_oam_bytes(&[0, 88, 0x01, 0x00]);
        assert!(!invisible_sprite.is_visible());
    }
    
    #[test]
    fn test_sprite_screen_coordinates() {
        let sprite = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x00]);
        assert_eq!(sprite.screen_x(), 80); // 88 - 8
        assert_eq!(sprite.screen_y(), 64); // 80 - 16
    }
    
    #[test]
    fn test_sprite_scanline_detection() {
        let sprite = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x00]); // screen_y = 64
        
        // 8x8スプライト
        assert!(sprite.is_on_scanline(64, 8)); // スプライトの一番上
        assert!(sprite.is_on_scanline(71, 8)); // スプライトの一番下
        assert!(!sprite.is_on_scanline(63, 8)); // スプライトの上
        assert!(!sprite.is_on_scanline(72, 8)); // スプライトの下
    }
    
    #[test]
    fn test_sprite_flags() {
        let sprite = Sprite::from_oam_bytes(&[80, 88, 0x01, 0xF0]); // 全フラグON
        
        assert!(sprite.has_bg_priority());
        assert!(sprite.is_y_flipped());
        assert!(sprite.is_x_flipped());
        assert_eq!(sprite.palette_number(), 1);
    }
    
    #[test]
    fn test_oam_parsing() {
        let mut renderer = SpriteRenderer::new();
        let mut oam = [0u8; 160];
        
        // テスト用スプライトデータ
        oam[0] = 80;  // Sprite 0 Y
        oam[1] = 88;  // Sprite 0 X
        oam[2] = 0x01; // Sprite 0 Tile
        oam[3] = 0x00; // Sprite 0 Flags
        
        oam[4] = 100; // Sprite 1 Y
        oam[5] = 120; // Sprite 1 X
        oam[6] = 0x02; // Sprite 1 Tile
        oam[7] = 0x20; // Sprite 1 Flags (X flip)
        
        renderer.parse_oam(&oam);
        
        assert_eq!(renderer.sprites[0].y, 80);
        assert_eq!(renderer.sprites[0].x, 88);
        assert_eq!(renderer.sprites[1].y, 100);
        assert_eq!(renderer.sprites[1].x, 120);
        assert!(renderer.sprites[1].is_x_flipped());
    }
    
    #[test]
    fn test_scanline_sprite_finding() {
        let mut renderer = SpriteRenderer::new();
        let mut oam = [0u8; 160];
        
        // スプライト0: screen_y = 64 (Y=80)
        oam[0] = 80;
        oam[1] = 88;
        oam[2] = 0x01;
        oam[3] = 0x00;
        
        // スプライト1: screen_y = 84 (Y=100)
        oam[4] = 100;
        oam[5] = 120;
        oam[6] = 0x02;
        oam[7] = 0x00;
        
        renderer.parse_oam(&oam);
        
        // スキャンライン64: スプライト0のみ
        let sprites_line_64 = renderer.find_sprites_on_scanline(64, 8);
        assert_eq!(sprites_line_64.len(), 1);
        assert_eq!(sprites_line_64[0].0, 0); // スプライト0
        
        // スキャンライン84: スプライト1のみ
        let sprites_line_84 = renderer.find_sprites_on_scanline(84, 8);
        assert_eq!(sprites_line_84.len(), 1);
        assert_eq!(sprites_line_84[0].0, 1); // スプライト1
        
        // スキャンライン50: スプライトなし
        let sprites_line_50 = renderer.find_sprites_on_scanline(50, 8);
        assert_eq!(sprites_line_50.len(), 0);
    }
    
    #[test]
    fn test_sprite_priority_sorting() {
        let mut renderer = SpriteRenderer::new();
        let mut oam = [0u8; 160];
        
        // 同じスキャンライン上に複数のスプライト
        // スプライト0: X=100, OAMインデックス=0
        oam[0] = 80;   // Y
        oam[1] = 100;  // X
        oam[2] = 0x01; // Tile
        oam[3] = 0x00; // Flags
        
        // スプライト1: X=80, OAMインデックス=1 (より左、高優先度)
        oam[4] = 80;   // Y
        oam[5] = 80;   // X
        oam[6] = 0x02; // Tile
        oam[7] = 0x00; // Flags
        
        // スプライト2: X=120, OAMインデックス=2 (最も右、低優先度)
        oam[8] = 80;   // Y
        oam[9] = 120;  // X
        oam[10] = 0x03; // Tile
        oam[11] = 0x00; // Flags
        
        renderer.parse_oam(&oam);
        
        let sprites = renderer.find_sprites_on_scanline(64, 8); // screen_y = 64
        assert_eq!(sprites.len(), 3);
        
        // X座標順にソートされているかチェック
        assert_eq!(sprites[0].0, 1); // スプライト1 (X=80)
        assert_eq!(sprites[1].0, 0); // スプライト0 (X=100)
        assert_eq!(sprites[2].0, 2); // スプライト2 (X=120)
    }
    
    #[test]
    fn test_sprite_same_x_priority() {
        let mut renderer = SpriteRenderer::new();
        let mut oam = [0u8; 160];
        
        // 同じX座標のスプライト（OAMインデックス順で優先度決定）
        // スプライト2: X=100, OAMインデックス=2
        oam[8] = 80;   // Y
        oam[9] = 100;  // X
        oam[10] = 0x03; // Tile
        oam[11] = 0x00; // Flags
        
        // スプライト5: X=100, OAMインデックス=5 (同じX座標、低優先度)
        oam[20] = 80;   // Y
        oam[21] = 100;  // X
        oam[22] = 0x06; // Tile
        oam[23] = 0x00; // Flags
        
        renderer.parse_oam(&oam);
        
        let sprites = renderer.find_sprites_on_scanline(64, 8);
        assert_eq!(sprites.len(), 2);
        
        // 同じX座標の場合、OAMインデックスが小さいほうが優先
        assert_eq!(sprites[0].0, 2); // スプライト2 (OAMインデックス小)
        assert_eq!(sprites[1].0, 5); // スプライト5 (OAMインデックス大)
    }
    
    #[test]
    fn test_ten_sprite_limit() {
        let mut renderer = SpriteRenderer::new();
        let mut oam = [0u8; 160];
        
        // 同じスキャンラインに15個のスプライトを配置
        for i in 0..15 {
            let base = i * 4;
            oam[base] = 80;         // Y (screen_y = 64)
            oam[base + 1] = 8 + i as u8; // X (8, 9, 10, ..., 22)
            oam[base + 2] = i as u8;     // Tile
            oam[base + 3] = 0;           // Flags
        }
        
        renderer.parse_oam(&oam);
        
        let sprites = renderer.find_sprites_on_scanline(64, 8);
        
        // 最大10スプライトまでしか返されない
        assert_eq!(sprites.len(), 10);
        
        // OAMインデックス順で最初の10個が選ばれる
        for i in 0..10 {
            assert_eq!(sprites[i].0, i);
        }
    }
    
    #[test]
    fn test_optimized_sprite_finding() {
        let mut renderer = SpriteRenderer::new();
        let mut oam = [0u8; 160];
        
        // テスト用スプライト
        oam[0] = 80;
        oam[1] = 88;
        oam[2] = 0x01;
        oam[3] = 0x00;
        
        renderer.parse_oam(&oam);
        
        // 通常版と最適化版で同じ結果が得られるかテスト
        let normal = renderer.find_sprites_on_scanline(64, 8);
        let optimized = renderer.find_sprites_on_scanline_optimized(64, 8);
        
        assert_eq!(normal.len(), optimized.len());
        for (n, o) in normal.iter().zip(optimized.iter()) {
            assert_eq!(n.0, o.0);
            assert_eq!(n.1.x, o.1.x);
            assert_eq!(n.1.y, o.1.y);
        }
    }
    
    #[test]
    fn test_sprite_line_rendering() {
        let renderer = SpriteRenderer::new();
        let mut vram = crate::ppu::vram::Vram::new();
        
        // テスト用タイルデータ（タイル1）
        // 簡単なパターン: 上半分が色1、下半分が色2
        let tile_data = [
            0xFF, 0x00, // 行0: 11111111, 00000000 -> 全て色1
            0xFF, 0x00, // 行1: 11111111, 00000000 -> 全て色1
            0xFF, 0x00, // 行2: 11111111, 00000000 -> 全て色1
            0xFF, 0x00, // 行3: 11111111, 00000000 -> 全て色1
            0xFF, 0xFF, // 行4: 11111111, 11111111 -> 全て色3
            0xFF, 0xFF, // 行5: 11111111, 11111111 -> 全て色3
            0xFF, 0xFF, // 行6: 11111111, 11111111 -> 全て色3
            0xFF, 0xFF, // 行7: 11111111, 11111111 -> 全て色3
        ];
        
        // タイル1のデータを書き込み
        for (i, &byte) in tile_data.iter().enumerate() {
            vram.write(16 + i as u16, byte); // タイル1は16バイト目から
        }
        
        let sprite = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x00]); // Y=80, X=88, Tile=1, Flags=0
        
        // スキャンライン64（スプライト上半分）
        let pixels = renderer.render_sprite_line(&sprite, 64, 8, &vram);
        assert_eq!(pixels, [1, 1, 1, 1, 1, 1, 1, 1]); // 全て色1
        
        // スキャンライン68（スプライト下半分）
        let pixels = renderer.render_sprite_line(&sprite, 68, 8, &vram);
        assert_eq!(pixels, [3, 3, 3, 3, 3, 3, 3, 3]); // 全て色3
    }
    
    #[test]
    fn test_sprite_x_flip() {
        let renderer = SpriteRenderer::new();
        let mut vram = crate::ppu::vram::Vram::new();
        
        // テスト用タイルデータ（タイル1）
        // パターン: 左半分が色1、右半分が色2
        let tile_data = [
            0xF0, 0x0F, // 行0: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行1: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行2: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行3: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行4: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行5: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行6: 11110000, 00001111 -> 1111 2222
            0xF0, 0x0F, // 行7: 11110000, 00001111 -> 1111 2222
        ];
        
        for (i, &byte) in tile_data.iter().enumerate() {
            vram.write(16 + i as u16, byte);
        }
        
        // 通常スプライト
        let sprite_normal = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x00]);
        let pixels_normal = renderer.render_sprite_line(&sprite_normal, 64, 8, &vram);
        assert_eq!(pixels_normal, [1, 1, 1, 1, 2, 2, 2, 2]);
        
        // X flippedスプライト
        let sprite_flipped = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x20]); // X flip flag
        let pixels_flipped = renderer.render_sprite_line(&sprite_flipped, 64, 8, &vram);
        assert_eq!(pixels_flipped, [2, 2, 2, 2, 1, 1, 1, 1]); // 左右反転
    }
    
    #[test]
    fn test_sprite_y_flip() {
        let renderer = SpriteRenderer::new();
        let mut vram = crate::ppu::vram::Vram::new();
        
        // テスト用タイルデータ（タイル1）
        // パターン: 行ごとに異なる色
        let tile_data = [
            0xFF, 0x00, // 行0: 色1
            0x00, 0xFF, // 行1: 色2
            0xFF, 0xFF, // 行2: 色3
            0x00, 0x00, // 行3: 色0
            0xFF, 0x00, // 行4: 色1
            0x00, 0xFF, // 行5: 色2
            0xFF, 0xFF, // 行6: 色3
            0x00, 0x00, // 行7: 色0
        ];
        
        for (i, &byte) in tile_data.iter().enumerate() {
            vram.write(16 + i as u16, byte);
        }
        
        // 通常スプライト（行0）
        let sprite_normal = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x00]);
        let pixels_normal = renderer.render_sprite_line(&sprite_normal, 64, 8, &vram);
        assert_eq!(pixels_normal, [1, 1, 1, 1, 1, 1, 1, 1]); // 行0 = 色1
        
        // Y flippedスプライト（行0だが実際は行7）
        let sprite_flipped = Sprite::from_oam_bytes(&[80, 88, 0x01, 0x40]); // Y flip flag
        let pixels_flipped = renderer.render_sprite_line(&sprite_flipped, 64, 8, &vram);
        assert_eq!(pixels_flipped, [0, 0, 0, 0, 0, 0, 0, 0]); // 行7 = 色0
    }
    
    #[test]
    fn test_sprite_pixel_drawing() {
        let renderer = SpriteRenderer::new();
        let mut framebuffer = [0u8; 160 * 144 * 3];
        
        // テスト用パレット
        let obp0 = 0xE4; // 11 10 01 00
        let obp1 = 0x1B; // 00 01 10 11
        
        // パレット0、色1を描画
        assert!(renderer.draw_sprite_pixel(&mut framebuffer, 10, 20, 1, 0, obp0, obp1));
        
        // RGB値を確認（パレット0の色1 = (obp0 >> 2) & 0x03 = 1）
        let pixel_index = (20 * 160 + 10) * 3;
        assert_eq!(framebuffer[pixel_index], 0x8B);     // R
        assert_eq!(framebuffer[pixel_index + 1], 0xAC); // G
        assert_eq!(framebuffer[pixel_index + 2], 0x0F); // B
        
        // 透明ピクセル（色0）は描画されない
        assert!(!renderer.draw_sprite_pixel(&mut framebuffer, 11, 20, 0, 0, obp0, obp1));
    }
}