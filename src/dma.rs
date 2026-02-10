// src/dma.rs
// GameBoy OAM DMA転送コントローラ
//
// DMAレジスタ (0xFF46) に書き込むと、指定アドレスから160バイトをOAMにコピー。
// 転送元: (value << 8) + 0x00 ～ (value << 8) + 0x9F
// 転送先: 0xFE00 ～ 0xFE9F (OAM)
// 転送時間: 160 Mサイクル (640 Tサイクル)
// 転送中はHRAM以外のメモリアクセスが制限される（簡易実装では即時コピー）

/// DMA転送コントローラ
pub struct Dma {
    /// DMA転送アクティブフラグ
    pub active: bool,
    /// 転送元ベースアドレス (上位バイト)
    pub source: u8,
    /// 転送バイトカウンタ
    pub byte_counter: u8,
    /// 残り転送サイクル
    pub remaining_cycles: u16,
}

impl Dma {
    pub fn new() -> Self {
        Self {
            active: false,
            source: 0,
            byte_counter: 0,
            remaining_cycles: 0,
        }
    }

    /// DMAレジスタへの書き込み（転送開始）
    pub fn start(&mut self, value: u8) {
        self.active = true;
        self.source = value;
        self.byte_counter = 0;
        self.remaining_cycles = 640; // 160 Mサイクル = 640 Tサイクル
    }

    /// DMAレジスタの読み取り
    pub fn read(&self) -> u8 {
        self.source
    }

    /// 転送元アドレスを計算
    pub fn source_address(&self) -> u16 {
        (self.source as u16) << 8
    }

    /// DMA転送が完了したかどうか
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// DMAを1サイクル進める。転送すべきバイトがあれば(src_addr, dst_addr)を返す
    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if !self.active {
            return None;
        }

        // 4Tサイクルごとに1バイト転送（デクリメント前にチェック）
        let transfer = if self.remaining_cycles % 4 == 0 && self.byte_counter < 160 {
            let src = self.source_address() + self.byte_counter as u16;
            let dst = 0xFE00 + self.byte_counter as u16;
            self.byte_counter += 1;
            Some((src, dst))
        } else {
            None
        };

        self.remaining_cycles = self.remaining_cycles.saturating_sub(1);

        if self.byte_counter >= 160 || self.remaining_cycles == 0 {
            self.active = false;
        }

        transfer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_creation() {
        let dma = Dma::new();
        assert!(!dma.is_active());
        assert_eq!(dma.source, 0);
    }

    #[test]
    fn test_dma_start() {
        let mut dma = Dma::new();
        dma.start(0xC0); // 転送元: 0xC000
        assert!(dma.is_active());
        assert_eq!(dma.source_address(), 0xC000);
        assert_eq!(dma.read(), 0xC0);
    }

    #[test]
    fn test_dma_transfer_addresses() {
        let mut dma = Dma::new();
        dma.start(0xC0);

        // 最初のティックで転送が発生
        let result = dma.tick();
        assert!(result.is_some());
        let (src, dst) = result.unwrap();
        assert_eq!(src, 0xC000);
        assert_eq!(dst, 0xFE00);
    }

    #[test]
    fn test_dma_completes() {
        let mut dma = Dma::new();
        dma.start(0xC0);

        let mut transfer_count = 0;
        for _ in 0..700 {
            if dma.tick().is_some() {
                transfer_count += 1;
            }
        }

        assert_eq!(transfer_count, 160);
        assert!(!dma.is_active());
    }

    #[test]
    fn test_dma_inactive_tick() {
        let mut dma = Dma::new();
        assert!(dma.tick().is_none());
    }
}
