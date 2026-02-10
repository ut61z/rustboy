# CLAUDE.md

このファイルはClaude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## プロジェクト概要

RustBoy — Rustで書かれたGameBoy (DMG) エミュレータ。ハードウェア精確なエミュレーションを目指し、段階的に開発中。現在Phase 5（メモリバス統合・割り込み・タイマー・CPU命令拡張）まで完了、93テスト全パス。

## 環境設定

- **Rust版**: 1.87.0（`mise.toml`で管理）
- **Edition**: 2024
- **Lints**: `dead_code`と`unused_variables`はallow設定

## 共通開発コマンド

### ビルドと実行
```bash
cargo check                           # 高速コンパイルチェック（ビルドなし）
cargo test                            # 全58テストを実行
cargo test -- --nocapture             # テスト出力を表示して実行
cargo run                             # ダミーBootROMで実行（PPUとLCDテスト含む）
cargo run <bootrom_file>              # 実際のBootROMファイルで実行
```

### 機能フラグ
```bash
cargo run --features with_sdl         # SDL2 LCD表示を有効化（160x144、60FPS）
cargo run --features trace_memory     # メモリアクセストレースを有効化
SKIP_LCD_TEST=1 cargo run             # インタラクティブLCD表示テストをスキップ
```

### CI環境での注意
- SDL2未インストール環境では`--features with_sdl`を使わないこと
- `SKIP_LCD_TEST=1`を設定するとSDL2ウィンドウを開かずにテストが完走する

## ディレクトリ構成

```
rustboy/
├── Cargo.toml                  # プロジェクト設定・依存関係
├── Cargo.lock
├── CLAUDE.md                   # このファイル
├── mise.toml                   # Rustツールチェーン設定（1.87.0）
├── doc/                        # フェーズ別設計ドキュメント
│   ├── phase2.md              #   Phase 2: メモリシステム
│   ├── phase3.md              #   Phase 3: CPU実装
│   └── phase4.md              #   Phase 4: PPU・LCD表示
└── src/
    ├── main.rs                 # エントリポイント・テストハーネス
    ├── memory_map.rs           # メモリアドレス定義（dmg, io_registers モジュール）
    ├── peripherals.rs          # メモリバス・アドレスデコード
    ├── lcd.rs                  # SDL2 LCD表示（with_sdl機能フラグ）
    ├── simple_display.rs       # ASCIIフォールバック表示
    ├── memory/
    │   ├── mod.rs              # メモリモジュール公開
    │   ├── bootrom.rs          # BootROM（256B、無効化機能付き）
    │   ├── wram.rs             # Work RAM（8KB、エコー領域対応）
    │   └── hram.rs             # High RAM（127B、スタック操作ヘルパー付き）
    ├── cpu/
    │   ├── mod.rs              # CPUコア（フェッチ・デコード・実行、ALU、CB-prefix）
    │   ├── registers.rs        # 8/16ビットレジスタとフラグ管理
    │   ├── instructions.rs     # 命令定義・オペコード列挙
    │   ├── decoder.rs          # 命令デコーダ
    │   ├── interrupts.rs       # 割り込みコントローラ（VBlank/STAT/Timer/Serial/Joypad）
    │   └── timer.rs            # タイマーシステム（DIV/TIMA/TMA/TAC）
    └── ppu/
        ├── mod.rs              # PPUコア（Mode 0-3遷移、フレームバッファ）
        ├── registers.rs        # LCDレジスタ（LCDC, STAT, SCY, SCX, LY, LYC, BGP）
        ├── vram.rs             # VRAM（8KB、タイルデータ/マップアクセス）
        ├── tiles.rs            # タイルレンダリング（キャッシュ、パレット変換）
        ├── background.rs       # 背景描画（スクロール、折り返し対応）
        └── timing.rs           # PPUタイミング定数
```

## アーキテクチャ

### コンポーネント関係

```
main.rs
  ├── Peripherals (メモリバス — PPU/Timer/割り込みを統合)
  │     ├── BootRom
  │     ├── WorkRam
  │     ├── HighRam
  │     ├── Ppu (VRAM/OAM/レジスタのメモリバスアクセス)
  │     ├── Timer (DIV/TIMA/TMA/TAC)
  │     ├── interrupt_flag (IF: 0xFF0F)
  │     └── interrupt_enable (IE: 0xFFFF)
  ├── Cpu
  │     ├── Registers
  │     ├── Decoder → Instructions
  │     ├── InterruptController (interrupts.rs)
  │     └── ALU (alu_add, alu_sub, etc.)
  └── Display (lcd.rs / simple_display.rs)
```

