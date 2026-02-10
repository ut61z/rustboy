# Phase 5: メモリバス統合・割り込み・CPU命令拡張 - チェックリスト

## 🎯 目標
PPUをメモリバスに統合し、割り込みシステムとタイマーを実装し、CPU命令を大幅に拡張して
BootROMを完走できるレベルのエミュレータにする

## 📋 実装項目

### 1. PPU-メモリバス統合
- [ ] **Peripheralsにppu統合**
  - [ ] `Peripherals`構造体にPpuを追加
  - [ ] VRAM (0x8000-0x9FFF) のread/writeルーティング
  - [ ] OAM (0xFE00-0xFE9F) のread/writeルーティング
  - [ ] PPUレジスタ (0xFF40-0xFF4B) のread/writeルーティング
  - [ ] PPUモードに応じたアクセス制御

### 2. 割り込みシステム
- [ ] **割り込みコントローラ**
  - [ ] IF (0xFF0F) 割り込みフラグレジスタ
  - [ ] IE (0xFFFF) 割り込み許可レジスタ
  - [ ] 割り込み優先順位処理 (VBlank > STAT > Timer > Serial > Joypad)
  - [ ] 割り込みハンドラ（PC退避→ジャンプ→IME無効化）

- [ ] **割り込みCPU連携**
  - [ ] `EI` (0xFB) 命令: IME有効化（1命令遅延）
  - [ ] `DI` (0xF3) 命令: IME無効化
  - [ ] `RETI` (0xD9) 命令: 割り込みリターン
  - [ ] `HALT` (0x76) 修正: 割り込みで復帰
  - [ ] VBlank割り込み発火 (PPU → CPU)
  - [ ] STAT割り込み発火

### 3. タイマーシステム
- [ ] **タイマーレジスタ**
  - [ ] DIV (0xFF04) 分周器（16bit内部カウンタ、上位8bit読み出し）
  - [ ] TIMA (0xFF05) タイマーカウンタ
  - [ ] TMA (0xFF06) タイマーモジュロ（オーバーフロー時のリロード値）
  - [ ] TAC (0xFF07) タイマー制御（有効/無効、周波数選択）

- [ ] **タイマー動作**
  - [ ] DIVの16bitインクリメント（4MHzクロック毎）
  - [ ] DIV書き込み時のゼロリセット
  - [ ] TACの周波数選択（CPU/1024, CPU/16, CPU/64, CPU/256）
  - [ ] TIMAオーバーフロー → TMAリロード → Timer割り込み

### 4. CPU命令拡張

#### 4a. 8ビットロード命令
- [ ] LD r, r' (0x40-0x7F) — レジスタ間コピー（HALT除く）
- [ ] LD r, (HL) — メモリからレジスタへ
- [ ] LD (HL), r — レジスタからメモリへ
- [ ] LD (HL), n (0x36) — メモリに即値
- [ ] LD A, (BC) (0x0A) / LD A, (DE) (0x1A)
- [ ] LD (BC), A (0x02) / LD (DE), A (0x12)
- [ ] LDH A, (n) (0xF0) / LDH (n), A (0xE0) — 0xFF00+nアクセス
- [ ] LD A, (C) (0xF2) / LD (C), A (0xE2) — 0xFF00+Cアクセス
- [ ] LD A, (nn) (0xFA) / LD (nn), A (0xEA)
- [ ] LD (HL+), A (0x22) / LD A, (HL+) (0x2A)
- [ ] LD (HL-), A (0x32) / LD A, (HL-) (0x3A)

#### 4b. 16ビットロード命令
- [ ] LD rr, nn — 16bitレジスタに即値（BC:0x01, DE:0x11, HL:0x21）
- [ ] LD (nn), SP (0x08)
- [ ] LD SP, HL (0xF9)
- [ ] LD HL, SP+n (0xF8)
- [ ] PUSH rr (BC:0xC5, DE:0xD5, HL:0xE5, AF:0xF5)
- [ ] POP rr (BC:0xC1, DE:0xD1, HL:0xE1, AF:0xF1)

#### 4c. 8ビット算術・論理演算
- [ ] ADD A, r / ADD A, (HL) / ADD A, n (0x80-0x87, 0xC6)
- [ ] ADC A, r / ADC A, (HL) / ADC A, n (0x88-0x8F, 0xCE)
- [ ] SUB r / SUB (HL) / SUB n (0x90-0x97, 0xD6)
- [ ] SBC A, r / SBC A, (HL) / SBC A, n (0x98-0x9F, 0xDE)
- [ ] AND r / AND (HL) / AND n (0xA0-0xA7, 0xE6)
- [ ] XOR r / XOR (HL) / XOR n (0xA8-0xAF, 0xEE)
- [ ] OR r / OR (HL) / OR n (0xB0-0xB7, 0xF6)
- [ ] CP r / CP (HL) / CP n (0xB8-0xBF, 0xFE)
- [ ] INC r / INC (HL) (0x04,0x0C,0x14,0x1C,0x24,0x2C,0x34,0x3C)
- [ ] DEC r / DEC (HL) (0x05,0x0D,0x15,0x1D,0x25,0x2D,0x35,0x3D)
- [ ] DAA (0x27), CPL (0x2F), SCF (0x37), CCF (0x3F)

