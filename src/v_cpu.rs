use std::collections::HashMap;

struct DecodedInstruction {
    opcode: u8,
    rd: u8,
    funct3: u8,
    rs1: u8,
    rs2: u8,
    funct7: u8,
    imm: u32
}

pub struct VirtualCPU {
    pub regs: [u64; 32],
    pub pc: u64,
    pub memory: HashMap<u64, u8>,
}

const OPCODE_R: u8 = 0b0110011;
const OPCODE_I: u8 = 0b0010011;
const OPCODE_I_LOAD: u8 = 0b0000011;
const OPCODE_I_JUMP_LINK: u8 = 0b1100111;
const OPCODE_I_ENV: u8 = 0b1110011;
const OPCODE_S: u8 = 0b0100011;
const OPCODE_LUI: u8 = 0b0110111;
const OPCODE_AUIPC: u8 = 0b0010111;
const OPCODE_B: u8 = 0b1100011;
const OPCODE_JAL: u8 = 0b1101111;
const OPCODE_I_JALR: u8 = 0b1100111;

impl VirtualCPU 
{
    pub fn new() -> Self 
    {
        VirtualCPU 
        {
            regs: [0; 32],
            pc: 0,
            memory: HashMap::new(),
        }
    }

    pub fn fetch(&self) -> u32 
    {
        //TODO: CHANGE 
        let mut instruction: u32 = 0;
        for i in 0..4 
        {
            if let Some(&byte) = self.memory.get(&(self.pc + i)) 
            {
                instruction |= (byte as u32) << (i * 8);
            }
        }
        instruction        
    }

    pub fn decode(&self, instruction: u32) -> DecodedInstruction 
    {
        let opcode = (instruction & 0x7f) as u8;
        let rd = ((instruction >> 7) & 0x1f) as u8;
        let funct3 = ((instruction >> 12) & 0x07) as u8;
        let rs1 = ((instruction >> 15) & 0x1f) as u8;
        let rs2 = ((instruction >> 20) & 0x1f) as u8;
        let funct7 = ((instruction >> 25) & 0x7f) as u8;
        let imm = self.decode_immediate(opcode, instruction);

        DecodedInstruction 
        {
            opcode,
            rd,
            funct3,
            rs1,
            rs2,
            funct7,
            imm
        }
    }

    fn decode_immediate(&self, opcode: u8, instruction: u32) -> u32 
    {
        match opcode 
        {
            OPCODE_I | OPCODE_I_LOAD | OPCODE_I_JUMP_LINK | OPCODE_I_ENV | OPCODE_I_JALR => 
            {
                self.sign_extend(instruction >> 20, 12)
            }
            OPCODE_S => 
            {
                let imm = ((instruction >> 25) << 5) | ((instruction >> 7) & 0x1f);
                self.sign_extend(imm, 12)
            }
            OPCODE_B => 
            {
                let imm = (((instruction >> 31) & 0x1) << 12)
                    | (((instruction >> 7) & 0x1) << 11)
                    | (((instruction >> 25) & 0x3f) << 5)
                    | (((instruction >> 8) & 0xf) << 1);       
                self.sign_extend(imm & !0x1, 13)
            }
            OPCODE_LUI | OPCODE_AUIPC => 
            {
                instruction & 0xfffff000
            }
            OPCODE_JAL => 
            {
                let imm = (((instruction >> 31) & 0x1) << 20)
                    | (((instruction >> 21) & 0x3ff) << 1)
                    | (((instruction >> 20) & 0x1) << 11)
                    | (((instruction >> 12) & 0xff) << 12);
                imm
            }
            _ => 0, // Handle other opcodes if needed
        }
    }

    fn sign_extend(&self, imm: u32, bits: u32) -> u32 
    {
        let shift = 32 - bits;
        ((imm << shift) as i32 >> shift) as u32
    }

