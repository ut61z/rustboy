use crate::memory_map::dmg::{BOOTROM_SIZE, BOOTROM_START, BOOTROM_END};
pub struct BootRom {
    data: Box<[u8]>,
    active: bool,
}


impl BootRom {
    pub fn new(data: Box<[u8]>) -> Result<Self, String> {
        // BootRomは必ず256バイトでなければならない
        if data.len() != BOOTROM_SIZE {
            return Err(format!(
                "BootRom must be exactly 256 bytes, got {} bytes",
                data.len()
            ));
        }

        Ok(BootRom {
            data,
            active: true, // BootRomは初期状態でアクティブ
        })
    }

    pub fn new_dummy() -> Self {
        let mut data = vec![0x00; BOOTROM_SIZE];

        data[0xFC] = 0xC3; // JP命令のオペコード
        data[0xFD] = 0x00; // ジャンプ先の下位バイト
        data[0xFE] = 0x01; // ジャンプ先の上位バイト
        // これ以降はあとで

        Self {
            data: data.into_boxed_slice(),
            active: true, // 初期状態でアクティブ
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        if !self.active {
            return 0xFF; // BootRomが非アクティブな場合は常に0xFFを返す
        }

        if addr < BOOTROM_START || addr > BOOTROM_END {
            return 0xFF; // BootRomの範囲外は常に0xFFを返す
        }

        self.data[addr as usize]
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn write_disable_register(&mut self, value: u8) {
        if value != 0 {
            self.active = false; // 0以外の値が書き込まれたらBootRomを非アクティブにする
            println!("0以外の値 0x{:02X} がBootRomに書き込まれました。BootRomを非アクティブにします。", value);
        }
    }

    pub fn dump(&self) -> String {
        let mut result = String::new();
        result.push_str("BootRom Dump:\n");
        result.push_str(&format!("Active: {}\n", self.active));
        result.push_str("Address  : 00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F\n");

        for row in 0..16 {
            result.push_str(&format!("0x{:02X}0   : ", row));
            for col in 0..16 {
                let addr = row * 16 + col;
                result.push_str(&format!("{:02X} ", self.data[addr]));
            }
            result.push('\n');
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bootrom_creation() {
        let data = vec![0u8; 256];
        let bootrom = BootRom::new(data.into_boxed_slice()).unwrap();
        assert!(bootrom.is_active());
    }
    
    #[test]
    fn test_bootrom_invalid_size() {
        let data = vec![0u8; 100];  // 256バイトではない
        let result = BootRom::new(data.into_boxed_slice());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_bootrom_read() {
        let mut data = vec![0u8; 256];
        data[0x50] = 0x42;  // テストデータ
        
        let bootrom = BootRom::new(data.into_boxed_slice()).unwrap();
        assert_eq!(bootrom.read(0x50), 0x42);
    }
    
    #[test]
    fn test_bootrom_disable() {
        let data = vec![0u8; 256];
        let mut bootrom = BootRom::new(data.into_boxed_slice()).unwrap();
        
        assert!(bootrom.is_active());
        bootrom.write_disable_register(1);
        assert!(!bootrom.is_active());
        
        // 無効化後は0xFFを返す
        assert_eq!(bootrom.read(0x00), 0xFF);
    }
}
