# Phase 5: スプライト実装 - 詳細チェックリスト

## 🎯 目標
GameBoyのスプライト（オブジェクト）描画を完全実装し、キャラクターやアイテムが画面に表示されるようにする

## 📋 実装項目

### 1. スプライト基礎理解
- [x] **スプライトシステム理解**
  - [x] 40個のスプライト（OAM内）
  - [x] 1スキャンライン最大10スプライト
  - [x] 8x8 / 8x16 ピクセルサイズ
  - [x] X/Y座標オフセット（-8, -16）

### 2. OAMデータ構造
- [x] **Sprite構造体定義**
  - [x] Y座標（バイト0）
  - [x] X座標（バイト1）  
  - [x] タイルインデックス（バイト2）
  - [x] 属性フラグ（バイト3）

- [x] **属性フラグ解析**
  - [x] bit 7: OBJ-to-BG Priority (0=前面, 1=BG優先)
  - [x] bit 6: Y flip (垂直反転)
  - [x] bit 5: X flip (水平反転)
  - [x] bit 4: Palette (0=OBP0, 1=OBP1)
  - [x] bit 3-0: 未使用

### 3. スプライト描画エンジン
- [x] **スキャンライン処理**
  - [x] 現在ライン（LY）のスプライト検索
  - [x] 最大10スプライト制限
  - [x] X座標ソート（優先度決定）

- [x] **タイルデータ取得**
  - [x] 8x8モード: タイルインデックス直接使用
  - [x] 8x16モード: 偶数インデックス（上半分）+奇数（下半分）
  - [x] Y flipの行計算
  - [x] タイルデータの読み込み

- [x] **ピクセル描画**
  - [x] X flipの列計算
  - [x] 透明ピクセル（色0）の処理
  - [x] パレット適用（OBP0/OBP1）
  - [x] BG優先度チェック

### 4. 描画優先度システム
- [ ] **レイヤー順序**
  - [ ] 背景（BG）
  - [ ] ウィンドウ（Window）
  - [ ] スプライト（OBJ）

- [ ] **スプライト内優先度**
  - [ ] X座標の小さい順
  - [ ] 同じX座標の場合はOAMインデックス順
  - [ ] OBJ-to-BG Priorityの処理

### 5. OAM DMA実装
- [ ] **DMA制御**
  - [ ] 0xFF46レジスタ処理
  - [ ] ソースアドレス計算（xx00-xx9F）
  - [ ] 160バイト転送
  - [ ] 160 M-cycle 転送時間

- [ ] **CPU制御**
  - [ ] DMA中のCPU停止
  - [ ] HRAMのみアクセス可能
  - [ ] DMA完了検出

### 6. PPUレジスタ統合
- [ ] **peripherals.rs統合**
  - [ ] OAMメモリアクセス制御
  - [ ] スプライトサイズ制御（LCDC bit 2）
  - [ ] スプライト有効/無効（LCDC bit 1）
  - [ ] パレットレジスタ（OBP0, OBP1）

### 7. デバッグ・テスト機能
- [ ] **スプライトデバッグ**
  - [ ] OAMダンプ機能
  - [ ] スプライト可視化
  - [ ] 優先度確認機能
  - [ ] DMA状態監視

## 🚀 実装順序

### Step 1: データ構造とOAM (1-2時間) ✅ 完了
```rust
// src/ppu/sprites.rs
struct Sprite {
    y: u8,
    x: u8, 
    tile_index: u8,
    flags: u8,
}

// OAM parsing
fn parse_oam(&self) -> [Sprite; 40]
```

### Step 2: スプライト検索・ソート (1-2時間) ✅ 完了
```rust
// 現在スキャンラインのスプライト検索
fn find_sprites_on_line(&self, line: u8) -> Vec<Sprite>

// X座標でソート（優先度）
fn sort_sprites_by_priority(&self, sprites: &mut [Sprite])
```

### Step 3: タイル描画 (2-3時間) ✅ 完了
```rust
// スプライトタイルの描画
fn render_sprite_tile(&self, sprite: &Sprite, line: u8) -> [u8; 8]

// フリップ処理
fn apply_flip(&self, pixels: &mut [u8], x_flip: bool)
```

