// SDL2 LCD表示システム

#[cfg(feature = "with_sdl")]
use sdl2::pixels::{Color, PixelFormatEnum};
#[cfg(feature = "with_sdl")]
use sdl2::render::{Canvas, Texture, TextureCreator};
#[cfg(feature = "with_sdl")]
use sdl2::video::{Window, WindowContext};
#[cfg(feature = "with_sdl")]
use sdl2::{EventPump, Sdl, VideoSubsystem};

const SCREEN_WIDTH: u32 = 160;
const SCREEN_HEIGHT: u32 = 144;
const WINDOW_SCALE: u32 = 4;  // 4倍拡大表示

pub struct LcdDisplay {
    _sdl_context: Sdl,
    _video_subsystem: VideoSubsystem,
    canvas: Canvas<Window>,
    event_pump: EventPump,
}

impl LcdDisplay {
    pub fn new(title: &str) -> Result<Self, String> {
        // SDL2初期化
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        
        // ウィンドウ作成
        let window = video_subsystem
            .window(title, SCREEN_WIDTH * WINDOW_SCALE, SCREEN_HEIGHT * WINDOW_SCALE)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        
        // キャンバス作成
        let mut canvas = window
            .into_canvas()
            .present_vsync()  // VSync有効
            .build()
            .map_err(|e| e.to_string())?;
        
        canvas.set_draw_color(Color::RGB(155, 188, 15));  // GameBoy風の背景色
        canvas.clear();
        canvas.present();
        
        // イベントポンプ作成
        let event_pump = sdl_context.event_pump()?;
        
        Ok(Self {
            _sdl_context: sdl_context,
            _video_subsystem: video_subsystem,
            canvas,
            event_pump,
        })
    }
    
    // フレームバッファを画面に表示
    pub fn present_frame(&mut self, framebuffer: &[u8; 160 * 144 * 3]) -> Result<(), String> {
        // テクスチャを毎回作成して描画
        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
            .map_err(|e| e.to_string())?;
        
        // テクスチャを更新
        texture.update(None, framebuffer, (SCREEN_WIDTH * 3) as usize)
            .map_err(|e| format!("Texture update failed: {:?}", e))?;
        
        // 画面クリア
        self.canvas.clear();
        
        // テクスチャを描画（拡大表示）
        self.canvas.copy(&texture, None, None)?;
        
        // 画面に表示
        self.canvas.present();
        
        Ok(())
    }
    
    // 単色画面を表示（テスト用）
    pub fn present_solid_color(&mut self, r: u8, g: u8, b: u8) -> Result<(), String> {
        let mut buffer = [0u8; 160 * 144 * 3];
        
        for i in (0..buffer.len()).step_by(3) {
            buffer[i] = r;
            buffer[i + 1] = g;
            buffer[i + 2] = b;
        }
        
        self.present_frame(&buffer)
    }
    
    // チェッカーパターンを表示（テスト用）
    pub fn present_checker_pattern(&mut self) -> Result<(), String> {
        let mut buffer = [0u8; 160 * 144 * 3];
        
        for y in 0..144 {
            for x in 0..160 {
                let index = (y * 160 + x) * 3;
                
                // 8x8のチェッカーパターン
                let checker = ((x / 8) + (y / 8)) % 2 == 0;
                if checker {
                    buffer[index] = 0x9B;     // GameBoy緑
                    buffer[index + 1] = 0xBC;
                    buffer[index + 2] = 0x0F;
                } else {
                    buffer[index] = 0x0F;     // 暗い緑
                    buffer[index + 1] = 0x38;
                    buffer[index + 2] = 0x0F;
                }
            }
        }
        
        self.present_frame(&buffer)
    }
    
