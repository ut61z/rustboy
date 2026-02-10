pub mod registers;
pub mod timing;
pub mod vram;
pub mod tiles;
pub mod background;
pub mod sprites;

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

    // ウィンドウ内部ラインカウンタ（フレーム内でウィンドウが描画された行数）
    pub window_line_counter: u8,

    // 描画バッファ
    pub framebuffer: [u8; 160 * 144 * 3],  // RGB888形式

    // BG色ID配列（スプライト優先度判定用）
    bg_color_ids: [u8; 160],

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

            window_line_counter: 0,

            framebuffer: [0; 160 * 144 * 3],
            bg_color_ids: [0; 160],

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
                        self.window_line_counter = 0;
                        self.mode = PpuMode::OamScan;
                    }
                }
            },
        }
        
        // STATレジスタを更新
        self.registers.stat = (self.registers.stat & 0xFC) | (self.mode as u8);
        
        false
    }
    
    // スキャンライン描画（BG + ウィンドウ + スプライト）
    fn draw_scanline(&mut self) {
        let y = self.scanline as usize;
        if y >= 144 {
            return;
        }

        // BG色ID配列をクリア
        self.bg_color_ids = [0; 160];

        if !self.registers.is_bg_enabled() {
            // BG無効時は白で塗りつぶし
            for x in 0..160 {
                let pixel_index = (y * 160 + x) * 3;
                self.framebuffer[pixel_index] = 0x9B;     // R (最明色)
                self.framebuffer[pixel_index + 1] = 0xBC; // G
                self.framebuffer[pixel_index + 2] = 0x0F; // B
            }
        } else {
            // 背景描画
            self.draw_bg_scanline(y);

            // ウィンドウ描画
            self.draw_window_scanline(y);
        }

        // スプライト描画
        let start = y * 160 * 3;
        let end = start + 160 * 3;
        sprites::SpriteRenderer::render_scanline(
            &self.oam,
            &self.vram,
            &self.registers,
            self.scanline,
            &self.bg_color_ids,
            &mut self.framebuffer[start..end],
        );
    }

    // 背景スキャンライン描画
    fn draw_bg_scanline(&mut self, y: usize) {
        let bg_y = (y as u8).wrapping_add(self.registers.scy);
        let tile_y = bg_y / 8;
        let pixel_y = bg_y % 8;

        let tilemap_base = if self.registers.is_bg_tilemap_high() {
            0x1C00
        } else {
            0x1800
        };

        let tiledata_mode = self.registers.is_bg_window_tiledata_high();

        for x in 0..160 {
            let bg_x = (x as u8).wrapping_add(self.registers.scx);
            let tile_x = bg_x / 8;
            let pixel_x_in_tile = bg_x % 8;

            let tile_map_addr = tilemap_base + (tile_y as u16) * 32 + (tile_x as u16);
            let tile_id = self.vram.read(tile_map_addr);

            let tile_data_addr = Self::calc_tile_data_addr(tile_id, tiledata_mode);

            let byte1 = self.vram.read(tile_data_addr + pixel_y as u16 * 2);
            let byte2 = self.vram.read(tile_data_addr + pixel_y as u16 * 2 + 1);

            let bit = 7 - pixel_x_in_tile;
            let pixel_low = (byte1 >> bit) & 1;
            let pixel_high = (byte2 >> bit) & 1;
            let color_id = pixel_low | (pixel_high << 1);

            // BG色IDを保存（スプライト優先度判定用）
            self.bg_color_ids[x] = color_id;

            let palette_color = self.registers.get_bg_palette_color(color_id);
            let (r, g, b) = tiles::ColorConverter::dmg_to_rgb888(palette_color);

            let pixel_index = (y * 160 + x) * 3;
            self.framebuffer[pixel_index] = r;
            self.framebuffer[pixel_index + 1] = g;
            self.framebuffer[pixel_index + 2] = b;
        }
    }

    // ウィンドウスキャンライン描画
    fn draw_window_scanline(&mut self, y: usize) {
        if !self.registers.is_window_enabled() {
            return;
        }

        let wy = self.registers.wy;
        let wx = self.registers.wx;

        // WXは画面X + 7 の値
        if wx > 166 || wy > 143 {
            return;
        }

        // 現在のスキャンラインがウィンドウ開始Y以降か
        if (y as u8) < wy {
            return;
        }

        let window_x_start = if wx < 7 { 0 } else { (wx - 7) as usize };

        let tilemap_base = if self.registers.is_window_tilemap_high() {
            0x1C00
        } else {
            0x1800
        };

        let tiledata_mode = self.registers.is_bg_window_tiledata_high();
        let window_line = self.window_line_counter;
        let tile_y = window_line / 8;
        let pixel_y = window_line % 8;

        let mut window_drawn = false;

        for x in window_x_start..160 {
            let window_x = (x - window_x_start) as u8;
            let tile_x = window_x / 8;
            let pixel_x_in_tile = window_x % 8;

            let tile_map_addr = tilemap_base + (tile_y as u16) * 32 + (tile_x as u16);
            let tile_id = self.vram.read(tile_map_addr);

            let tile_data_addr = Self::calc_tile_data_addr(tile_id, tiledata_mode);

            let byte1 = self.vram.read(tile_data_addr + pixel_y as u16 * 2);
            let byte2 = self.vram.read(tile_data_addr + pixel_y as u16 * 2 + 1);

            let bit = 7 - pixel_x_in_tile;
            let pixel_low = (byte1 >> bit) & 1;
            let pixel_high = (byte2 >> bit) & 1;
            let color_id = pixel_low | (pixel_high << 1);

            // ウィンドウ部分のBG色IDを更新
            self.bg_color_ids[x] = color_id;

            let palette_color = self.registers.get_bg_palette_color(color_id);
            let (r, g, b) = tiles::ColorConverter::dmg_to_rgb888(palette_color);

            let pixel_index = (y * 160 + x) * 3;
            self.framebuffer[pixel_index] = r;
            self.framebuffer[pixel_index + 1] = g;
            self.framebuffer[pixel_index + 2] = b;

            window_drawn = true;
        }

        // ウィンドウが実際に描画された場合のみカウンタをインクリメント
        if window_drawn {
            self.window_line_counter += 1;
        }
    }

    // タイルデータアドレス計算
    fn calc_tile_data_addr(tile_id: u8, unsigned_mode: bool) -> u16 {
        if unsigned_mode {
            (tile_id as u16) * 16
        } else {
            if tile_id < 128 {
                0x1000 + (tile_id as u16) * 16
            } else {
                0x0800 + ((tile_id as u16 - 128) * 16)
            }
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