// src/cpu/decoder.rs
// GameBoy CPU 命令デコーダ

use super::instructions::{InstructionTable, Instruction, InstructionType};

/// 命令デコーダ
pub struct InstructionDecoder {
    /// 命令テーブル
    instruction_table: InstructionTable,
}

impl InstructionDecoder {
    /// 新しいデコーダを作成
    pub fn new() -> Self {
        Self {
            instruction_table: InstructionTable::new(),
        }
    }
    
    /// オペコードを命令にデコード
    pub fn decode(&self, opcode: u8) -> Result<&Instruction, String> {
        match self.instruction_table.get_instruction(opcode) {
            Some(instruction) => Ok(instruction),
            None => Err(format!("未実装の命令: 0x{:02X}", opcode)),
        }
    }
    
    /// CB prefixed 命令をデコード（将来の拡張用）
    pub fn decode_cb(&self, opcode: u8) -> Result<&Instruction, String> {
        // TODO: CB命令の実装
        Err(format!("CB命令は未実装: 0xCB{:02X}", opcode))
    }
    
    /// 命令の詳細情報を取得
    pub fn get_instruction_info(&self, opcode: u8) -> String {
        match self.decode(opcode) {
            Ok(instruction) => format!(
                "0x{:02X}: {} (length:{}, cycles:{})",
                opcode,
                instruction.description,
                instruction.length,
                instruction.cycles
            ),
            Err(e) => e,
        }
    }
    
    /// 実装済み命令の一覧を表示
    pub fn list_implemented_instructions(&self) -> String {
        let opcodes = self.instruction_table.get_implemented_opcodes();
        let mut result = String::new();
        result.push_str("実装済み命令一覧:\n");
        
        for opcode in opcodes {
            if let Ok(instruction) = self.decode(opcode) {
                result.push_str(&format!(
                    "  0x{:02X}: {:<12} ({}bytes, {}cycles)\n",
                    opcode,
                    instruction.description,
                    instruction.length,
                    instruction.cycles
                ));
            }
        }
        
        result
    }
    
    /// 命令タイプ別の統計を取得
    pub fn get_instruction_stats(&self) -> String {
        let opcodes = self.instruction_table.get_implemented_opcodes();
        let mut nop_count = 0;
        let mut load_count = 0;
        let mut jump_count = 0;
        let mut unknown_count = 0;
        
        for opcode in opcodes {
            if let Ok(instruction) = self.decode(opcode) {
                match instruction.instruction_type {
                    InstructionType::Nop => nop_count += 1,
                    InstructionType::LdR8N | InstructionType::LdR16N => load_count += 1,
                    InstructionType::JpNN | InstructionType::JrN => jump_count += 1,
                    InstructionType::Unknown => unknown_count += 1,
                }
            }
        }
        
        format!(
            "命令統計:\n  NOP: {}\n  LOAD: {}\n  JUMP: {}\n  UNKNOWN: {}\n  合計: {}",
            nop_count,
            load_count,
            jump_count,
            unknown_count,
            nop_count + load_count + jump_count + unknown_count
        )
    }
}

impl Default for InstructionDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decoder_creation() {
        let decoder = InstructionDecoder::new();
        
        // 基本的な命令がデコードできることを確認
        assert!(decoder.decode(0x00).is_ok()); // NOP
        assert!(decoder.decode(0x3E).is_ok()); // LD A, n
        assert!(decoder.decode(0xC3).is_ok()); // JP nn
    }
    
    #[test]
    fn test_decode_valid_instruction() {
        let decoder = InstructionDecoder::new();
        
        let instruction = decoder.decode(0x00).unwrap();
        assert_eq!(instruction.opcode, 0x00);
        assert_eq!(instruction.instruction_type, InstructionType::Nop);
    }
    
    #[test]
    fn test_decode_invalid_instruction() {
        let decoder = InstructionDecoder::new();
        
        let result = decoder.decode(0xFF);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("未実装"));
    }
    
    #[test]
    fn test_instruction_info() {
        let decoder = InstructionDecoder::new();
        
        let info = decoder.get_instruction_info(0x00);
        assert!(info.contains("NOP"));
        assert!(info.contains("length:1"));
        assert!(info.contains("cycles:4"));
    }
    
    #[test]
    fn test_cb_instruction() {
        let decoder = InstructionDecoder::new();
        
        let result = decoder.decode_cb(0x00);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CB命令は未実装"));
    }
    
    #[test]
    fn test_list_instructions() {
        let decoder = InstructionDecoder::new();
        
        let list = decoder.list_implemented_instructions();
        assert!(list.contains("実装済み命令一覧"));
        assert!(list.contains("NOP"));
        assert!(list.contains("LD A, n"));
    }
    
    #[test]
    fn test_instruction_stats() {
        let decoder = InstructionDecoder::new();
        
        let stats = decoder.get_instruction_stats();
        assert!(stats.contains("命令統計"));
        assert!(stats.contains("NOP: 1"));
        assert!(stats.contains("LOAD:"));
        assert!(stats.contains("JUMP:"));
    }
}