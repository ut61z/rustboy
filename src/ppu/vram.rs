// VRAM (Video RAM) 実装

use crate::memory_map::dmg;

pub struct Vram {
    data: [u8; dmg::VRAM_SIZE],
    access_count: u32,
}

impl Vram {
    pub fn new() -> Self {
        Self {
            data: [0; dmg::VRAM_SIZE],
            access_count: 0,
        }
    }
    
    // VRAM読み取り（相対アドレス）
    pub fn read(&self, address: u16) -> u8 {
        if (address as usize) < dmg::VRAM_SIZE {
            self.data[address as usize]
        } else {
            0xFF
        }
    }
    
    // VRAM書き込み（相対アドレス）
    pub fn write(&mut self, address: u16, value: u8) {
        if (address as usize) < dmg::VRAM_SIZE {
            self.data[address as usize] = value;
            self.access_count += 1;
        }
    }
    
    // タイルデータ読み取り（8x8ピクセル、2bpp）
    pub fn read_tile_data(&self, tile_id: u8, addressing_mode: TileAddressingMode) -> TileData {
        let base_address = match addressing_mode {
            TileAddressingMode::Signed => {
                // $8800-$97FF (signed -128 to 127)
                if tile_id < 128 {
                    0x1000 + (tile_id as u16) * 16  // $9000 + tile_id * 16
                } else {
                    0x0800 + ((tile_id as u16 - 128) * 16)  // $8800 + (tile_id - 128) * 16
                }
            }
            TileAddressingMode::Unsigned => {
                // $8000-$8FFF (unsigned 0 to 255)
                (tile_id as u16) * 16  // $8000 + tile_id * 16
            }
        };
        
        let mut tile_data = TileData::new();
        
        // 8行のタイルデータを読み取り
        for y in 0..8 {
            let byte1 = self.read(base_address + y * 2);
            let byte2 = self.read(base_address + y * 2 + 1);
            
            // 8ピクセルのライン
            for x in 0..8 {
                let bit = 7 - x;
                let pixel_low = (byte1 >> bit) & 1;
                let pixel_high = (byte2 >> bit) & 1;
                let pixel_value = pixel_low | (pixel_high << 1);
                
                tile_data.pixels[y as usize][x as usize] = pixel_value;
            }
        }
        
        tile_data
    }
    
    // タイルマップ読み取り
    pub fn read_tile_map(&self, map_select: TileMapSelect, x: u8, y: u8) -> u8 {
        if x >= 32 || y >= 32 {
            return 0;
        }
        
        let base_address = match map_select {
            TileMapSelect::Map0 => 0x1800,  // $9800-$9BFF
            TileMapSelect::Map1 => 0x1C00,  // $9C00-$9FFF
        };
        
        self.read(base_address + (y as u16) * 32 + (x as u16))
    }
    
    // 統計情報
    pub fn get_access_count(&self) -> u32 {
        self.access_count
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileAddressingMode {
    Signed,    // $8800-$97FF (LCDC.4 = 0)
    Unsigned,  // $8000-$8FFF (LCDC.4 = 1)
}

#[derive(Debug, Clone, Copy)]
pub enum TileMapSelect {
    Map0,  // $9800-$9BFF (LCDC.3 = 0)
    Map1,  // $9C00-$9FFF (LCDC.3 = 1)
}

// 8x8タイルデータ（2bpp、4色）
#[derive(Debug, Clone)]
pub struct TileData {
    pub pixels: [[u8; 8]; 8],  // [y][x] = color_id (0-3)
}

impl TileData {
    pub fn new() -> Self {
        Self {
            pixels: [[0; 8]; 8],
        }
    }
    
    // タイルデータを文字で表示（デバッグ用）
    pub fn print(&self) {
        let chars = [' ', '░', '▒', '█'];
        for row in &self.pixels {
            for &pixel in row {
                print!("{}", chars[pixel as usize]);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vram_read_write() {
        let mut vram = Vram::new();
        
        // 基本的な読み書き
        vram.write(0x0000, 0x42);
        assert_eq!(vram.read(0x0000), 0x42);
        
        // 範囲外アクセス
        assert_eq!(vram.read(0x2000), 0xFF);
        vram.write(0x2000, 0x42);  // 無視される
        assert_eq!(vram.get_access_count(), 1);
    }
    
    #[test]
    fn test_tile_data_creation() {
        let mut vram = Vram::new();
        
        // 簡単なタイルパターンを作成
        // "X" パターン
        let pattern = [
            0b10000001, 0b00000000,  // 行0: X.....X
            0b01000010, 0b00000000,  // 行1: .X...X.
            0b00100100, 0b00000000,  // 行2: ..X.X..
            0b00011000, 0b00000000,  // 行3: ...X...
            0b00011000, 0b00000000,  // 行4: ...X...
            0b00100100, 0b00000000,  // 行5: ..X.X..
            0b01000010, 0b00000000,  // 行6: .X...X.
            0b10000001, 0b00000000,  // 行7: X.....X
        ];
        
        // タイル0にパターンを書き込み
        for (i, &byte) in pattern.iter().enumerate() {
            vram.write(i as u16, byte);
        }
        
        // タイルデータを取得
        let tile = vram.read_tile_data(0, TileAddressingMode::Unsigned);
        
        // パターン確認
        assert_eq!(tile.pixels[0][0], 1);  // X
        assert_eq!(tile.pixels[0][1], 0);  // .
        assert_eq!(tile.pixels[0][7], 1);  // X
        assert_eq!(tile.pixels[3][3], 1);  // 中央のX
        assert_eq!(tile.pixels[3][4], 1);  // 中央のX
    }
    
    #[test]
    fn test_tile_map_access() {
        let mut vram = Vram::new();
        
        // タイルマップに値を設定
        vram.write(0x1800, 0x42);  // Map0の(0,0)
        vram.write(0x1801, 0x24);  // Map0の(1,0)
        vram.write(0x1C00, 0x99);  // Map1の(0,0)
        
        assert_eq!(vram.read_tile_map(TileMapSelect::Map0, 0, 0), 0x42);
        assert_eq!(vram.read_tile_map(TileMapSelect::Map0, 1, 0), 0x24);
        assert_eq!(vram.read_tile_map(TileMapSelect::Map1, 0, 0), 0x99);
        
        // 範囲外
        assert_eq!(vram.read_tile_map(TileMapSelect::Map0, 32, 0), 0);
        assert_eq!(vram.read_tile_map(TileMapSelect::Map0, 0, 32), 0);
    }
}