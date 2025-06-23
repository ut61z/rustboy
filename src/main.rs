// src/main.rs - メモリマップ対応版
use std::env;
use std::fs;

mod memory_map;      // メモリマップ定義
mod memory;          // メモリコンポーネント
mod peripherals;     // メモリバス
mod cpu;             // CPUコンポーネント
mod ppu;             // PPUコンポーネント
mod simple_display;  // 簡易ASCII表示

#[cfg(feature = "with_sdl")]
mod lcd;             // LCDディスプレイ

use memory::BootRom;
use peripherals::Peripherals;
use cpu::Cpu;
use ppu::Ppu;

// メモリマップモジュールから関数をインポート
use memory_map::{
    print_memory_map, 
    get_address_info, 
    analyze_address,
    dmg,
    io_registers
};

fn main() {
    println!("=== Game Boy Emulator - Phase 2: Memory System with Memory Map ===\n");
    
    // メモリマップを表示
    print_memory_map();
    println!();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        // BootROMファイルが指定された場合
        load_bootrom_from_file(&args[1]);
    } else {
        // ダミーBootROMでテスト
        test_with_dummy_bootrom();
    }
    
    // メモリマップのデモ
    demo_memory_map();
}

fn load_bootrom_from_file(bootrom_path: &str) {
    println!("BootROMファイルを読み込み中: {}", bootrom_path);
    
    match fs::read(bootrom_path) {
        Ok(data) => {
            match BootRom::new(data.into_boxed_slice()) {
                Ok(bootrom) => {
                    println!("✓ BootROM読み込み成功");
                    test_memory_system(bootrom);
                }
                Err(e) => {
                    eprintln!("✗ BootROM作成エラー: {}", e);
                    println!("ダミーBootROMでテストを続行...\n");
                    test_with_dummy_bootrom();
                }
            }
        }
        Err(e) => {
            eprintln!("✗ ファイル読み込みエラー: {}", e);
            println!("ダミーBootROMでテストを続行...\n");
            test_with_dummy_bootrom();
        }
    }
}

fn test_with_dummy_bootrom() {
    println!("ダミーBootROMでテスト開始");
    let bootrom = BootRom::new_dummy();
    test_memory_system(bootrom);
}

fn test_memory_system(bootrom: BootRom) {
    let mut peripherals = Peripherals::new(bootrom);
    
    println!("\n=== メモリシステムテスト（メモリマップ対応版） ===");
    
    // 1. アドレス情報テスト
    test_address_info();
    
    // 2. BootROMテスト
    test_bootrom(&mut peripherals);
    
    // 3. WRAMテスト
    test_wram(&mut peripherals);
    
    // 4. HRAMテスト
    test_hram(&mut peripherals);
    
    // 5. I/Oレジスタテスト
    test_io_registers(&mut peripherals);
    
    // 6. 統計情報表示
    show_statistics(&peripherals);
    
    // 7. Phase 3: CPU テスト
    test_cpu_system(&mut peripherals);
    
    // 8. Phase 4: PPU + LCD テスト
    test_ppu_lcd_system();
    
    println!("\n=== Phase 2 + Phase 3 + Phase 4 テスト完了 ===");
}

fn test_address_info() {
    println!("\n--- アドレス情報テスト ---");
    
    let test_addresses = [
        0x0000,  // BootROM
        0x0100,  // Cartridge ROM
        0x8000,  // VRAM
        0xC000,  // WRAM
        0xFF00,  // JOYP
        0xFF40,  // LCDC
        0xFF50,  // BootROM disable
        0xFF80,  // HRAM
        0xFFFF,  // IE
    ];
    
    for &addr in &test_addresses {
        println!("{}", get_address_info(addr));
    }
}

