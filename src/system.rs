use crate::register::Register;
use crate::{variant,Variant};

/// A single RISCV core
/// Includes a single program counter and 32 registers
/// Const generics will allow support of the E extensions for 16 registers
pub struct Core<R: Register> {
    registers: [R; 32],

    pub pc: R
}
impl<R: Register + Default + Copy + Clone> Core<R> {
    /// Creates a new core starting execution at the given address
    /// address must be aligned to 4 bytes else a panic will occur during execution
    pub fn new(address: R::Unsigned) -> Self {
        Self {
            registers: [Default::default(); 32],
            pc: R::from_unsigned(address)
        }
    }

    /// Increments the program counter by the instruction size of 4 bytes
    pub fn step(&mut self) {
        self.pc = self.pc.add_unsigned(R::zero_extended_byte(4))
    }

    /// Get the register `x{index}`
    /// # Safety
    /// A panic will occur if index is larger than 31
    #[inline(always)]
    pub fn get(&self, index: usize) -> R {
        self.registers[index]
    }

    /// Set register `x{index}` to be equal to `register`
    /// # Safety
    /// A panic will occur if index is larger than 31
    #[inline(always)]
    pub fn set(&mut self, index: usize, register: R) {
        if index > 0 {
            self.registers[index] = register
        }
    }