    // グラデーションパターンを表示（テスト用）  
    pub fn present_gradient_pattern(&mut self) -> Result<(), String> {
        let mut buffer = [0u8; 160 * 144 * 3];
        
        for y in 0..144 {
            for x in 0..160 {
                let index = (y * 160 + x) * 3;
                
                // X座標に基づくグラデーション（GameBoy 4色）
                let color_level = (x * 4) / 160;  // 0-3
                let (r, g, b) = match color_level {
                    0 => (0x9B, 0xBC, 0x0F),  // 最明色
                    1 => (0x8B, 0xAC, 0x0F),  // 明
                    2 => (0x30, 0x62, 0x30),  // 暗
                    _ => (0x0F, 0x38, 0x0F),  // 最暗色
                };
                
                buffer[index] = r;
                buffer[index + 1] = g;
                buffer[index + 2] = b;
            }
        }
        
        self.present_frame(&buffer)
    }
    
    // イベント処理
    pub fn poll_events(&mut self) -> Vec<LcdEvent> {
        use sdl2::event::Event;
        use sdl2::keyboard::Keycode;
        
        let mut events = Vec::new();
        
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => events.push(LcdEvent::Quit),
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    events.push(LcdEvent::Quit);
                }
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    if let Some(button) = keycode_to_gameboy_button(keycode) {
                        events.push(LcdEvent::ButtonDown(button));
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    if let Some(button) = keycode_to_gameboy_button(keycode) {
                        events.push(LcdEvent::ButtonUp(button));
                    }
                }
                _ => {}
            }
        }
        
        events
    }
    
    // FPS制御（60FPS目標）
    pub fn limit_fps(&self) {
        std::thread::sleep(std::time::Duration::from_millis(16)); // 約60FPS
    }
}

// LCD表示イベント
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LcdEvent {
    Quit,
    ButtonDown(GameBoyButton),
    ButtonUp(GameBoyButton),
}

// GameBoyボタン
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameBoyButton {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

// キーコードをGameBoyボタンに変換
fn keycode_to_gameboy_button(keycode: sdl2::keyboard::Keycode) -> Option<GameBoyButton> {
    use sdl2::keyboard::Keycode;
    
    match keycode {
        Keycode::Up | Keycode::W => Some(GameBoyButton::Up),
        Keycode::Down | Keycode::S => Some(GameBoyButton::Down),
        Keycode::Left | Keycode::A => Some(GameBoyButton::Left),
        Keycode::Right | Keycode::D => Some(GameBoyButton::Right),
        Keycode::Z | Keycode::J => Some(GameBoyButton::A),
        Keycode::X | Keycode::K => Some(GameBoyButton::B),
        Keycode::Return => Some(GameBoyButton::Start),
        Keycode::RShift | Keycode::LShift => Some(GameBoyButton::Select),
        _ => None,
    }
}

// フレームレート計測器
pub struct FpsCounter {
    frame_count: u32,
    last_time: std::time::Instant,
    fps: f64,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            frame_count: 0,
            last_time: std::time::Instant::now(),
            fps: 0.0,
        }
    }
    
    pub fn tick(&mut self) {
        self.frame_count += 1;
        
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_time);
        
        if elapsed.as_secs_f64() >= 1.0 {
            self.fps = self.frame_count as f64 / elapsed.as_secs_f64();
            self.frame_count = 0;
            self.last_time = now;
        }
    }
    
    pub fn fps(&self) -> f64 {
        self.fps
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keycode_conversion() {
        use sdl2::keyboard::Keycode;
        
        assert_eq!(keycode_to_gameboy_button(Keycode::Up), Some(GameBoyButton::Up));
        assert_eq!(keycode_to_gameboy_button(Keycode::W), Some(GameBoyButton::Up));
        assert_eq!(keycode_to_gameboy_button(Keycode::Z), Some(GameBoyButton::A));
        assert_eq!(keycode_to_gameboy_button(Keycode::X), Some(GameBoyButton::B));
        assert_eq!(keycode_to_gameboy_button(Keycode::Space), None);
    }
    
    #[test]
    fn test_fps_counter() {
        let mut counter = FpsCounter::new();
        
        // 初期値
        assert_eq!(counter.fps(), 0.0);
        
        // フレームカウント
        counter.tick();
        counter.tick();
        
        // 実際のFPS計算は時間に依存するため、値の範囲のみテスト
        assert!(counter.fps() >= 0.0);
    }
}