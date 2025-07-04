# Phase 3: 最小限CPU実装 - チェックリスト

## 🎯 目標
BootROMの最初の数命令を実行できる最小限のCPUを実装する

## 📋 実装項目

### 1. CPUレジスタシステム強化
- [x] **完全なレジスタ実装**
  - [x] 8bitレジスタ (A, B, C, D, E, F, H, L)
  - [x] 16bitレジスタペア (AF, BC, DE, HL, SP, PC)
  - [x] フラグレジスタ操作 (Z, N, H, C)
  - [x] レジスタアクセサ関数

### 2. 命令フェッチサイクル
- [x] **基本フェッチ機能**
  - [x] PCからオペコード読み取り
  - [x] PC自動インクリメント
  - [x] 命令長の管理

### 3. 命令デコーダ
- [x] **オペコード解析**
  - [x] 基本命令のデコード
  - [x] CB prefixed命令対応（フレームワーク実装済み）
  - [x] エラーハンドリング

### 4. 基本命令実装
- [x] **必須命令**
  - [x] `0x00` NOP (何もしない)
  - [x] `0x3E` LD A, n (Aレジスタに即値ロード)
  - [x] `0x06` LD B, n (Bレジスタに即値ロード)
  - [x] `0x0E` LD C, n (Cレジスタに即値ロード)
  - [x] `0x16` LD D, n (Dレジスタに即値ロード)
  - [x] `0x1E` LD E, n (Eレジスタに即値ロード)
  - [x] `0x26` LD H, n (Hレジスタに即値ロード)
  - [x] `0x2E` LD L, n (Lレジスタに即値ロード)

- [x] **ジャンプ命令**
  - [x] `0xC3` JP nn (絶対ジャンプ)
  - [x] `0x18` JR n (相対ジャンプ)

- [x] **スタック操作**
  - [x] `0x31` LD SP, nn (スタックポインタ設定)

### 5. CPU実行ループ
- [x] **メインループ**
  - [x] fetch → decode → execute サイクル
  - [x] 命令実行カウンタ
  - [x] デバッグ出力

### 6. テストとデバッグ
- [x] **単体テスト**
  - [x] レジスタ操作テスト
  - [x] 各命令の動作テスト
  - [x] フラグ操作テスト

- [x] **統合テスト**
  - [x] 簡単なプログラム実行
  - [x] BootROM最初の数命令実行
  - [x] デバッグ情報出力

## 🚀 実装順序

### Step 1: CPUコア構造
1. `src/cpu/mod.rs` - CPUモジュール統合
2. `src/cpu/registers.rs` - レジスタシステム
3. `src/cpu/instructions.rs` - 命令定義

### Step 2: 命令システム
1. フェッチサイクル実装
2. デコーダ実装  
3. 基本命令実装

### Step 3: 統合とテスト
1. メインループ統合
2. テストプログラム作成
3. BootROM実行テスト

## ✅ 完了判定基準

### 最小限の成功条件
- [x] NOP命令が実行できる
- [x] LD A, n 命令が実行できる
- [x] JP nn 命令が実行できる
- [x] 簡単なプログラムループが実行できる

### 理想的な完了条件
- [x] BootROMの最初の10命令が実行できる
- [x] レジスタ状態がデバッグ出力される
- [x] 命令実行トレースが出力される
- [x] 全テストがパスする

## 📁 作成済みファイル

```
src/cpu/
├── mod.rs           # CPUモジュール統合 ✅
├── registers.rs     # レジスタシステム ✅
├── instructions.rs  # 命令実装 ✅
└── decoder.rs       # 命令デコーダ ✅
```

## 🕐 予想所要時間
- **Step 1**: 1-2時間 (レジスタシステム)
- **Step 2**: 2-3時間 (命令システム)
- **Step 3**: 1時間 (統合・テスト)
- **合計**: 4-6時間

## 🎉 Phase 3 完了

**実装された機能:**
- ✅ 完全なレジスタシステム（8bit/16bitレジスタ、フラグ操作）
- ✅ 命令フェッチ・デコード・実行サイクル
- ✅ 11個の基本命令（NOP、LD、JP、JR）
- ✅ 包括的なテストスイート
- ✅ デバッグ機能とトレース出力

**獲得したスキル:**
- CPUアーキテクチャの理解
- 命令セットアーキテクチャ (ISA) の実装
- フェッチ・デコード・実行サイクル
- アセンブリ言語とマシン語の関係
- ハードウェアエミュレーションの基礎