    pub fn execute(&mut self, instruction: DecodedInstruction) 
    {
        let rd = instruction.rd as usize;
        let rs1 = instruction.rs1 as usize;
        let rs2 = instruction.rs2 as usize;
        let imm = instruction.imm as u64;
        match instruction.opcode 
        {
            OPCODE_R => 
            {
                match instruction.funct3 
                {
                    0x0 => 
                    {
                        // add, sub
                        match instruction.funct7 
                        {
                            0x00 => 
                            {
                                self.regs[rd] = self.regs[rs1] + self.regs[rs2];
                            }
                            0x20 => 
                            {
                                self.regs[rd] = self.regs[rs1] - self.regs[rs2];
                            }
                            _ => {}
                        }
                    }
                    0x4 => 
                    {
                        // xor
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                    }
                    0x6 => 
                    {
                        // or
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                    }
                    0x7 => 
                    {
                        // and
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                    }
                    0x1 => 
                    {
                        // sll
                        self.regs[rd] = self.regs[rs1] << (self.regs[rs2] & 0x1f);
                    }
                    0x5 => 
                    {
                        if (instruction.funct7 & 0x20) == 0x20 
                        {
                            // sra
                            self.regs[rd] = ((self.regs[rs1] as i64) >> (self.regs[rs2] & 0x1f)) as u64;

                        } 
                        else 
                        {
                            // srl
                            self.regs[rd] = self.regs[rs1] >> (self.regs[rs2] & 0x1f);
                        }
                    }
                    0x2 => 
                    {
                        // slt
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
                    }
                    0x3 => 
                    {
                        // sltu
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
                    }
                    _ => {},
                }
            }

            OPCODE_I =>
            {
                println!("I opcode");
                match instruction.funct3 
                {
                    0x0 => 
                    {
                        // addi
                        self.regs[rd] = self.regs[rs1] + imm;
                    }
                    0x4 => 
                    {
                        // xori
                        self.regs[rd] = self.regs[rs1] ^ imm;
                    }
                    0x6 => 
                    {
                        // ori
                        self.regs[rd] = self.regs[rs1] | imm;
                    }
                    0x7 => 
                    {
                        // andi
                        self.regs[rd] = self.regs[rs1] & imm;
                    }
                    0x1 => 
                    {
                        // slli
                        self.regs[rd] = self.regs[rs1] << (imm & 0x1f);
                    }
                    0x5 => 
                    {
                        if (imm & 0x20) == 0x20 
                        {
                            // srai
                            self.regs[rd] = ((self.regs[rs1] as i64) >> (imm & 0x1f)) as u64;
                        } 
                        else 
                        {
                            // srli
                            self.regs[rd] = self.regs[rs1] >> (imm & 0x1f);
                        }
                    }
                    0x2 => 
                    {
                        // slti
                        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
                    }
                    0x3 => 
                    {
                        // sltiu
                        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
                    }
                    _ => {},
                }
            }

            OPCODE_I_LOAD =>
            {
                match instruction.funct3
                {
                    0x0 => 
                    {
                        // lb
                        let addr = self.regs[rs1] + imm;
                        let byte = *self.memory.get(&(addr as u64)).unwrap_or(&0);
                        self.regs[rd] = byte as i8 as i64 as u64; // Sign extension
                    }
                    0x1 => 
                    {
                        // lh
                        let addr = self.regs[rs1] + imm;
                        let b1 = *self.memory.get(&(addr as u64)).unwrap_or(&0);
                        let b2 = *self.memory.get(&(addr + 1 as u64)).unwrap_or(&0);
                        let half = (b1 as u16) << 8 | (b2 as u16);
                        println!("lw half word is {{{:x}}} {}", half, half);
                        self.regs[rd] = half as i16 as i64 as u64; // Sign extension
                    }
                    0x2 => 
                    {
                        // lw
                        let addr = self.regs[rs1] + imm;
                        let mut word: u32 = 0;
                        for x in 0..4
                        {
                            let byte = *self.memory.get(&(addr + x as u64)).unwrap_or(&0);
                            word |= (byte as u32) << (8 * x);
                        }
                        self.regs[rd] = word as i32 as i64 as u64; // Sign extension
                    }
                    0x4 => 
                    {
                        // lbu
                        let addr = self.regs[rs1] + imm;
                        let byte = *self.memory.get(&(addr as u64)).unwrap_or(&0);
                        self.regs[rd] = (byte & 0xff) as u64; // Zero extension
                    }
                    0x5 => 
                    {
                        // lhu
                        let addr = self.regs[rs1] + imm;
                        let b1 = *self.memory.get(&(addr as u64)).unwrap_or(&0);
                        let b2 = *self.memory.get(&(addr + 1 as u64)).unwrap_or(&0);
                        let half = (b1 as u16) << 8 | (b2 as u16);
                        self.regs[rd] = (half & 0xffff) as u64; // Zero extension
                    }
                    _ => {},
                }
            }
                    
            OPCODE_S => 
            {
                match instruction.funct3 
                {
                    0x0 => 
                    {
                        // sb
                        let addr = self.regs[rs1] + imm;
                        let value = self.regs[rs2] & 0xff;
                        self.memory.insert(addr as u64, value as u8);
                    }
                    0x1 => 
                    {
                        // sh
                        println!("rs1 {} rs2 {} imm {}", rs1, rs2, imm);
                        let addr = self.regs[rs1] + imm;
                        let value = self.regs[rs2] & 0xffff;
                        self.memory.insert(addr as u64, value as u8);
                        self.memory.insert(addr + 1 as u64, (value >> 8) as u8);
                    }
                    0x2 => 
                    {
                        // sw
                        println!("rs1 {} rs2 {} imm {}", rs1, rs2, imm);
                        let addr = self.regs[rs1] + imm;
                        let value = self.regs[rs2] & 0xffffffff;
                        self.memory.insert(addr as u64, value as u8);
                        self.memory.insert(addr + 1 as u64, (value >> 8) as u8);
                        self.memory.insert(addr + 2 as u64, (value >> 16) as u8);
                        self.memory.insert(addr + 3 as u64, (value >> 24) as u8);
                    }
                    _ => {},
                }
            }
            OPCODE_B => 
            {
                match instruction.funct3 
                {
                    0x0 => 
                    {
                        // beq
                        if self.regs[rs1] == self.regs[rs2] 
                        {
                            self.pc += imm as u64;
                        }
                    }
                    0x1 => 
                    {
                        // bne
                        if self.regs[rs1] != self.regs[rs2] 
                        {
                            self.pc += imm as u64;
                        }
                    }
                    0x4 => 
                    {
                        // blt
                        if self.regs[rs1] < self.regs[rs2] 
                        {
                            self.pc += imm as u64;
                        }
                    }
                    0x5 => 
                    {
                        // bge
                        if self.regs[rs1] >= self.regs[rs2] 
                        {
                            self.pc += imm as u64;
                        }
                    }
                    0x6 => 
                    {
                        // bltu
                        if (self.regs[rs1] as u64) < (self.regs[rs2] as u64) 
                        {
                            self.pc += imm as u64;
                        }
                    }
                    0x7 => 
                    {
                        // bgeu
                        if (self.regs[rs1] as u64) >= (self.regs[rs2] as u64) 
                        {
                            self.pc += imm as u64;
                        }
                    }
                    _ => {},
                }
            }
            OPCODE_JAL => 
            {
                // jal
                self.regs[rd] = self.pc + 4;
                self.pc += imm as u64;
            }
            OPCODE_JALR => 
            {
                // jalr
                self.regs[rd] = self.pc + 4;
                self.pc = (self.regs[rs1] + imm) & !1;
            }
            OPCODE_LUI => 
            {
                // lui
                self.regs[rd] = imm << 12;
            }
            OPCODE_AUIPC => 
            {
                // auipc
                self.regs[rd] = self.pc + (imm << 12);
            }
            _ => {},
        }
    }   
}


