pub mod registers;
pub mod timing;
pub mod vram;
pub mod tiles;
pub mod background;

use crate::memory_map::{dmg, io_registers};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PpuMode {
    HBlank = 0,      // Mode 0: H-Blank
    VBlank = 1,      // Mode 1: V-Blank
    OamScan = 2,     // Mode 2: OAM scan
    Drawing = 3,     // Mode 3: Drawing
}

pub struct Ppu {
    pub registers: registers::PpuRegisters,
    pub vram: vram::Vram,
    pub oam: [u8; 160],  // Object Attribute Memory
    
    // PPU状態
    pub mode: PpuMode,
    pub cycles: u32,
    pub scanline: u8,
    
    // 描画バッファ
    pub framebuffer: [u8; 160 * 144 * 3],  // RGB888形式
    
    // フラグ
    pub vblank_interrupt: bool,
    pub stat_interrupt: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            registers: registers::PpuRegisters::new(),
            vram: vram::Vram::new(),
            oam: [0; 160],
            
            mode: PpuMode::OamScan,
            cycles: 0,
            scanline: 0,
            
            framebuffer: [0; 160 * 144 * 3],
            
            vblank_interrupt: false,
            stat_interrupt: false,
        }
    }
    
    // PPUを1サイクル進める
    pub fn step(&mut self) -> bool {
        self.cycles += 1;
        
        // LYレジスタを更新
        self.registers.ly = self.scanline;
        
        match self.mode {
            PpuMode::OamScan => {
                if self.cycles >= 80 {
                    self.mode = PpuMode::Drawing;
                    self.cycles = 0;
                }
            },
            PpuMode::Drawing => {
                if self.cycles >= 172 {
                    self.mode = PpuMode::HBlank;
                    self.cycles = 0;
                    
                    // スキャンライン描画
                    if self.registers.is_lcd_enabled() {
                        self.draw_scanline();
                    }
                }
            },
            PpuMode::HBlank => {
                if self.cycles >= 204 {
                    self.scanline += 1;
                    self.cycles = 0;
                    
                    if self.scanline >= 144 {
                        // VBlank開始
                        self.mode = PpuMode::VBlank;
                        self.vblank_interrupt = true;
                        return true;  // VBlank割り込み発生
                    } else {
                        self.mode = PpuMode::OamScan;
                    }
                }
            },
            PpuMode::VBlank => {
                if self.cycles >= 456 {
                    self.scanline += 1;
                    self.cycles = 0;
                    
                    if self.scanline >= 154 {
                        // フレーム完了、新しいフレーム開始
                        self.scanline = 0;
                        self.mode = PpuMode::OamScan;
                    }
                }
            },
        }
        
        // STATレジスタを更新
        self.registers.stat = (self.registers.stat & 0xFC) | (self.mode as u8);
        
        false
    }
    
    // スキャンライン描画（改良版）
    fn draw_scanline(&mut self) {
        if !self.registers.is_bg_enabled() {
            // BG無効時は白で塗りつぶし
            let y = self.scanline as usize;
            if y < 144 {
                for x in 0..160 {
                    let pixel_index = (y * 160 + x) * 3;
                    self.framebuffer[pixel_index] = 0x9B;     // R (最明色)
                    self.framebuffer[pixel_index + 1] = 0xBC; // G
                    self.framebuffer[pixel_index + 2] = 0x0F; // B
                }
            }
            return;
        }
        
        let y = self.scanline as usize;
        if y >= 144 {
            return;
        }
        
        // スクロール補正されたY座標
        let bg_y = (y as u8).wrapping_add(self.registers.scy);
        let tile_y = bg_y / 8;          // タイル行
        let pixel_y = bg_y % 8;         // タイル内Y座標
        
        // タイルマップ選択
        let tilemap_base = if self.registers.is_bg_tilemap_high() {
            0x1C00  // $9C00-$9FFF
        } else {
            0x1800  // $9800-$9BFF
        };
        
        // タイルデータアドレス指定モード
        let tiledata_mode = self.registers.is_bg_window_tiledata_high();
        
        // 160ピクセルを描画
        for x in 0..160 {
            // スクロール補正されたX座標
            let bg_x = (x as u8).wrapping_add(self.registers.scx);
            let tile_x = bg_x / 8;          // タイル列
            let pixel_x_in_tile = bg_x % 8; // タイル内X座標
            
            // タイルIDを取得
            let tile_map_addr = tilemap_base + (tile_y as u16) * 32 + (tile_x as u16);
            let tile_id = self.vram.read(tile_map_addr);
            
            // タイルデータアドレス計算
            let tile_data_addr = if tiledata_mode {
                // $8000-$8FFF (unsigned 0-255)
                (tile_id as u16) * 16
            } else {
                // $8800-$97FF (signed -128 to 127)
                if tile_id < 128 {
                    0x1000 + (tile_id as u16) * 16  // $9000 + tile_id * 16
                } else {
                    0x0800 + ((tile_id as u16 - 128) * 16)  // $8800 + (tile_id - 128) * 16
                }
            };
            
            // ピクセルデータを取得（2bpp）
            let byte1 = self.vram.read(tile_data_addr + pixel_y as u16 * 2);
            let byte2 = self.vram.read(tile_data_addr + pixel_y as u16 * 2 + 1);
            
            // ピクセル値を計算
            let bit = 7 - pixel_x_in_tile;
            let pixel_low = (byte1 >> bit) & 1;
            let pixel_high = (byte2 >> bit) & 1;
            let color_id = pixel_low | (pixel_high << 1);
            
            // パレット適用
            let palette_color = match color_id {
                0 => self.registers.bgp & 0x03,
                1 => (self.registers.bgp >> 2) & 0x03,
                2 => (self.registers.bgp >> 4) & 0x03,
                3 => (self.registers.bgp >> 6) & 0x03,
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
            let pixel_index = (y * 160 + x) * 3;
            self.framebuffer[pixel_index] = r;
            self.framebuffer[pixel_index + 1] = g;
            self.framebuffer[pixel_index + 2] = b;
        }
    }
    
    // VBlank割り込みフラグをクリア
    pub fn clear_vblank_interrupt(&mut self) {
        self.vblank_interrupt = false;
    }
    
    // STAT割り込みフラグをクリア
    pub fn clear_stat_interrupt(&mut self) {
        self.stat_interrupt = false;
    }
    
    /// VRAM読み込み（Peripheralsから呼ばれる）
    pub fn read_vram(&self, address: u16) -> u8 {
        self.vram.read(address - dmg::VRAM_START)
    }

    /// VRAM書き込み（Peripheralsから呼ばれる、Drawingモード中はブロック）
    pub fn write_vram(&mut self, address: u16, value: u8) {
        if self.mode != PpuMode::Drawing {
            self.vram.write(address - dmg::VRAM_START, value);
        }
    }

    /// OAM読み込み（Peripheralsから呼ばれる）
    pub fn read_oam(&self, address: u16) -> u8 {
        self.oam[(address - dmg::OAM_START) as usize]
    }

    /// OAM書き込み（Peripheralsから呼ばれる、Drawing/OamScanモード中はブロック）
    pub fn write_oam(&mut self, address: u16, value: u8) {
        if self.mode != PpuMode::Drawing && self.mode != PpuMode::OamScan {
            self.oam[(address - dmg::OAM_START) as usize] = value;
        }
    }

    // メモリ読み込み（レガシー: PPU単体テスト用）
    pub fn read(&self, address: u16) -> u8 {
        match address {
            dmg::VRAM_START..=dmg::VRAM_END => {
                self.vram.read(address - dmg::VRAM_START)
            },
            dmg::OAM_START..=dmg::OAM_END => {
                self.oam[(address - dmg::OAM_START) as usize]
            },
            io_registers::LCDC => self.registers.lcdc,
            io_registers::STAT => self.registers.stat,
            io_registers::SCY => self.registers.scy,
            io_registers::SCX => self.registers.scx,
            io_registers::LY => self.registers.ly,
            io_registers::LYC => self.registers.lyc,
            io_registers::BGP => self.registers.bgp,
            _ => 0xFF,
        }
    }
    
    // メモリ書き込み
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            dmg::VRAM_START..=dmg::VRAM_END => {
                if self.mode != PpuMode::Drawing {
                    self.vram.write(address - dmg::VRAM_START, value);
                }
            },
            dmg::OAM_START..=dmg::OAM_END => {
                if self.mode != PpuMode::Drawing && self.mode != PpuMode::OamScan {
                    self.oam[(address - dmg::OAM_START) as usize] = value;
                }
            },
            io_registers::LCDC => self.registers.lcdc = value,
            io_registers::STAT => self.registers.stat = (self.registers.stat & 0x07) | (value & 0xF8),
            io_registers::SCY => self.registers.scy = value,
            io_registers::SCX => self.registers.scx = value,
            io_registers::LY => {}, // LY は読み取り専用
            io_registers::LYC => self.registers.lyc = value,
            io_registers::BGP => self.registers.bgp = value,
            _ => {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ppu_creation() {
        let ppu = Ppu::new();
        assert_eq!(ppu.mode, PpuMode::OamScan);
        assert_eq!(ppu.cycles, 0);
        assert_eq!(ppu.scanline, 0);
    }
    
    #[test]
    fn test_ppu_step_timing() {
        let mut ppu = Ppu::new();
        
        // OAM Scan (80 cycles)
        for _ in 0..79 {
            assert!(!ppu.step());
            assert_eq!(ppu.mode, PpuMode::OamScan);
        }
        
        assert!(!ppu.step());
        assert_eq!(ppu.mode, PpuMode::Drawing);
    }
}