fn test_bootrom(peripherals: &mut Peripherals) {
    println!("\n--- BootROMテスト（メモリマップ対応） ---");
    
    // BootROM有効時のテスト
    let stats = peripherals.get_stats();
    println!("BootROM状態: {}", if stats.bootrom_active { "有効" } else { "無効" });
    
    // BootROM領域から読み取り（定数を使用）
    println!("BootROM範囲: 0x{:04X}-0x{:04X}", dmg::BOOTROM_START, dmg::BOOTROM_END);
    
    let value_start = peripherals.read(dmg::BOOTROM_START);
    let value_mid = peripherals.read(0x0050);
    let value_end = peripherals.read(dmg::BOOTROM_END);
    
    println!("BootROM[0x{:04X}] = 0x{:02X}", dmg::BOOTROM_START, value_start);
    println!("BootROM[0x0050] = 0x{:02X}", value_mid);
    println!("BootROM[0x{:04X}] = 0x{:02X}", dmg::BOOTROM_END, value_end);
    
    // BootROM無効化テスト（定数を使用）
    println!("BootROMを無効化中（0x{:04X}に書き込み）...", io_registers::BOOTROM_DISABLE);
    peripherals.write(io_registers::BOOTROM_DISABLE, 0x01);
    
    let stats = peripherals.get_stats();
    println!("BootROM状態: {}", if stats.bootrom_active { "有効" } else { "無効" });
    
    // 無効化後の読み取り
    let value_after = peripherals.read(dmg::BOOTROM_START);
    println!("無効化後のBootROM[0x{:04X}] = 0x{:02X}", dmg::BOOTROM_START, value_after);
}

fn test_wram(peripherals: &mut Peripherals) {
    println!("\n--- WRAMテスト（メモリマップ対応） ---");
    
    println!("WRAM範囲: 0x{:04X}-0x{:04X} ({} bytes)", 
             dmg::WRAM_START, dmg::WRAM_END, dmg::WRAM_SIZE);
    
    // WRAM書き込みテスト（定数を使用）
    peripherals.write(dmg::WRAM_START, 0x42);
    peripherals.write(dmg::WRAM_START + 1, 0x24);
    peripherals.write(dmg::WRAM_END, 0xFF);
    
    // WRAM読み取りテスト
    println!("WRAM[0x{:04X}] = 0x{:02X}", dmg::WRAM_START, peripherals.read(dmg::WRAM_START));
    println!("WRAM[0x{:04X}] = 0x{:02X}", dmg::WRAM_START + 1, peripherals.read(dmg::WRAM_START + 1));
    println!("WRAM[0x{:04X}] = 0x{:02X}", dmg::WRAM_END, peripherals.read(dmg::WRAM_END));
    
    // WRAMエコー領域テスト（定数を使用）
    println!("WRAMエコー範囲: 0x{:04X}-0x{:04X}", dmg::WRAM_ECHO_START, dmg::WRAM_ECHO_END);
    peripherals.write(dmg::WRAM_ECHO_START, 0x99);  // エコー領域に書き込み
    println!("エコー書き込み後のWRAM[0x{:04X}] = 0x{:02X}", 
             dmg::WRAM_START, peripherals.read(dmg::WRAM_START));
}

fn test_hram(peripherals: &mut Peripherals) {
    println!("\n--- HRAMテスト（メモリマップ対応） ---");
    
    println!("HRAM範囲: 0x{:04X}-0x{:04X} ({} bytes)", 
             dmg::HRAM_START, dmg::HRAM_END, dmg::HRAM_SIZE);
    
    // HRAM書き込み・読み取りテスト（定数を使用）
    peripherals.write(dmg::HRAM_START, 0xAB);
    peripherals.write(dmg::HRAM_START + 1, 0xCD);
    peripherals.write(dmg::HRAM_END, 0xEF);
    
    println!("HRAM[0x{:04X}] = 0x{:02X}", dmg::HRAM_START, peripherals.read(dmg::HRAM_START));
    println!("HRAM[0x{:04X}] = 0x{:02X}", dmg::HRAM_START + 1, peripherals.read(dmg::HRAM_START + 1));
    println!("HRAM[0x{:04X}] = 0x{:02X}", dmg::HRAM_END, peripherals.read(dmg::HRAM_END));
}

