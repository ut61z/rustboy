# CLAUDE.md

このファイルはClaude Code (claude.ai/code) がこのリポジトリで作業する際のガイダンスを提供します。

## 共通開発コマンド

### ビルドと実行
- `cargo run` - ダミーBootROMでエミュレータを実行（PPUとLCDテストを含む）
- `cargo run <bootrom_file>` - 実際のBootROMファイルで実行
- `cargo test` - すべてのテストを実行
- `cargo test -- --nocapture` - テスト出力を表示してテストを実行
- `cargo check` - 実行ファイルをビルドせずに高速コンパイルチェック

### PPUと表示機能
- `cargo run --features with_sdl` - SDL2 LCD表示を有効化（160x144、60FPS）
- `SKIP_LCD_TEST=1 cargo run` - インタラクティブなLCD表示テストをスキップ
- `cargo run --features trace_memory` - デバッグ用メモリアクセストレースを有効化

## アーキテクチャ概要

これはRustで書かれたGameBoy (DMG) エミュレータで、現在Phase 4（PPUとLCD表示）を実装しています。アーキテクチャはGameBoyハードウェア構成に従っています。

### システムアーキテクチャ

#### メモリシステム
- **Peripherals** (`src/peripherals.rs`) - アドレスデコードとルーティングを処理するメインメモリバス
- **BootROM** (`src/memory/bootrom.rs`) - 無効化機能付き256バイトブートROM
- **WorkRAM** (`src/memory/wram.rs`) - エコー領域サポート付き8KB作業RAM
- **HighRAM** (`src/memory/hram.rs`) - 127バイト高速RAM
- **Memory Map** (`src/memory_map.rs`) - 集約化されたメモリアドレス定義とユーティリティ

#### CPUシステム
- **CPU Core** (`src/cpu/mod.rs`) - フェッチ・デコード・実行サイクル付きメインCPU
- **Registers** (`src/cpu/registers.rs`) - 8ビット/16ビットレジスタとフラグ管理
- **Instructions** (`src/cpu/instructions.rs`) - 命令定義とオペコード
- **Decoder** (`src/cpu/decoder.rs`) - 命令デコードとエラーハンドリング

#### PPUシステム（Phase 4）
- **PPU Core** (`src/ppu/mod.rs`) - Mode 0-3タイミング付きピクチャー処理ユニット
- **VRAM** (`src/ppu/vram.rs`) - タイルデータとタイルマップ管理付き8KBビデオRAM
- **Registers** (`src/ppu/registers.rs`) - LCD制御レジスタ（LCDC、STAT、SCY、SCX等）
- **Tiles** (`src/ppu/tiles.rs`) - キャッシュ付きタイルレンダリングシステム
- **Background** (`src/ppu/background.rs`) - スクロールサポート付き背景描画
- **Timing** (`src/ppu/timing.rs`) - PPUタイミング定数とフレームレート制御

#### 表示システム
- **LCD Display** (`src/lcd.rs`) - SDL2ベースのリアルタイム表示（160x144、60FPS）
- **Simple Display** (`src/simple_display.rs`) - デバッグ用ASCIIベースフォールバック表示

### 主要な設計パターン
- メモリコンポーネントはアドレスデコードを処理する`Peripherals`バスを通じてアクセス
- すべてのメモリアドレスは`memory_map.rs`モジュールの定数を使用（例：`dmg::BOOTROM_START`）
- CPU命令は分離されたデコードと実行フェーズでモジュラー設計を使用
- 個別8ビットと結合16ビットインターフェースの両方でレジスタアクセス
- デバッグと分析用のメモリ統計追跡
- デバッグトレース用の条件付きコンパイル機能
- ファイル操作でResultタイプを使用したエラーハンドリング
- PPUタイミングはGameBoy仕様に従う（Mode 0-3、フレームあたり70224サイクル）
- ハードウェア精度のためPPUモードによるVRAMアクセス制御
- タイルレンダリングはGameBoyの8x8ピクセルタイルシステムを使用
- 背景スクロールは32x32タイルマップ（256x256ピクセル）で折り返し
- LCD表示タイミングは適切なフレーム同期で60FPSを目標

### メモリレイアウト（GameBoy DMG）
- 0x0000-0x00FF: BootROM（256B） - 0xFF50レジスタで無効化
- 0x0100-0x7FFF: カートリッジROM（未実装）
- 0x8000-0x9FFF: ビデオRAM（8KB） - タイルデータとタイルマップ
- 0xA000-0xBFFF: カートリッジRAM（未実装）
- 0xC000-0xDFFF: 作業RAM（8KB）
- 0xE000-0xFDFF: 作業RAMエコー（WRAMのミラー）
- 0xFF00-0xFF7F: I/Oレジスタ（PPUレジスタ実装済み：LCDC、STAT、SCY、SCX、LY、LYC、BGP）
- 0xFF80-0xFFFE: 高速RAM（127B）
- 0xFFFF: 割り込み有効レジスタ

