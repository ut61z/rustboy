// src/cpu/instructions.rs
// GameBoy CPU 命令定義

/// 命令の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionType {
    /// 何もしない
    Nop,
    /// 8bitレジスタに即値をロード
    LdR8N,
    /// 16bitレジスタに即値をロード
    LdR16N,
    /// 絶対ジャンプ
    JpNN,
    /// 相対ジャンプ
    JrN,
    /// 不明な命令
    Unknown,
}

/// 8bitレジスタの識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register8 {
    A, B, C, D, E, H, L,
}

/// 16bitレジスタの識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register16 {
    AF, BC, DE, HL, SP, PC,
}

/// 命令の情報
#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    /// 命令の種類
    pub instruction_type: InstructionType,
    /// オペコード
    pub opcode: u8,
    /// 命令長（バイト数）
    pub length: u8,
    /// 実行サイクル数
    pub cycles: u8,
    /// 対象レジスタ（8bit）
    pub reg8: Option<Register8>,
    /// 対象レジスタ（16bit）
    pub reg16: Option<Register16>,
    /// 命令の説明
    pub description: &'static str,
}

impl Instruction {
    /// 新しい命令を作成
    pub fn new(
        instruction_type: InstructionType,
        opcode: u8,
        length: u8,
        cycles: u8,
        description: &'static str,
    ) -> Self {
        Self {
            instruction_type,
            opcode,
            length,
            cycles,
            reg8: None,
            reg16: None,
            description,
        }
    }
    
    /// 8bitレジスタを指定した命令を作成
    pub fn with_reg8(mut self, reg: Register8) -> Self {
        self.reg8 = Some(reg);
        self
    }
    
    /// 16bitレジスタを指定した命令を作成
    pub fn with_reg16(mut self, reg: Register16) -> Self {
        self.reg16 = Some(reg);
        self
    }
}

/// 命令テーブル
pub struct InstructionTable {
    instructions: [Option<Instruction>; 256],
}

impl InstructionTable {
    /// 新しい命令テーブルを作成
    pub fn new() -> Self {
        let mut table = Self {
            instructions: [None; 256],
        };
        
        table.initialize_instructions();
        table
    }
    
    /// 命令テーブルを初期化
    fn initialize_instructions(&mut self) {
        // NOP
        self.add_instruction(
            0x00,
            Instruction::new(InstructionType::Nop, 0x00, 1, 4, "NOP")
        );
        
        // LD r8, n 命令群
        self.add_instruction(
            0x3E,
            Instruction::new(InstructionType::LdR8N, 0x3E, 2, 8, "LD A, n")
                .with_reg8(Register8::A)
        );
        self.add_instruction(
            0x06,
            Instruction::new(InstructionType::LdR8N, 0x06, 2, 8, "LD B, n")
                .with_reg8(Register8::B)
        );
        self.add_instruction(
            0x0E,
            Instruction::new(InstructionType::LdR8N, 0x0E, 2, 8, "LD C, n")
                .with_reg8(Register8::C)
        );
        self.add_instruction(
            0x16,
            Instruction::new(InstructionType::LdR8N, 0x16, 2, 8, "LD D, n")
                .with_reg8(Register8::D)
        );
        self.add_instruction(
            0x1E,
            Instruction::new(InstructionType::LdR8N, 0x1E, 2, 8, "LD E, n")
                .with_reg8(Register8::E)
        );
        self.add_instruction(
            0x26,
            Instruction::new(InstructionType::LdR8N, 0x26, 2, 8, "LD H, n")
                .with_reg8(Register8::H)
        );
        self.add_instruction(
            0x2E,
            Instruction::new(InstructionType::LdR8N, 0x2E, 2, 8, "LD L, n")
                .with_reg8(Register8::L)
        );
        
        // LD r16, nn 命令群
        self.add_instruction(
            0x31,
            Instruction::new(InstructionType::LdR16N, 0x31, 3, 12, "LD SP, nn")
                .with_reg16(Register16::SP)
        );
        
        // ジャンプ命令
        self.add_instruction(
            0xC3,
            Instruction::new(InstructionType::JpNN, 0xC3, 3, 16, "JP nn")
        );
        self.add_instruction(
            0x18,
            Instruction::new(InstructionType::JrN, 0x18, 2, 12, "JR n")
        );
    }
    
    /// 命令を追加
    fn add_instruction(&mut self, opcode: u8, instruction: Instruction) {
        self.instructions[opcode as usize] = Some(instruction);
    }
    
    /// オペコードから命令を取得
    pub fn get_instruction(&self, opcode: u8) -> Option<&Instruction> {
        self.instructions[opcode as usize].as_ref()
    }
    
    /// 実装済み命令の一覧を取得
    pub fn get_implemented_opcodes(&self) -> Vec<u8> {
        let mut opcodes = Vec::new();
        for (i, instruction) in self.instructions.iter().enumerate() {
            if instruction.is_some() {
                opcodes.push(i as u8);
            }
        }
        opcodes
    }
}

impl Default for InstructionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_instruction_table_creation() {
        let table = InstructionTable::new();
        
        // NOP命令の確認
        let nop = table.get_instruction(0x00).unwrap();
        assert_eq!(nop.instruction_type, InstructionType::Nop);
        assert_eq!(nop.cycles, 4);
        assert_eq!(nop.length, 1);
        
        // LD A, n命令の確認
        let ld_a_n = table.get_instruction(0x3E).unwrap();
        assert_eq!(ld_a_n.instruction_type, InstructionType::LdR8N);
        assert_eq!(ld_a_n.reg8, Some(Register8::A));
        assert_eq!(ld_a_n.cycles, 8);
        assert_eq!(ld_a_n.length, 2);
        
        // JP nn命令の確認
        let jp_nn = table.get_instruction(0xC3).unwrap();
        assert_eq!(jp_nn.instruction_type, InstructionType::JpNN);
        assert_eq!(jp_nn.cycles, 16);
        assert_eq!(jp_nn.length, 3);
    }
    
    #[test]
    fn test_unknown_instruction() {
        let table = InstructionTable::new();
        
        // 未実装の命令
        assert!(table.get_instruction(0xFF).is_none());
    }
    
    #[test]
    fn test_implemented_opcodes() {
        let table = InstructionTable::new();
        let opcodes = table.get_implemented_opcodes();
        
        // 実装済み命令が含まれていることを確認
        assert!(opcodes.contains(&0x00)); // NOP
        assert!(opcodes.contains(&0x3E)); // LD A, n
        assert!(opcodes.contains(&0xC3)); // JP nn
        
        // 最低限の命令数が実装されていることを確認
        assert!(opcodes.len() >= 10);
    }
}