fn test_io_registers(peripherals: &mut Peripherals) {
    println!("\n--- I/Oレジスタテスト ---");
    
    // 重要なI/Oレジスタの情報表示
    let important_registers = [
        io_registers::JOYP,
        io_registers::LCDC,
        io_registers::STAT,
        io_registers::LY,
        io_registers::BOOTROM_DISABLE,
    ];
    
    println!("重要なI/Oレジスタ:");
    for &addr in &important_registers {
        println!("  {}", get_address_info(addr));
    }
    
    // I/Oレジスタへの書き込みテスト
    println!("\nI/Oレジスタ書き込みテスト:");
    peripherals.write(io_registers::LCDC, 0x91);  // 典型的なLCDC値
    println!("LCDC(0x{:04X})に0x91を書き込み", io_registers::LCDC);
    
    let lcdc_value = peripherals.read(io_registers::LCDC);
    println!("LCDC読み取り結果: 0x{:02X}", lcdc_value);
}

fn show_statistics(peripherals: &Peripherals) {
    println!("\n--- メモリ統計情報 ---");
    let stats = peripherals.get_stats();
    println!("{}", stats);
}

fn demo_memory_map() {
    println!("\n=== メモリマップデモ ===");
    
    // 特定アドレスの詳細分析
    println!("\n特定アドレスの詳細分析:");
    analyze_address(io_registers::LCDC);
    
    // メモリ領域の境界確認
    println!("\nメモリ領域境界の確認:");
    let boundary_addresses = [
        dmg::BOOTROM_END,
        dmg::BOOTROM_END + 1,
        dmg::WRAM_START - 1,
        dmg::WRAM_START,
        dmg::HRAM_START - 1,
        dmg::HRAM_START,
    ];
    
    for &addr in &boundary_addresses {
        println!("  {}", get_address_info(addr));
    }
}

fn test_cpu_system(peripherals: &mut Peripherals) {
    println!("\n=== Phase 3: CPU システムテスト ===");
    
    let mut cpu = Cpu::new();
    
    // CPUレジスタテスト
    test_cpu_registers(&mut cpu);
    
    // 基本命令テスト
    test_basic_instructions(&mut cpu, peripherals);
    
    // 簡単なプログラム実行テスト
    test_simple_program(&mut cpu, peripherals);
    
    println!("=== CPU テスト完了 ===");
}

fn test_cpu_registers(cpu: &mut Cpu) {
    println!("\n--- CPUレジスタテスト ---");
    
    // 初期状態確認
    println!("初期状態: {}", cpu.debug_string());
    
    // レジスタペア操作テスト
    cpu.registers.set_af(0x1234);
    cpu.registers.set_bc(0x5678);
    cpu.registers.set_de(0x9ABC);
    cpu.registers.set_hl(0xDEF0);
    cpu.registers.sp = 0xFFFE;
    cpu.registers.pc = 0x0100;
    
    println!("設定後: {}", cpu.debug_string());
    
    // フラグテスト
    cpu.registers.set_flags(true, false, true, false);
    println!("フラグ: {}", cpu.registers.flags_string());
    
    assert_eq!(cpu.registers.af(), 0x12A0); // フラグの下位4bitはマスク
    assert_eq!(cpu.registers.bc(), 0x5678);
    assert_eq!(cpu.registers.de(), 0x9ABC);
    assert_eq!(cpu.registers.hl(), 0xDEF0);
    
    println!("✓ レジスタテスト成功");
}

