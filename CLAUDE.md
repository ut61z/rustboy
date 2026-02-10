# CLAUDE.md

このファイルはClaude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## プロジェクト概要

RustBoy — Rustで書かれたGameBoy (DMG) エミュレータ。ハードウェア精確なエミュレーションを目指し、段階的に開発中。現在Phase 4（PPUとLCD表示）まで完了、58テスト全パス。

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
    │   ├── mod.rs              # CPUコア（フェッチ・デコード・実行）
    │   ├── registers.rs        # 8/16ビットレジスタとフラグ管理
    │   ├── instructions.rs     # 命令定義・オペコード列挙
    │   └── decoder.rs          # 命令デコーダ（CB-prefix対応予定）
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
  ├── Peripherals (メモリバス)
  │     ├── BootRom
  │     ├── WorkRam
  │     └── HighRam
  ├── Cpu
  │     ├── Registers
  │     └── Decoder → Instructions
  ├── Ppu
  │     ├── PpuRegisters
  │     ├── Vram
  │     ├── TileRenderer (tiles.rs)
  │     └── BackgroundRenderer (background.rs)
  └── Display (lcd.rs / simple_display.rs)
```

### メモリシステム
- **Peripherals** (`src/peripherals.rs`) — アドレスデコードとルーティングを処理するメインメモリバス。読み書き統計を追跡
- **BootROM** (`src/memory/bootrom.rs`) — 256バイト、0xFF50書き込みで無効化（不可逆）
- **WorkRAM** (`src/memory/wram.rs`) — 8KB（0xC000-0xDFFF）、0xE000-0xFDFFのエコー領域をミラー
- **HighRAM** (`src/memory/hram.rs`) — 127バイト（0xFF80-0xFFFE）、スタック操作ヘルパー付き
- **Memory Map** (`src/memory_map.rs`) — `dmg`モジュール（アドレス定数）と`io_registers`モジュール（I/Oアドレス定数）

### CPUシステム（Sharp LR35902）
- **CPU Core** (`src/cpu/mod.rs`) — `step()`メソッドでフェッチ・デコード・実行サイクル。IME、HALT状態、命令カウンタ管理
- **Registers** (`src/cpu/registers.rs`) — A,B,C,D,E,H,L,F(8bit) / AF,BC,DE,HL,SP,PC(16bit)。フラグレジスタ下位4bit自動マスク
- **Instructions** (`src/cpu/instructions.rs`) — 命令型列挙、メタデータ（opcode, length, cycles, description）
- **Decoder** (`src/cpu/decoder.rs`) — オペコードデコード、CB-prefix拡張用構造あり

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
| 0xFE00-0xFE9F | 160B | OAM（スプライト属性） | ✅ メモリのみ |
| 0xFF00-0xFF7F | 128B | I/Oレジスタ | ✅ 部分的 |
| 0xFF80-0xFFFE | 127B | High RAM | ✅ |
| 0xFFFF | 1B | 割り込み有効レジスタ | ✅ メモリのみ |

## 実装済みCPU命令（11命令）

| オペコード | 命令 | 説明 |
|---|---|---|
| `0x00` | NOP | 操作なし |
| `0x06` | LD B,n | Bレジスタに即値ロード |
| `0x0E` | LD C,n | Cレジスタに即値ロード |
| `0x16` | LD D,n | Dレジスタに即値ロード |
| `0x1E` | LD E,n | Eレジスタに即値ロード |
| `0x26` | LD H,n | Hレジスタに即値ロード |
| `0x2E` | LD L,n | Lレジスタに即値ロード |
| `0x31` | LD SP,nn | スタックポインタに16ビット即値ロード |
| `0x3E` | LD A,n | Aレジスタに即値ロード |
| `0x18` | JR n | 相対ジャンプ（符号付き8ビットオフセット） |
| `0xC3` | JP nn | 絶対ジャンプ（16ビットアドレス） |

## 主要な設計パターン

- **メモリバス抽象化**: 全メモリアクセスは`Peripherals`構造体を経由。アドレスは`memory_map::dmg`の定数を使用
- **モジュラーCPU**: デコード(`decoder.rs`)と実行(`mod.rs`)を分離。命令追加時はdecoder + 実行ロジックの両方を更新
- **PPUモードタイミング**: ハードウェア精確なMode遷移（OamScan→Drawing→HBlank→VBlank）。フレームあたり70224サイクル
- **条件付きコンパイル**: `#[cfg(feature = "with_sdl")]`でSDL2依存を分離、`#[cfg(feature = "trace_memory")]`でデバッグトレース
- **テスト内蔵**: 各モジュールに`#[cfg(test)] mod tests`を配置
- **日本語コメント**: コードベース全体で日本語コメントを使用

## テスト戦略

全58テスト。各モジュールにユニットテストを内蔵：
- **メモリ**: BootROM読み書き・無効化、WRAM読み書き・アドレス変換、HRAM読み書き・境界値
- **メモリバス**: Peripherals統合テスト（BootROM/WRAM/HRAM/16ビットアクセス/WRAMエコー）
- **メモリマップ**: 領域判定、アドレス情報取得、I/Oレジスタ名解決
- **CPU**: レジスタ操作、フラグ操作、16ビットペアアクセス、命令デコード、実行サイクル
- **PPU**: 生成・初期化、Modeタイミング遷移、VRAMアクセス、タイルデータ読み出し、タイルキャッシュ、色変換、背景描画、スクロール折り返し
- **表示**: SimpleDisplay生成、色変換、FPSカウンタ

## 現在の実装状況

| フェーズ | 内容 | 状態 |
|---|---|---|
| Phase 1-2 | メモリシステム | ✅ 完了 |
| Phase 3 | CPU基礎（11命令） | ✅ 完了 |
| Phase 4 | PPU・LCD表示 | ✅ 完了 |
| Phase 5+ | スプライト、ウィンドウ、DMA、APU、入力、割り込み完全実装、追加CPU命令 | 🔲 未着手 |

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
1. `instructions.rs`に命令型を追加
2. `decoder.rs`にデコードロジックを追加
3. `cpu/mod.rs`の`execute`に実行ロジックを追加
4. WRAM領域（0xC000+）でテストを作成

### PPU開発の注意点
- PPUタイミングはGameBoy仕様に忠実に（Mode 0-3、スキャンライン0-153）
- タイルレンダリングはキャッシュを活用しつつ精度を維持
- 表示はSDL2/ASCIIの両方で動作確認
- LCD表示テストではSKIP_LCD_TEST=1でインタラクティブテストをスキップ可能

### コードスタイル
- コメントは日本語で記述
- `cargo check`でコンパイルエラーがないことを確認
- `cargo test`で全テストがパスすることを確認