### メモリシステム
- **Peripherals** (`src/peripherals.rs`) — メインメモリバス。PPU/Timer/IF/IEを統合し、`tick(cycles)`でCPUサイクルに同期して全周辺機器を駆動
- **BootROM** (`src/memory/bootrom.rs`) — 256バイト、0xFF50書き込みで無効化（不可逆）
- **WorkRAM** (`src/memory/wram.rs`) — 8KB（0xC000-0xDFFF）、0xE000-0xFDFFのエコー領域をミラー
- **HighRAM** (`src/memory/hram.rs`) — 127バイト（0xFF80-0xFFFE）、スタック操作ヘルパー付き
- **Memory Map** (`src/memory_map.rs`) — `dmg`モジュール（アドレス定数）と`io_registers`モジュール（I/Oアドレス定数）

### CPUシステム（Sharp LR35902）
- **CPU Core** (`src/cpu/mod.rs`) — フェッチ・デコード・実行サイクル。割り込みチェック→HALT復帰→EI遅延→命令実行。全ALU操作とCB-prefix命令を内蔵
- **Registers** (`src/cpu/registers.rs`) — A,B,C,D,E,H,L,F(8bit) / AF,BC,DE,HL,SP,PC(16bit)。フラグレジスタ下位4bit自動マスク
- **Instructions** (`src/cpu/instructions.rs`) — 命令型列挙、メタデータ（opcode, length, cycles, description）
- **Decoder** (`src/cpu/decoder.rs`) — オペコードデコード
- **Interrupts** (`src/cpu/interrupts.rs`) — 割り込み優先順位処理（VBlank>STAT>Timer>Serial>Joypad）、IF&IEからの保留割り込み検出
- **Timer** (`src/cpu/timer.rs`) — 16bit内部カウンタ、DIV/TIMA/TMA/TAC、falling edge検出によるTIMAインクリメント

### PPUシステム
- **PPU Core** (`src/ppu/mod.rs`) — Mode 0(HBlank), 1(VBlank), 2(OamScan), 3(Drawing)のタイミング遷移。160×144 RGB888フレームバッファ出力
- **VRAM** (`src/ppu/vram.rs`) — 8KB、タイルデータ(2bpp)読み出し、Signed/Unsigned両アドレッシングモード対応
- **Registers** (`src/ppu/registers.rs`) — LCDC(0xFF40), STAT(0xFF41), SCY/SCX, LY, LYC, BGP のビットレベルアクセサ
- **Tiles** (`src/ppu/tiles.rs`) — 8×8タイルレンダリング、LRUキャッシュ（最大64タイル）、4色→RGB888パレット変換
- **Background** (`src/ppu/background.rs`) — 32×32タイルマップ（256×256px）上のスキャンライン描画、スクロール折り返し
- **Timing** (`src/ppu/timing.rs`) — CPU周波数4,194,304Hz、フレーム70224サイクル、目標59.73FPS

### 表示システム
- **LCD** (`src/lcd.rs`) — SDL2ベース、160×144を4倍拡大（640×576ウィンドウ）、VSync/60FPS、キー入力マッピング
- **Simple Display** (`src/simple_display.rs`) — ASCII文字による4色表示、SDL2不要環境用フォールバック

## メモリレイアウト（GameBoy DMG）

| アドレス範囲 | サイズ | 用途 | 実装状況 |
|---|---|---|---|
| 0x0000-0x00FF | 256B | BootROM（0xFF50で無効化） | ✅ |
| 0x0100-0x7FFF | 32KB | カートリッジROM | ❌ 未実装 |
| 0x8000-0x9FFF | 8KB | VRAM（タイルデータ+タイルマップ） | ✅ |
| 0xA000-0xBFFF | 8KB | カートリッジRAM | ❌ 未実装 |
| 0xC000-0xDFFF | 8KB | Work RAM | ✅ |
| 0xE000-0xFDFF | — | WRAMエコー（ミラー） | ✅ |
| 0xFE00-0xFE9F | 160B | OAM（スプライト属性） | ✅ メモリバス統合済み |
| 0xFF00-0xFF7F | 128B | I/Oレジスタ | ✅ PPU/Timer/IF |
| 0xFF80-0xFFFE | 127B | High RAM | ✅ |
| 0xFFFF | 1B | 割り込み有効レジスタ (IE) | ✅ 実装済み |

## 実装済みCPU命令

メイン命令244/256 + CB-prefix全256命令 = 合計500命令:

| カテゴリ | 命令群 |
|---|---|
| 8bitロード | LD r,r' / LD r,n / LD r,(HL) / LD (HL),r / LD (HL),n / LD A,(BC)/(DE)/(nn) / LD (BC)/(DE)/(nn),A / LD (HL+/-),A / LD A,(HL+/-) / LDH |
| 16bitロード | LD rr,nn / LD (nn),SP / LD SP,HL / LD HL,SP+n / PUSH / POP |
| 8bit算術 | ADD / ADC / SUB / SBC / AND / XOR / OR / CP / INC / DEC / DAA / CPL / SCF / CCF |
| 16bit算術 | ADD HL,rr / INC rr / DEC rr / ADD SP,n |
| ジャンプ | JP / JP cc / JR / JR cc / JP (HL) |
| コール・リターン | CALL / CALL cc / RET / RET cc / RETI / RST |
| ローテート | RLCA / RRCA / RLA / RRA |
| 割り込み | DI / EI / HALT |
| CB-prefix | RLC / RRC / RL / RR / SLA / SRA / SWAP / SRL / BIT / RES / SET（全レジスタ+（HL）対応）|

