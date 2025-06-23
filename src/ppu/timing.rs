// PPU タイミング制御

// GameBoy PPU タイミング定数
pub const CYCLES_OAM_SCAN: u32 = 80;     // Mode 2: OAM scan
pub const CYCLES_DRAWING: u32 = 172;     // Mode 3: Drawing  
pub const CYCLES_HBLANK: u32 = 204;      // Mode 0: H-Blank
pub const CYCLES_SCANLINE: u32 = 456;    // 1スキャンライン合計

pub const SCANLINES_VISIBLE: u8 = 144;   // 可視スキャンライン数
pub const SCANLINES_TOTAL: u8 = 154;     // 総スキャンライン数
pub const SCANLINES_VBLANK: u8 = 10;     // VBlankスキャンライン数

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// PPUタイミング情報を計算
pub struct PpuTiming {
    pub cycles_per_frame: u32,
    pub cycles_per_second: u32,
    pub frames_per_second: f64,
}

impl PpuTiming {
    pub fn new() -> Self {
        let cycles_per_frame = CYCLES_SCANLINE * (SCANLINES_TOTAL as u32);
        let cpu_frequency = 4_194_304; // GameBoy CPU クロック周波数 (Hz)
        let frames_per_second = cpu_frequency as f64 / cycles_per_frame as f64;
        
        Self {
            cycles_per_frame,
            cycles_per_second: cpu_frequency,
            frames_per_second,
        }
    }
    
    // 指定されたフレーム数に必要なサイクル数
    pub fn cycles_for_frames(&self, frames: u32) -> u32 {
        self.cycles_per_frame * frames
    }
    
    // 指定された時間(秒)に必要なサイクル数
    pub fn cycles_for_duration(&self, seconds: f64) -> u32 {
        (self.cycles_per_second as f64 * seconds) as u32
    }
    
    // サイクル数から経過時間(秒)を計算
    pub fn duration_from_cycles(&self, cycles: u32) -> f64 {
        cycles as f64 / self.cycles_per_second as f64
    }
}

// スキャンライン位置からPPUモードを判定
pub fn get_expected_mode(scanline: u8, cycle_in_line: u32) -> super::PpuMode {
    if scanline >= SCANLINES_VISIBLE {
        super::PpuMode::VBlank
    } else if cycle_in_line < CYCLES_OAM_SCAN {
        super::PpuMode::OamScan
    } else if cycle_in_line < CYCLES_OAM_SCAN + CYCLES_DRAWING {
        super::PpuMode::Drawing
    } else {
        super::PpuMode::HBlank
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_timing_constants() {
        assert_eq!(CYCLES_SCANLINE, CYCLES_OAM_SCAN + CYCLES_DRAWING + CYCLES_HBLANK);
        assert_eq!(SCANLINES_TOTAL, SCANLINES_VISIBLE + SCANLINES_VBLANK);
    }
    
    #[test]
    fn test_ppu_timing() {
        let timing = PpuTiming::new();
        
        // フレームレートが約59.7FPSであることを確認
        assert!((timing.frames_per_second - 59.7).abs() < 0.1);
        
        // 1フレームのサイクル数
        assert_eq!(timing.cycles_per_frame, CYCLES_SCANLINE * SCANLINES_TOTAL as u32);
    }
    
    #[test]
    fn test_mode_detection() {
        use super::super::PpuMode;
        
        // 可視スキャンライン中のモード
        assert_eq!(get_expected_mode(0, 40), PpuMode::OamScan);
        assert_eq!(get_expected_mode(0, 120), PpuMode::Drawing);
        assert_eq!(get_expected_mode(0, 300), PpuMode::HBlank);
        
        // VBlank中
        assert_eq!(get_expected_mode(144, 0), PpuMode::VBlank);
        assert_eq!(get_expected_mode(150, 200), PpuMode::VBlank);
    }
}