fn test_basic_instructions(cpu: &mut Cpu, peripherals: &mut Peripherals) {
    println!("\n--- 基本命令テスト ---");
    
    // CPUリセット
    cpu.reset();
    
    // テストプログラムをWRAM領域に配置
    let test_program = [
        0x00,       // NOP
        0x3E, 0x42, // LD A, 0x42
        0x06, 0x24, // LD B, 0x24
        0x31, 0xFE, 0xFF, // LD SP, 0xFFFE
        0x00,       // NOP
    ];
    
    cpu.registers.pc = 0xC000; // WRAMの開始アドレスから実行
    for (i, &byte) in test_program.iter().enumerate() {
        peripherals.write(0xC000 + i as u16, byte);
    }
    
    println!("初期状態: {}", cpu.debug_string());
    
    // 命令を順次実行
    for i in 0..5 {
        match cpu.step(peripherals) {
            Ok(cycles) => {
                println!("命令{}: {} ({}cycles)", i + 1, cpu.debug_string(), cycles);
            }
            Err(e) => {
                eprintln!("命令実行エラー: {}", e);
                break;
            }
        }
    }
    
    // 結果確認
    assert_eq!(cpu.registers.a, 0x42);
    assert_eq!(cpu.registers.b, 0x24);
    assert_eq!(cpu.registers.sp, 0xFFFE);
    assert_eq!(cpu.instruction_count, 5);
    
    println!("✓ 基本命令テスト成功");
}

fn test_simple_program(cpu: &mut Cpu, peripherals: &mut Peripherals) {
    println!("\n--- 簡単なプログラム実行テスト ---");
    
    cpu.reset();
    
    // 簡単なループプログラムをWRAM領域に配置
    let program = [
        0x3E, 0x01,       // LD A, 1        (PC: 0xC000-0xC001)
        0x06, 0x05,       // LD B, 5        (PC: 0xC002-0xC003)
        0x00,             // NOP            (PC: 0xC004)
        0x18, 0xFD,       // JR -3          (PC: 0xC005-0xC006) -> 0xC004
        0x00,             // NOP (到達しない)
    ];
    
    cpu.registers.pc = 0xC000; // WRAMの開始アドレスから実行
    for (i, &byte) in program.iter().enumerate() {
        peripherals.write(0xC000 + i as u16, byte);
    }
    
    println!("プログラム実行開始: {}", cpu.debug_string());
    
    // 最大10命令実行（無限ループ防止）
    for i in 0..10 {
        match cpu.step(peripherals) {
            Ok(cycles) => {
                println!("実行{}: {} ({}cycles)", i + 1, cpu.debug_string(), cycles);
                
                // 0xC004-0xC006のループに入ったら停止
                if cpu.registers.pc == 0xC004 && i > 5 {
                    println!("ループ検出、テスト終了");
                    break;
                }
            }
            Err(e) => {
                eprintln!("実行エラー: {}", e);
                break;
            }
        }
    }
    
    // レジスタ値確認
    assert_eq!(cpu.registers.a, 0x01);
    assert_eq!(cpu.registers.b, 0x05);
    
    println!("✓ プログラム実行テスト成功");
}

fn test_ppu_lcd_system() {
    
    println!("\n=== Phase 4: PPU + LCD システムテスト ===");
    
    // PPU基本テスト
    test_ppu_basic();
    
    // LCD表示テスト（オプション）
    #[cfg(feature = "with_sdl")]
    {
        println!("\nLCD表示テストを実行しますか？ (5秒間表示)");
        println!("注意: ウィンドウが開きます。ESCキーで終了できます。");
        
        // 環境変数でLCDテストをスキップ可能
        if std::env::var("SKIP_LCD_TEST").is_ok() {
            println!("SKIP_LCD_TEST環境変数が設定されているため、LCD表示テストをスキップします。");
            return;
        }
        
        test_lcd_display();
    }
    
    #[cfg(not(feature = "with_sdl"))]
    {
        println!("\nLCD表示テストはSDL2機能が有効でないためスキップされます。");
        println!("SDL2機能を有効にするには: cargo run --features with_sdl");
        println!("\n代わりに簡易ASCII表示デモを実行します:");
        test_simple_display();
    }
}

