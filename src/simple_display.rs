// 簡易ASCII描画システム（SDL2代替）

use crate::ppu::Ppu;

pub struct SimpleDisplay {
    width: usize,
    height: usize,
    scale: usize,
}

impl SimpleDisplay {
    pub fn new() -> Self {
        Self {
            width: 160,
            height: 144,
            scale: 2,  // 2x2文字で1ピクセルを表現
        }
    }
    
    // PPUフレームバッファをコンソールに表示
    pub fn present_frame(&self, framebuffer: &[u8; 160 * 144 * 3]) {
        println!("\x1b[2J\x1b[H"); // 画面クリア + カーソル移動
        println!("=== RustBoy GameBoy Emulator ===");
        println!("160x144 画面 (ASCII表示) - 2x2ピクセル縮小");
        println!();
        
        // 2x2ピクセルごとに1文字で表示（より詳細）
        for y in (0..self.height).step_by(2) {
            for x in (0..self.width).step_by(2) {
                let pixel_index = (y * self.width + x) * 3;
                if pixel_index + 2 < framebuffer.len() {
                    let r = framebuffer[pixel_index];
                    let g = framebuffer[pixel_index + 1];
                    let b = framebuffer[pixel_index + 2];
                    
                    // GameBoy色を直接判定
                    let char = match (r, g, b) {
                        (0x0F, 0x38, 0x0F) => '█',  // 最暗色
                        (0x30, 0x62, 0x30) => '▓',  // 暗
                        (0x8B, 0xAC, 0x0F) => '▒',  // 明
                        (0x9B, 0xBC, 0x0F) => '░',  // 最明色
                        _ => {
                            // その他の色は輝度で判定
                            let brightness = ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8;
                            match brightness {
                                0..=63 => '█',    // 最暗
                                64..=127 => '▓',  // 暗
                                128..=191 => '▒', // 明
                                192..=255 => '░', // 最明
                            }
                        }
                    };
                    print!("{}", char);
                }
            }
            println!();
        }
        
        println!();
        println!("Press Ctrl+C to exit");
    }
    
    // PPUテスト用デモパターン表示
    pub fn demo_patterns(&self) {
        println!("=== PPU デモパターン ===");
        
        // チェッカーパターン
        println!("\n1. チェッカーパターン:");
        for y in 0..18 {
            for x in 0..40 {
                let checker = ((x / 2) + (y / 2)) % 2 == 0;
                print!("{}", if checker { "█" } else { "░" });
            }
            println!();
        }
        
        // グラデーション
        println!("\n2. グラデーション:");
        let chars = ['█', '▓', '▒', '░', ' '];
        for _y in 0..8 {
            for x in 0..40 {
                let level = (x * chars.len()) / 40;
                print!("{}", chars[level.min(chars.len() - 1)]);
            }
            println!();
        }
        
        // タイルパターン
        println!("\n3. 8x8タイルパターン:");
        for y in 0..16 {
            for x in 0..32 {
                let tile_x = x / 4;
                let tile_y = y / 4;
                let in_tile_x = x % 4;
                let in_tile_y = y % 4;
                
                // 簡単なタイルパターン
                let pattern = match (tile_x + tile_y) % 4 {
                    0 => (in_tile_x + in_tile_y) % 2 == 0,  // チェッカー
                    1 => in_tile_x < 2,                     // 縦縞
                    2 => in_tile_y < 2,                     // 横縞
                    _ => in_tile_x == in_tile_y,            // 対角線
                };
                
                print!("{}", if pattern { "█" } else { "░" });
            }
            println!();
        }
    }
    
    // PPUフレームバッファの統計情報を表示
    pub fn show_framebuffer_stats(&self, framebuffer: &[u8; 160 * 144 * 3]) {
        let mut color_counts = [0u32; 4];
        
        for chunk in framebuffer.chunks(3) {
            if chunk.len() == 3 {
                let r = chunk[0];
                let g = chunk[1];
                let b = chunk[2];
                
                // GameBoy色を判定
                let color = match (r, g, b) {
                    (0x9B, 0xBC, 0x0F) => 0, // 最明色
                    (0x8B, 0xAC, 0x0F) => 1, // 明
                    (0x30, 0x62, 0x30) => 2, // 暗
                    (0x0F, 0x38, 0x0F) => 3, // 最暗色
                    _ => 0, // その他は最明色として扱う
                };
                color_counts[color] += 1;
            }
        }
        
        println!("=== フレームバッファ統計 ===");
        println!("色0 (最明): {} ピクセル ({:.1}%)", 
                 color_counts[0], 
                 color_counts[0] as f32 / (160.0 * 144.0) * 100.0);
        println!("色1 (明):   {} ピクセル ({:.1}%)", 
                 color_counts[1], 
                 color_counts[1] as f32 / (160.0 * 144.0) * 100.0);
        println!("色2 (暗):   {} ピクセル ({:.1}%)", 
                 color_counts[2], 
                 color_counts[2] as f32 / (160.0 * 144.0) * 100.0);
        println!("色3 (最暗): {} ピクセル ({:.1}%)", 
                 color_counts[3], 
                 color_counts[3] as f32 / (160.0 * 144.0) * 100.0);
        println!("総ピクセル数: {}", 160 * 144);
    }
    
