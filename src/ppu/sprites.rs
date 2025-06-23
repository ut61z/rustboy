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
}