#[cfg(feature = "with_sdl")]
fn test_lcd_display() {
    use lcd::{LcdDisplay, LcdEvent, FpsCounter};
    
    match LcdDisplay::new("RustBoy - Phase 4 テスト") {
        Ok(mut display) => {
            println!("✓ SDL2 LCDディスプレイ初期化成功");
            
            let mut fps_counter = FpsCounter::new();
            let start_time = std::time::Instant::now();
            let test_duration = std::time::Duration::from_secs(5);
            
            // 5秒間のLCD表示テスト
            while start_time.elapsed() < test_duration {
                // イベント処理
                let events = display.poll_events();
                for event in events {
                    match event {
                        LcdEvent::Quit => {
                            println!("✓ 終了イベント受信");
                            return;
                        }
                        LcdEvent::ButtonDown(button) => {
                            println!("ボタン押下: {:?}", button);
                        }
                        LcdEvent::ButtonUp(button) => {
                            println!("ボタン離し: {:?}", button);
                        }
                    }
                }
                
                // 画面表示テスト（時間に応じてパターン変更）
                let elapsed_secs = start_time.elapsed().as_secs();
                match elapsed_secs {
                    0..=1 => {
                        if let Err(e) = display.present_solid_color(0x9B, 0xBC, 0x0F) {
                            eprintln!("表示エラー: {}", e);
                            break;
                        }
                    }
                    2..=3 => {
                        if let Err(e) = display.present_checker_pattern() {
                            eprintln!("表示エラー: {}", e);
                            break;
                        }
                    }
                    _ => {
                        if let Err(e) = display.present_gradient_pattern() {
                            eprintln!("表示エラー: {}", e);
                            break;
                        }
                    }
                }
                
                fps_counter.tick();
                display.limit_fps();
            }
            
            println!("✓ LCD表示テスト完了 (平均FPS: {:.1})", fps_counter.fps());
        }
        Err(e) => {
            println!("⚠ SDL2初期化失敗: {}", e);
            println!("  LCDテストをスキップします（システムにSDL2が必要です）");
        }
    }
}

fn test_ppu_basic() {
    println!("\n--- PPU基本テスト ---");
    
    let mut ppu = Ppu::new();
    
    // 初期状態確認
    println!("PPU初期状態: Mode={:?}, Scanline={}, Cycles={}", 
             ppu.mode, ppu.scanline, ppu.cycles);
    
    // PPU step テスト
    let mut vblank_occurred = false;
    let mut step_count = 0;
    
    // 1フレーム分実行（約70224サイクル）
    while step_count < 80000 && !vblank_occurred {
        vblank_occurred = ppu.step();
        step_count += 1;
        
        if step_count % 10000 == 0 {
            println!("Step {}: Mode={:?}, Scanline={}, Cycles={}", 
                     step_count, ppu.mode, ppu.scanline, ppu.cycles);
        }
    }
    
    if vblank_occurred {
        println!("✓ VBlank割り込み発生 (Step: {})", step_count);
    } else {
        println!("⚠ VBlank割り込み未発生");
    }
    
    // VRAM基本テスト
    println!("\n--- VRAM基本テスト ---");
    
    // VRAMに簡単なパターンを書き込み
    ppu.write(0x8000, 0xFF);  // タイルデータ
    ppu.write(0x8001, 0x00);
    ppu.write(0x9800, 0x00);  // タイルマップ
    
    let tile_data = ppu.read(0x8000);
    let tile_map = ppu.read(0x9800);
    
    println!("VRAM[0x8000] = 0x{:02X}", tile_data);
    println!("VRAM[0x9800] = 0x{:02X}", tile_map);
    
    assert_eq!(tile_data, 0xFF);
    assert_eq!(tile_map, 0x00);
    
    println!("✓ VRAM読み書きテスト成功");
    
    // レジスタテスト
    println!("\n--- PPUレジスタテスト ---");
    
    ppu.write(io_registers::LCDC, 0x91);
    ppu.write(io_registers::SCY, 0x10);
    ppu.write(io_registers::SCX, 0x08);
    
    assert_eq!(ppu.read(io_registers::LCDC), 0x91);
    assert_eq!(ppu.read(io_registers::SCY), 0x10);
    assert_eq!(ppu.read(io_registers::SCX), 0x08);
    
    println!("✓ PPUレジスタテスト成功");
    
    println!("✓ PPU基本テスト完了");
}

