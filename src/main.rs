// src/main.rs - メモリマップ対応版
use std::env;
use std::fs;

mod memory_map;      // メモリマップ定義
mod memory;          // メモリコンポーネント
mod peripherals;     // メモリバス
mod cpu;             // CPUコンポーネント

use memory::BootRom;
use peripherals::Peripherals;
use cpu::Cpu;

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
    
    println!("\n=== Phase 2 + Phase 3 テスト完了 ===");
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