#### 4d. 16ビット算術
- [ ] ADD HL, rr (BC:0x09, DE:0x19, HL:0x29, SP:0x39)
- [ ] INC rr (BC:0x03, DE:0x13, HL:0x23, SP:0x33)
- [ ] DEC rr (BC:0x0B, DE:0x1B, HL:0x2B, SP:0x3B)
- [ ] ADD SP, n (0xE8)

#### 4e. ジャンプ・コール・リターン
- [ ] JP cc, nn (NZ:0xC2, Z:0xCA, NC:0xD2, C:0xDA)
- [ ] JR cc, n (NZ:0x20, Z:0x28, NC:0x30, C:0x38)
- [ ] JP (HL) (0xE9)
- [ ] CALL nn (0xCD)
- [ ] CALL cc, nn (NZ:0xC4, Z:0xCC, NC:0xD4, C:0xDC)
- [ ] RET (0xC9)
- [ ] RET cc (NZ:0xC0, Z:0xC8, NC:0xD0, C:0xD8)
- [ ] RST n (0xC7,0xCF,0xD7,0xDF,0xE7,0xEF,0xF7,0xFF)

#### 4f. ローテート・シフト（メイン）
- [ ] RLCA (0x07), RLA (0x17), RRCA (0x0F), RRA (0x1F)

#### 4g. CB-prefix命令
- [ ] RLC r (0xCB 0x00-0x07)
- [ ] RRC r (0xCB 0x08-0x0F)
- [ ] RL r (0xCB 0x10-0x17)
- [ ] RR r (0xCB 0x18-0x1F)
- [ ] SLA r (0xCB 0x20-0x27)
- [ ] SRA r (0xCB 0x28-0x2F)
- [ ] SWAP r (0xCB 0x30-0x37)
- [ ] SRL r (0xCB 0x38-0x3F)
- [ ] BIT b, r (0xCB 0x40-0x7F)
- [ ] RES b, r (0xCB 0x80-0xBF)
- [ ] SET b, r (0xCB 0xC0-0xFF)

#### 4h. その他
- [ ] HALT (0x76)
- [ ] DI (0xF3), EI (0xFB), RETI (0xD9)

### 5. PeripheralsでのPPU step統合
- [ ] **CPUサイクルに同期したPPU駆動**
  - [ ] CPU step実行後、消費サイクル分PPUを進める
  - [ ] `Peripherals`にtick(cycles)メソッド追加
  - [ ] VBlank/STAT割り込みのIF反映

## 🚀 実装順序

### Step 1: PPU-メモリバス統合
1. `Peripherals`にPpuフィールドを追加
2. read/writeにVRAM, OAM, PPUレジスタルーティングを追加
3. 既存テストの修正・新規テストの追加

### Step 2: 割り込みシステム
1. `src/cpu/interrupts.rs` — InterruptController構造体
2. `Peripherals`にIF/IEレジスタを接続
3. CPU stepに割り込みチェックを追加
4. EI/DI/RETI命令の実装

### Step 3: タイマー
1. `src/cpu/timer.rs` — Timer構造体
2. `Peripherals`にTimerを統合
3. DIV/TIMA/TMA/TACのread/write
4. Timer割り込み発火

### Step 4: CPU命令拡張
1. 8bitロード命令（レジスタ間、メモリ間）
2. 16bitロード命令（PUSH/POP含む）
3. 算術・論理演算
4. ジャンプ・コール・リターン
5. ローテート・シフト
6. CB-prefix命令

### Step 5: 統合・同期
1. CPUサイクルに同期してPPU/Timerを駆動
2. 割り込みフロー全体の統合テスト
3. BootROM実行テスト

## ✅ 完了判定基準

### 最小限の成功条件
- [ ] PPUレジスタがメモリバス経由でread/write可能
- [ ] VBlank割り込みがCPUに発火される
- [ ] タイマーが動作しTimer割り込みが発火
- [ ] CPU命令が50以上実装されている
- [ ] PUSH/POP/CALL/RETが正常動作

### 理想的な完了条件
- [ ] BootROM (256バイト) が完走する
- [ ] CB-prefix命令がすべて実装されている
- [ ] 全割り込みタイプが動作する
- [ ] タイマーの4周波数モードが正確に動作
- [ ] CPUとPPUが同期して動作する

## 📁 作成・変更予定ファイル

```
src/cpu/
├── mod.rs              # 命令実行拡張、割り込みチェック追加
├── registers.rs        # 変更なし
├── instructions.rs     # 命令型追加
├── decoder.rs          # デコードロジック拡張
├── interrupts.rs       # 【新規】割り込みコントローラ
└── timer.rs            # 【新規】タイマーシステム

src/peripherals.rs      # PPU統合、IF/IE、Timer統合
src/ppu/mod.rs          # 割り込みフラグのIF反映対応
```

## 🔧 既存インフラ（再利用可能）

- OAMメモリ: 160バイト配列が`Ppu`に既存
- PPUレジスタアクセサ: `PpuRegisters`にビットフラグ操作が実装済み
- VRAMアクセス: `Vram`構造体にread/writeが実装済み
- I/Oレジスタ定数: `memory_map::io_registers`にDMA, OBP0/1, WX/WY, IF, Timer等が定義済み
- CPU IMEフラグ: `Cpu.ime`フィールドが既存
- CPU HALTフラグ: `Cpu.halted`フィールドが既存
- VBlank/STAT割り込みフラグ: `Ppu.vblank_interrupt`/`Ppu.stat_interrupt`が既存