fn test_simple_display() {
    use simple_display::SimpleDisplay;
    
    println!("\n=== 簡易ASCII表示デモ ===");
    
    let display = SimpleDisplay::new();
    let mut ppu = Ppu::new();
    
    // VRAMに簡単なパターンを設定
    println!("VRAMにテストパターンを設定中...");
    
    // タイル0: チェッカーパターン
    let checker_pattern = [
        0b10101010, 0b00000000,  // 行0
        0b01010101, 0b00000000,  // 行1
        0b10101010, 0b00000000,  // 行2
        0b01010101, 0b00000000,  // 行3
        0b10101010, 0b00000000,  // 行4
        0b01010101, 0b00000000,  // 行5
        0b10101010, 0b00000000,  // 行6
        0b01010101, 0b00000000,  // 行7
    ];
    
    for (i, &byte) in checker_pattern.iter().enumerate() {
        ppu.write(0x8000 + i as u16, byte);
    }
    
    // タイル1: 縦線パターン
    let vertical_pattern = [
        0b11001100, 0b00000000,  // 行0
        0b11001100, 0b00000000,  // 行1
        0b11001100, 0b00000000,  // 行2
        0b11001100, 0b00000000,  // 行3
        0b11001100, 0b00000000,  // 行4
        0b11001100, 0b00000000,  // 行5
        0b11001100, 0b00000000,  // 行6
        0b11001100, 0b00000000,  // 行7
    ];
    
    for (i, &byte) in vertical_pattern.iter().enumerate() {
        ppu.write(0x8010 + i as u16, byte);
    }
    
    // タイルマップにパターンを設定
    for i in 0..32*32 {
        let tile_id = if (i / 32 + i % 32) % 2 == 0 { 0 } else { 1 };
        ppu.write(0x9800 + i as u16, tile_id);
    }
    
    // PPUレジスタ設定
    ppu.write(io_registers::LCDC, 0x91);  // LCD有効、BG有効
    ppu.write(io_registers::BGP, 0b11100100);  // パレット設定
    
    // PPU状態表示
    display.show_ppu_info(&ppu);
    
    // 数フレーム実行して画面生成
    println!("\nPPUを実行してフレームを生成中...");
    
    for frame in 0..3 {
        println!("\n--- フレーム {} 実行 ---", frame + 1);
        
        let mut cycles = 0;
        while cycles < 70224 {
            let vblank = ppu.step();
            cycles += 1;
            
            if vblank {
                println!("VBlank発生! (サイクル: {})", cycles);
                break;
            }
        }
        
        // フレームバッファ統計表示
        display.show_framebuffer_stats(&ppu.framebuffer);
        
        // 最初のフレームのみ画面表示
        if frame == 0 {
            println!("\n実際のフレームバッファを表示:");
            display.present_frame(&ppu.framebuffer);
        }
    }
    
    // デモパターン表示
    display.demo_patterns();
    
    println!("\n=== 簡易表示デモ完了 ===");
    println!("より本格的な表示にはSDL2が必要です:");
    println!("1. cmake をインストール: sudo pacman -S cmake (Arch) / sudo apt install cmake (Ubuntu)");
    println!("2. SDL2機能でビルド: cargo run --features with_sdl");
}