### Step 4: OAM DMA (1-2時間)
```rust
// src/ppu/dma.rs
struct OamDma {
    active: bool,
    source: u16,
    cycles_remaining: u16,
}
```

### Step 5: 統合とテスト (1時間)
```rust
// PPU描画ループに統合
fn render_scanline(&mut self, line: u8) {
    self.render_background(line);
    self.render_sprites(line);  // NEW!
}
```

## ✅ 完了判定基準

### 最小限の成功条件
- [x] 1つのスプライトが正しい位置に表示される（実装済み - Step 3）
- [x] スプライトが背景の上に描画される（実装済み - Step 3）
- [x] 透明ピクセル（色0）が正しく処理される（実装済み - Step 3）
- [ ] OAM DMAが基本動作する

### 理想的な完了条件
- [x] 複数スプライトが正しく表示される（実装済み - Step 2-3）
- [x] X/Y フリップが正確に動作する（実装済み - Step 3、テスト済み）
- [x] スプライト優先度が正確（実装済み - Step 2、テスト済み）
- [x] 8x16スプライトが正しく表示される（実装済み - Step 3）
- [x] BG優先度が正確に処理される（実装済み - Step 3）
- [x] 1ライン10スプライト制限が機能する（実装済み - Step 2、テスト済み）

## 📁 新規作成ファイル

```
src/ppu/
├── sprites.rs       # NEW: スプライト描画エンジン
├── dma.rs          # NEW: OAM DMA制御
└── priority.rs     # NEW: 描画優先度管理（オプション）

tests/
└── sprite_tests.rs # NEW: スプライトテスト
```

## 🎮 テスト用データ

### テスト用スプライト配置
```rust
// OAMにテスト用スプライトを配置
fn setup_test_sprites(&mut self) {
    // スプライト0: 画面中央
    self.oam[0] = 16 + 72;  // Y = 72 (画面座標)
    self.oam[1] = 8 + 80;   // X = 80 (画面座標)
    self.oam[2] = 0x01;     // タイル1
    self.oam[3] = 0x00;     // フラグなし
    
    // スプライト1: 右下
    self.oam[4] = 16 + 120; // Y = 120
    self.oam[5] = 8 + 120;  // X = 120  
    self.oam[6] = 0x02;     // タイル2
    self.oam[7] = 0x20;     // X flip
}
```

### VRAM テストパターン
```rust
// タイル1: ハート型
let heart_tile = [
    0b01101100,
    0b11111110, 
    0b11111110,
    0b01111100,
    0b00111000,
    0b00010000,
    0b00000000,
    0b00000000,
];
```

## 🔧 実装のコツ

### 1. **段階的実装**
1. まず固定位置に1つのスプライト
2. 複数スプライト
3. フリップ機能
4. 優先度
5. DMA

### 2. **デバッグ支援**
```rust
// スプライト情報出力
fn debug_sprites(&self) {
    for i in 0..40 {
        let sprite = self.get_sprite(i);
        if sprite.y != 0 {
            println!("Sprite {}: X={}, Y={}, Tile={:02X}, Flags={:02X}", 
                     i, sprite.x, sprite.y, sprite.tile_index, sprite.flags);
        }
    }
}
```

### 3. **座標変換の注意**
```rust
// GameBoyスプライト座標は-8, -16オフセット
let screen_x = sprite.x.wrapping_sub(8);
let screen_y = sprite.y.wrapping_sub(16);

// 画面外チェック
if screen_x >= 160 || screen_y >= 144 {
    continue;
}
```

## 🕐 予想所要時間
- **Step 1**: 1-2時間 (データ構造)
- **Step 2**: 1-2時間 (スプライト検索)
- **Step 3**: 2-3時間 (描画エンジン)
- **Step 4**: 1-2時間 (DMA実装)
- **Step 5**: 1時間 (統合)
- **合計**: 6-10時間

## 🎉 Phase 5完了後の成果

### 視覚的な改善
- **Phase 4**: 背景のみ
- **Phase 5**: 背景 + **動くキャラクター・アイテム**

### 動作可能なゲーム要素
- テトリスの落下ブロック
- ポケモンのキャラクター
- マリオの敵キャラ
- RPGのキャラクタースプライト

**スプライト実装でGameBoyエミュレータが本格的なゲーム機になります！** 🎮✨