## 主要な設計パターン

- **統合メモリバス**: 全メモリアクセスは`Peripherals`構造体を経由。PPU/Timer/IF/IEを内包し、`tick()`で同期駆動
- **CPU命令実行**: `execute_instruction`のmatchでオペコード直接ディスパッチ。ALUヘルパーメソッドで算術・論理演算を共通化
- **割り込みシステム**: `handle_interrupts()`でIF&IEチェック→PCスタック退避→ハンドラジャンプ。EI命令は1命令遅延
- **PPUモードタイミング**: ハードウェア精確なMode遷移（OamScan→Drawing→HBlank→VBlank）。フレームあたり70224サイクル
- **タイマー**: 16bit内部カウンタのfalling edge検出でTIMAインクリメント。4周波数モード対応
- **条件付きコンパイル**: `#[cfg(feature = "with_sdl")]`でSDL2依存を分離、`#[cfg(feature = "trace_memory")]`でデバッグトレース
- **テスト内蔵**: 各モジュールに`#[cfg(test)] mod tests`を配置
- **日本語コメント**: コードベース全体で日本語コメントを使用

## テスト戦略

全93テスト。各モジュールにユニットテストを内蔵：
- **メモリ**: BootROM読み書き・無効化、WRAM読み書き・アドレス変換、HRAM読み書き・境界値
- **メモリバス**: Peripherals統合テスト（BootROM/WRAM/HRAM/VRAM/OAM/PPUレジスタ/割り込みレジスタ/VBlank tick）
- **メモリマップ**: 領域判定、アドレス情報取得、I/Oレジスタ名解決
- **CPU**: レジスタ、フラグ、命令デコード、LD/ALU/PUSH/POP/CALL/RET/JR/CB/LDH/ローテート/割り込み/HALT復帰
- **割り込み**: マスク、ハンドラアドレス、優先順位、IE&IFフィルタ
- **タイマー**: DIVインクリメント/リセット、TIMA周波数選択、オーバーフロー割り込み
- **PPU**: 生成・初期化、Modeタイミング遷移、VRAMアクセス、タイルデータ読み出し、タイルキャッシュ、色変換、背景描画、スクロール折り返し
- **表示**: SimpleDisplay生成、色変換、FPSカウンタ

## 現在の実装状況

| フェーズ | 内容 | 状態 |
|---|---|---|
| Phase 1-2 | メモリシステム | ✅ 完了 |
| Phase 3 | CPU基礎（11命令） | ✅ 完了 |
| Phase 4 | PPU・LCD表示 | ✅ 完了 |
| Phase 5 | メモリバス統合・割り込み・タイマー・CPU命令拡張（500命令） | ✅ 完了 |
| Phase 6+ | スプライト、ウィンドウ、DMA、APU、入力、カートリッジ | 🔲 未着手 |

## 開発ガイドライン

### 必須ルール
- **TDD**: テストを先に書き、実装を後に書く
- **ファイル末尾**: すべてのファイルの最終行に改行を追加すること
- **テストアドレス**: CPUテストではBootROM競合を避けWRAM領域（0xC000+）を使用
- **ハードウェア精度**: レジスタのビットマスク、メモリアクセス制限、タイミングはGameBoy仕様に忠実に

### メモリアクセス規約
- 新しいメモリアドレスは`memory_map.rs`に定数として追加し、ハードコードしない
- `Peripherals`のread/writeメソッドにアドレスルーティングを追加
- PPUモードに応じたVRAM/OAMアクセス制限を尊重
- 16ビットアクセスはリトルエンディアン

### CPU命令追加の手順
1. `cpu/mod.rs`の`execute_instruction`のmatchに新しいオペコードを追加
2. ALUヘルパーが必要なら`alu_*`メソッドを追加
3. WRAM領域（0xC000+）でテストを作成
4. （旧decoder.rsはPhase 3の遺産、現在の命令実行はmod.rsのmatch直接ディスパッチ）

### PPU開発の注意点
- PPUタイミングはGameBoy仕様に忠実に（Mode 0-3、スキャンライン0-153）
- タイルレンダリングはキャッシュを活用しつつ精度を維持
- 表示はSDL2/ASCIIの両方で動作確認
- LCD表示テストではSKIP_LCD_TEST=1でインタラクティブテストをスキップ可能

### コードスタイル
- コメントは日本語で記述
- `cargo check`でコンパイルエラーがないことを確認
- `cargo test`で全テストがパスすることを確認