### テスト戦略
- 各メモリコンポーネントは独自のテストモジュールを持つ
- CPUコンポーネントはレジスタ、命令、デコーダの包括的なユニットテストを持つ
- PPUコンポーネントはVRAM、レジスタ、タイミング、レンダリングのテストを持つ
- `Peripherals`の統合テストでメモリバス機能を検証
- CPU実行テストでフェッチ・デコード・実行サイクルを検証
- PPUテストでMode遷移、VBlank生成、フレームレンダリングを検証
- LCD表示テストでリアルタイムレンダリング機能をデモンストレーション
- テストは通常動作とエッジケースの両方をカバー
- 検証にメモリ統計を使用

## 開発メモ

### 現在の実装状況
このエミュレータはPPUとLCD表示機能でPhase 4開発を完了しています。現在の機能：
- 完全なメモリシステム（Phase 2）
- フェッチ・デコード・実行サイクル付き基本CPU（Phase 3）
- リアルタイムレンダリング付き完全PPU実装（Phase 4）
- 基本命令サポート：NOP、LD（即値）、JP、JR
- 8ビット/16ビットアクセスパターン付きレジスタ管理
- シンプルプログラム実行とループ検出
- VRAMとOAMメモリ管理（8KB + 160B）
- LCD制御レジスタとPPUタイミング
- スクロールサポート付き背景タイルレンダリング
- SDL2ベース表示システム（160x144、60FPS）
- SDL2なしシステム用ASCIIフォールバック表示
- GameBoy精確4色パレットシステム

次のフェーズではスプライトレンダリング、ウィンドウサポート、DMA転送、サウンド、入力処理を追加予定。

### コード構成
- メインエントリポイントはメモリシステム、CPU、PPU機能をデモンストレーション
- 日本語コメントを全体で使用（元の開発言語）
- 機能フラグでオプション機能を制御：
  - `trace_memory` - メモリアクセストレースを有効化
  - `with_sdl` - SDL2表示サポートを有効化
- すべてのメモリアクセスは集約化された`Peripherals`構造体を通じて実行
- CPUテストはBootROMアクセス競合を避けるためWRAM領域で実行
- PPUモジュールは機能別に構成（VRAM、レジスタ、レンダリング、タイミング）
- 表示システムはSDL2とASCII出力モードの両方をサポート

### 重要な考慮事項
- メモリアクセスパターンはGameBoyハードウェア動作に従う
- BootROMは一度無効化されるとアクセス不可（ハードウェア精確）
- 作業RAMエコー領域はハードウェア互換性のためWRAMをミラー
- 16ビットメモリアクセスはリトルエンディアンバイトオーダーを使用
- CPUレジスタは個別8ビットと結合16ビットアクセスの両方をサポート
- フラグレジスタ（F）は下位4ビットを自動マスク（ハードウェア精確）
- PCとSPは命令実行中に自動管理

### 実装済み命令
現在のCPUは以下のGameBoy命令をサポート：
- `0x00` NOP - 操作なし
- `0x3E` LD A,n - Aレジスタに即値ロード
- `0x06` LD B,n - Bレジスタに即値ロード
- `0x0E` LD C,n - Cレジスタに即値ロード
- `0x16` LD D,n - Dレジスタに即値ロード
- `0x1E` LD E,n - Eレジスタに即値ロード
- `0x26` LD H,n - Hレジスタに即値ロード
- `0x2E` LD L,n - Lレジスタに即値ロード
- `0x31` LD SP,nn - スタックポインタに即値ロード
- `0xC3` JP nn - 絶対ジャンプ
- `0x18` JR n - 相対ジャンプ

### 開発ガイドライン
- TDDで実装すること
- git commit メッセージはsemanticであること
    - `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore` をprefixとして用いること
    - 例: `feat: Aを新規実装` , `chore: ライブラリをバージョンアップ`
- すべてのファイルは最終行に空の行を追加すること
- CPUテストはBootROM競合を避けるためWRAM領域（0xC000+）を使用すること
- レジスタペア関数はハードウェア精確ビットマスキングを維持すること
- PPUテストは機能とタイミング精度の両方を検証すること
- LCD表示テスト時はSKIP_LCD_TEST環境変数でインタラクティブテストをスキップ
- SDL2機能は条件付きとし、利用不可時はASCII表示にフォールバック
- メモリアクセスパターンはPPUモード制限に従うこと
- タイルレンダリングは最適化すべきだがGameBoy精度を維持すること