    /// Decode and execute an instruction
    pub fn execute(&mut self, instruction: [u8; 4], mmu: &mut dyn Mmu<R>) -> Result<(), DecodeError> {
        let opcode = instruction[0] & 0x7F;
        let funct3 = (instruction[1] & 0x70) >> 4;
        let funct7 = (instruction[3] & 0xFE) >> 1;

        match (opcode, funct3, funct7) {
            // ADD
            (0b0110011, 0b000, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).add_unsigned(self.get(source2)));
                Ok(self.step())
            },
            // SUB
            (0b0110011, 0b000, 0b0100000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).sub_unsigned(self.get(source2)));
                Ok(self.step())
            },
            // SLT
            (0b0110011, 0b010, _) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, if self.get(source1).lt_signed(self.get(source2)) { R::zero_extended_byte(1) } else { R::zero_extended_byte(0) });
                Ok(self.step())
            },
            // SLTU
            (0b0110011, 0b011, _) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, if self.get(source1).lt_unsigned(self.get(source2)) { R::zero_extended_byte(1) } else { R::zero_extended_byte(0) });
                Ok(self.step())
            },
            // ADDI
            (0b0010011, 0b000, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).add_signed(immediate));
                Ok(self.step())
            },
            // SLTI
            (0b0010011, 0b010, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, if self.get(source).lt_signed(immediate) { R::zero_extended_byte(1) } else { R::zero_extended_byte(0) });
                Ok(self.step())
            },
            // SLTIU
            (0b0010011, 0b011, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, if self.get(source).lt_unsigned(immediate) { R::zero_extended_byte(1) } else { R::zero_extended_byte(0) });
                Ok(self.step())
            },

            // XOR
            (0b0110011, 0b100, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).xor(self.get(source2)));
                Ok(self.step())
            },
            // OR
            (0b0110011, 0b110, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).or(self.get(source2)));
                Ok(self.step())
            },
            // AND
            (0b0110011, 0b111, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).and(self.get(source2)));
                Ok(self.step())
            },
            // XORI
            (0b0010011, 0b100, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).xor(immediate));
                Ok(self.step())
            },
            // ORI
            (0b0010011, 0b110, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).or(immediate));
                Ok(self.step())
            },
            // ANDI
            (0b0010011, 0b111, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).and(immediate));
                Ok(self.step())
            },

            // SLL
            (0b0110011, 0b001, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).shl(self.get(source2)));
                Ok(self.step())
            },
            // SRL
            (0b0110011, 0b101, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).shr(self.get(source2)));
                Ok(self.step())
            },
            // SRA
            (0b0110011, 0b101, 0b0100000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).sha(self.get(source2)));
                Ok(self.step())
            },
            // SLLI
            (0b0010011, 0b001, 0b0000000) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).shl(immediate));
                Ok(self.step())
            },
            // SRLI
            (0b0010011, 0b101, 0b0000000) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).shr(immediate));
                Ok(self.step())
            },
            // SRAI
            (0b0010011, 0b101, 0b0100000) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).sha(immediate));
                Ok(self.step())
            },

            // LUI
            (0b0110111, _, _) => {
                let variant::U { destination, immediate } = Variant::decode(instruction);
                self.set(destination, immediate);
                Ok(self.step())
            },
            // AUIPC
            (0b0010111, _, _) => {
                let variant::U { destination, immediate } = Variant::decode(instruction);
                self.set(destination, self.pc.add_signed(immediate));
                Ok(self.step())
            },

            // LB
            (0b0000011, 0b000, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_byte(mmu.get(self.get(source).add_signed(immediate).unsigned())));
                Ok(self.step())
            },
            // LBU
            (0b0000011, 0b100, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, R::zero_extended_byte(mmu.get(self.get(source).add_signed(immediate).unsigned())));
                Ok(self.step())
            },
            // LH
            (0b0000011, 0b001, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                let address = self.get(source).add_signed(immediate);
                self.set(destination, R::sign_extended_half([mmu.get(address.unsigned()), mmu.get(address.append(1))]));
                Ok(self.step())
            },
            // LHU
            (0b0000011, 0b101, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                let address = self.get(source).add_signed(immediate);
                self.set(destination, R::zero_extended_half([mmu.get(address.unsigned()), mmu.get(address.append(1))]));
                Ok(self.step())
            },
            // LW
            (0b0000011, 0b010, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                let address = self.get(source).add_signed(immediate);
                self.set(destination, R::sign_extended_word([
                    mmu.get(address.unsigned()),
                    mmu.get(address.append(1)),
                    mmu.get(address.append(2)),
                    mmu.get(address.append(3))
                ]));
                Ok(self.step())
            },

            // SB
            (0b0100011, 0b000, _) => {
                let variant::S { source1, source2, immediate } = Variant::decode(instruction);
                let address = self.get(source1).add_signed(immediate);
                mmu.set(address.unsigned(), self.get(source2).byte());
                Ok(self.step())
            },
            // SH
            (0b0100011, 0b001, _) => {
                let variant::S { source1, source2, immediate } = Variant::decode(instruction);
                let address = self.get(source1).add_signed(immediate);
                let half = self.get(source2).half();
                mmu.set(address.unsigned(), half[0]);
                mmu.set(address.append(1), half[1]);
                Ok(self.step())
            },
            // SW
            (0b0100011, 0b010, _) => {
                let variant::S { source1, source2, immediate } = Variant::decode(instruction);
                let address = self.get(source1).add_signed(immediate);
                let word = self.get(source2).word();
                mmu.set(address.unsigned(), word[0]);
                mmu.set(address.append(1), word[1]);
                mmu.set(address.append(2), word[2]);
                mmu.set(address.append(3), word[3]);
                Ok(self.step())
            },

            // JAL
            (0b1101111, _, _) => {
                let variant::J { destination, immediate } = Variant::decode(instruction);
                self.set(destination, self.pc.add_unsigned(R::zero_extended_byte(4)));
                Ok(self.pc = self.pc.add_signed(immediate))
            },
            // JALR
            (0b1100111, 0b000, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                // Despite source and immediate being unrelated to pc and destination
                // And Register being pure, the target address must be calculated first, else an off by 4 error occurs
                // Please let me know if you know why, it is likely to give me a good chuckle 
                let to_set = self.get(source).add_signed(immediate);
                self.set(destination, self.pc.add_unsigned(R::zero_extended_byte(4)));
                Ok(self.pc = to_set)
            },

            // BEQ
            (0b1100011, 0b000, _) => {
                let variant::B { source1, source2, immediate } = Variant::decode(instruction);
                Ok(if self.get(source1).eq(self.get(source2)) { self.pc = self.pc.add_signed(immediate) } else { self.step() })
            },
            // BNE
            (0b1100011, 0b001, _) => {
                let variant::B { source1, source2, immediate } = Variant::decode(instruction);
                Ok(if self.get(source1).neq(self.get(source2)) { self.pc = self.pc.add_signed(immediate) } else { self.step() })
            },
            // BLT
            (0b1100011, 0b100, _) => {
                let variant::B { source1, source2, immediate } = Variant::decode(instruction);
                Ok(if self.get(source1).lt_signed(self.get(source2)) { self.pc = self.pc.add_signed(immediate) } else { self.step() })
            },
            // BLTU
            (0b1100011, 0b110, _) => {
                let variant::B { source1, source2, immediate } = Variant::decode(instruction);
                Ok(if self.get(source1).lt_unsigned(self.get(source2)) { self.pc = self.pc.add_signed(immediate) } else { self.step() })
            },
            // BGE
            (0b1100011, 0b101, _) => {
                let variant::B { source1, source2, immediate } = Variant::decode(instruction);
                Ok(if self.get(source1).gte_signed(self.get(source2)) { self.pc = self.pc.add_signed(immediate) } else { self.step() })
            },
            // BGEU
            (0b1100011, 0b111, _) => {
                let variant::B { source1, source2, immediate } = Variant::decode(instruction);
                Ok(if self.get(source1).gte_unsigned(self.get(source2)) { self.pc = self.pc.add_signed(immediate) } else { self.step() })
            },
            (opcode, funct3, funct7) => Err(DecodeError::UnknownInstruction(opcode, funct3, funct7))
        }
    }
}

pub trait Mmu<R: Register> {
    /// Get the byte at the given address
    fn get(&self, address: R::Unsigned) -> u8;
    /// Set the byte at the given address
    fn set(&mut self, address: R::Unsigned, value: u8);
}

pub enum DecodeError {
    UnknownInstruction(u8, u8, u8)
}
impl std::fmt::Debug for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownInstruction(opcode, funct3, funct7) => write!(f, "UnknownInstruction(opcode: {:#b}, funct3: {:#b}, funct7: {:#b})", opcode, funct3, funct7)
        }
    }
}