    // PPUレジスタ情報を表示
    pub fn show_ppu_info(&self, ppu: &Ppu) {
        println!("=== PPU状態情報 ===");
        println!("PPUモード: {:?}", ppu.mode);
        println!("スキャンライン: {}", ppu.scanline);
        println!("サイクル: {}", ppu.cycles);
        println!("LCDC: 0x{:02X}", ppu.registers.lcdc);
        println!("STAT: 0x{:02X}", ppu.registers.stat);
        println!("SCY: {}", ppu.registers.scy);
        println!("SCX: {}", ppu.registers.scx);
        println!("LY: {}", ppu.registers.ly);
        println!("BGP: 0x{:02X}", ppu.registers.bgp);
        
        println!("\nLCDC フラグ:");
        println!("  LCD有効: {}", ppu.registers.is_lcd_enabled());
        println!("  BG有効: {}", ppu.registers.is_bg_enabled());
        println!("  BGタイルマップ高位: {}", ppu.registers.is_bg_tilemap_high());
        println!("  BG/Windowタイルデータ高位: {}", ppu.registers.is_bg_window_tiledata_high());
    }
    
    // インタラクティブデモ
    pub fn interactive_demo(&self) {
        use std::io::{self, Write};
        
        loop {
            println!("\n=== RustBoy PPU デモメニュー ===");
            println!("1. デモパターン表示");
            println!("2. PPU動作シミュレーション");
            println!("3. VRAMデータ表示");
            println!("4. 終了");
            print!("選択してください (1-4): ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                match input.trim() {
                    "1" => self.demo_patterns(),
                    "2" => self.simulate_ppu_operation(),
                    "3" => self.show_vram_data(),
                    "4" => {
                        println!("デモを終了します。");
                        break;
                    }
                    _ => println!("無効な選択です。1-4を入力してください。"),
                }
            }
        }
    }
    
    fn simulate_ppu_operation(&self) {
        println!("\n=== PPU動作シミュレーション ===");
        let mut ppu = Ppu::new();
        
        // VRAMに簡単なパターンを設定
        for i in 0..16 {
            ppu.write(0x8000 + i, if i % 2 == 0 { 0xFF } else { 0x00 });
        }
        ppu.write(0x9800, 0x00); // タイルマップにタイル0を設定
        
        println!("PPUを数フレーム実行します...");
        
        for frame in 0..3 {
            println!("\n--- フレーム {} ---", frame + 1);
            
            let mut cycles = 0;
            while cycles < 70224 { // 1フレーム分のサイクル
                let vblank = ppu.step();
                cycles += 1;
                
                if vblank {
                    println!("VBlank発生! (サイクル: {})", cycles);
                    break;
                }
                
                if cycles % 20000 == 0 {
                    println!("Mode: {:?}, Line: {}, Cycles: {}", 
                             ppu.mode, ppu.scanline, ppu.cycles);
                }
            }
            
            self.show_framebuffer_stats(&ppu.framebuffer);
        }
    }
    
    fn show_vram_data(&self) {
        println!("\n=== VRAM データ表示 ===");
        let ppu = Ppu::new();
        
        println!("VRAM (最初の64バイト):");
        for row in 0..4 {
            print!("0x{:04X}: ", 0x8000 + row * 16);
            for col in 0..16 {
                let addr = 0x8000 + row * 16 + col;
                let value = ppu.read(addr);
                print!("{:02X} ", value);
            }
            println!();
        }
        
        println!("\nタイルマップ (最初の32バイト):");
        for row in 0..2 {
            print!("0x{:04X}: ", 0x9800 + row * 16);
            for col in 0..16 {
                let addr = 0x9800 + row * 16 + col;
                let value = ppu.read(addr);
                print!("{:02X} ", value);
            }
            println!();
        }
    }
}

// GameBoy色をUnicode文字に変換
pub fn gameboy_color_to_char(color_id: u8) -> char {
    match color_id & 0x03 {
        0 => ' ',   // 最明色 - 空白
        1 => '░',   // 明 - 薄いシェード
        2 => '▒',   // 暗 - 中間シェード
        3 => '█',   // 最暗色 - 実線
        _ => '?',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_display_creation() {
        let display = SimpleDisplay::new();
        assert_eq!(display.width, 160);
        assert_eq!(display.height, 144);
    }
    
    #[test]
    fn test_gameboy_color_conversion() {
        assert_eq!(gameboy_color_to_char(0), ' ');
        assert_eq!(gameboy_color_to_char(1), '░');
        assert_eq!(gameboy_color_to_char(2), '▒');
        assert_eq!(gameboy_color_to_char(3), '█');
    }
}