#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn test_decode_i_type() 
    {
        // Example: ADDI x1, x2, 5 -> 0000_0000_0101 00010 000 00001 0010011
        let instruction: u32 = 0b00000000010100010000000010010011;
        let cpu = VirtualCPU::new();
        let decoded = cpu.decode(instruction);
        assert_eq!(decoded.rd, 1);   // rd = 00001
        assert_eq!(decoded.rs1, 2);  // rs1 = 00010
        assert_eq!(decoded.imm, 5);  // imm = 000000000101
    }

    #[test]
    fn test_decode_s_type() 
    {
        // Example: SW x5, 8(x2) -> 000_0000 00101 00010 100 01000 0100011
        let instruction: u32 = 0b0_000000_00101_00010_100_01000_0100011;
        let cpu = VirtualCPU::new();
        let decoded = cpu.decode(instruction);
        assert_eq!(decoded.rs1, 2);  // rs1 = 00010
        assert_eq!(decoded.rs2, 5);  // rs2 = 00101
        assert_eq!(decoded.imm, 8);  // imm = 0000000001000
    }

    #[test]
    fn test_decode_b_type() 
    {
        // Example: BEQ x1, x2, 16 -> 0 100000 00010 00001 000 0001 0 1100011
        let instruction: u32 = 0b01000000001000001000000101100011;
        let cpu = VirtualCPU::new();
        let decoded = cpu.decode(instruction);
        assert_eq!(decoded.rs1, 1);  // rs1 = 00001
        assert_eq!(decoded.rs2, 2);  // rs2 = 00010
        assert_eq!(decoded.imm, 1026); // imm = 0_100000_0001_0 (with LSB=0)
    }

    #[test]
    fn test_decode_j_type() 
    {
        // Example: JAL 1452 -> 0 0000000010 1 00000000 00000 1101111
        let instruction: u32 = 0b00000000010100000000000001101111;
        let cpu = VirtualCPU::new();
        let decoded = cpu.decode(instruction);
        assert_eq!(decoded.imm, 2052); // imm = 100000000100 (with LSB=0)
    }

    #[test]
    fn test_load_store_instructions() 
    {
        let mut cpu = VirtualCPU::new();
        
        // Initialize memory
        cpu.memory.insert(0x1000, 0x12);   // byte
        cpu.memory.insert(0x1001, 0x34);   // byte
        cpu.memory.insert(0x1002, 0x55);   // halfword (16 bits)
        cpu.memory.insert(0x1003, 0x56);
        cpu.memory.insert(0x1004, 0x78);   // word (32 bits)

        // Set up registers
        cpu.regs[1] = 0x1000; // rs1 for load/store base address
        cpu.regs[2] = 0x00;   // rs2 for load/store offset

        // `lw` instruction
        let lw_instruction = DecodedInstruction {
            opcode: OPCODE_I_LOAD,
            funct3: 0x2, // lw
            funct7: 0x00,
            rd: 3,
            rs1: 1,
            rs2: 0,
            imm: 0x4, // Offset for the word
        };
        cpu.execute(lw_instruction);
        assert_eq!(cpu.regs[3], 0x78); // Load word at address 0x1004

        // `lh` instruction
        let lh_instruction = DecodedInstruction {
            opcode: OPCODE_I_LOAD,
            funct3: 0x1, // lh
            funct7: 0x00,
            rd: 4,
            rs1: 1,
            rs2: 0,
            imm: 0x2, // Offset for the halfword
        };
        cpu.execute(lh_instruction);
        assert_eq!(cpu.regs[4], 0x5556); // Load halfword at address 0x1002 (0x5654)

        // `lb` instruction
        let lb_instruction = DecodedInstruction {
            opcode: OPCODE_I_LOAD,
            funct3: 0x0, // lb
            funct7: 0x00,
            rd: 5,
            rs1: 1,
            rs2: 0,
            imm: 0x1, // Offset for the byte
        };
        cpu.execute(lb_instruction);
        assert_eq!(cpu.regs[5], 0x34); // Load byte at address 0x1001

        // `sw` instruction
        cpu.regs[6] = 0xABCD1234; // Value to store
        // store from regs[6] into memory address 0x1004
        let sw_instruction = DecodedInstruction {
            opcode: OPCODE_S,
            funct3: 0x2, // sw
            funct7: 0x00,
            rd: 0,
            rs1: 1,
            rs2: 6,
            imm: 0x4, // Offset for the word
        };
        cpu.execute(sw_instruction);
        assert_eq!(cpu.memory.get(&(0x1004 as u64)).unwrap(), &0x34); // Check stored value

        // `sh` instruction
        cpu.regs[7] = 0x1234; // Value to store
        let sh_instruction = DecodedInstruction {
            opcode: OPCODE_S,
            funct3: 0x1, // sh
            funct7: 0x00,
            rd: 0,
            rs1: 1,
            rs2: 7,
            imm: 0x2, // Offset for the halfword
        };
        cpu.execute(sh_instruction);
        assert_eq!(cpu.memory.get(&(0x1002 as u64)).unwrap(), &0x34); // Check stored halfword
    }

}
