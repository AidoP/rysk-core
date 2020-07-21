use crate::register::{Register,Register32,Register64,RegisterWidth};
use crate::variant::{self,Variant};
use crate::version;
#[cfg(feature = "ext-csr")]
use crate::csr::Csr;

/// A single RISCV core.
/// Includes a single program counter and 32 registers.
/// Const generics will allow support of the E extensions for 16 registers.
pub struct Core<R: Register> {
    /// The 32 general-purpose registers.
    /// Although all registers are general purpose in RISCV, their usage is still dictated by the standard calling convention.
    /// Register 0 always has a value of 0.
    registers: [R; 32],

    /// The program counter
    pub pc: R,

    /// CSR registers
    #[cfg(feature = "ext-csr")]
    csr: Csr<R>
}
impl<R: Register + Default + Copy + Clone> Core<R> {
    /// Creates a new core starting execution at the given address.
    /// address must be aligned to 4 bytes else a panic will occur during execution.
    #[cfg(not(feature = "ext-csr"))]
    pub fn new(address: R::Unsigned) -> Self {
        Self {
            registers: [Default::default(); 32],
            pc: R::from_unsigned(address)
        }
    }

    /// Creates a new core starting execution at the given address with the given hart ID.
    /// Hart ID's must be unique to ensure correct program behaviour. There must be a hart with ID 0 on a given system.
    /// `address` must be aligned to 4 bytes else a panic will occur during execution.
    #[cfg(feature = "ext-csr")]
    pub fn new(address: R::Unsigned, hart: R::Unsigned) -> Self {
        Self {
            registers: [Default::default(); 32],
            pc: R::from_unsigned(address),
            csr: Csr::new(hart, address)
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

    /// Get a value from a CSR. May have side-effects
    #[cfg(feature = "ext-csr")]
    pub fn get_csr(&self, index: usize) -> Result<R, Exception> {
        match index {
            // mstatus
            0x300 => unimplemented!(),
            // misa
            0x301 => {
                const I: u8 = 1 << 7;

                let isa0 = I;
                let isa1 = 0;
                let isa2 = 0;
                let isa3 = 0;

                const MXLEN32: u8 = 1;
                const MXLEN64: u8 = 2;
                const _MXLEN128: u8 = 3;
                Ok(
                    match R::WIDTH {
                        RegisterWidth::Bits32 => R::zero_extended_word([isa0, isa1, isa2, isa3 | MXLEN32 << 6]),
                        RegisterWidth::Bits64 => R::zero_extended_double([isa0, isa1, isa2, isa3, 0, 0, 0, MXLEN64 << 6]),
                    }
                )
            },
            // medeleg
            0x302 => Ok(self.csr.medeleg),
            // mideleg
            0x303 => Ok(self.csr.mideleg),
            // mie
            0x304 => Ok(self.csr.mie),
            // mtvec
            0x305 => Ok(self.csr.mtvec),
            // mcounteren
            0x306 => Ok(R::zero_extended_word(self.csr.mcounteren.word())),

            // mscratch
            0x340 => Ok(self.csr.mscratch),
            // mepc
            0x341 => Ok(self.csr.mepc),
            // mcause
            0x342 => Ok(self.csr.mcause),
            // mtval
            0x343 => Ok(self.csr.mtval),
            // mip
            0x344 => Ok(self.csr.mip),

            // mcycle and mcycleh
            0xB00 if R::WIDTH != RegisterWidth::Bits32 => Ok(R::zero_extended_double(self.csr.mcycle.double())),
            0xB00 if R::WIDTH == RegisterWidth::Bits32 => Ok(R::zero_extended_word((self.csr.mcycle.split().0).0)),
            0xB80 if R::WIDTH == RegisterWidth::Bits32 => Ok(R::zero_extended_word((self.csr.mcycle.split().1).0)),
            // minstret - Currently the same as mcycle
            0xB02 if R::WIDTH != RegisterWidth::Bits32 => Ok(R::zero_extended_double(self.csr.mcycle.double())),
            0xB02 if R::WIDTH == RegisterWidth::Bits32 => Ok(R::zero_extended_word((self.csr.mcycle.split().0).0)),
            0xB82 if R::WIDTH == RegisterWidth::Bits32 => Ok(R::zero_extended_word((self.csr.mcycle.split().1).0)),
            // Unused performance counters
            0xB03..=0xB1F => Ok(R::default()),
            0xB83..=0xB9F if R::WIDTH == RegisterWidth::Bits32 => Ok(R::default()),
            // Unused performance event selectors
            0xB23..=0xB3F => Ok(R::default()),

            // mvendorid
            // Requires a JEDEC vendor ID
            0xF11 => Ok(R::default()),
            // marchid
            // In the future an Architecture ID shall be requested
            0xF12 => Ok(R::default()),
            // mimpid
            // The version of rysk-core
            0xF13 => Ok(R::zero_extended_word([version::PATCH, version::MINOR, version::MAJOR, 0])),
            // mhartid
            0xF14 => Ok(self.csr.mhartid),
            _ => Err(Exception::IllegalInstruction)
        }
    }

    /// Set a CSR to the specified value with program-defined access. May have side-effects
    #[cfg(feature = "ext-csr")]
    pub fn set_csr(&mut self, index: usize, value: R) {
        match index {
            // mie
            0x304 => {
                // WPRI fields must be hardwired to zero
                self.csr.mie = value.and(R::zero_extended_half([!0x44, !0xF4]))
            },
            // mip
            0x344 => {
                // WPRI fields must be hardwired to zero
                self.csr.mip = value.and(R::zero_extended_half([!0x44, !0xF4]))
            },
            _ => ()
        }
    }

    /// Decode and execute an instruction
    #[allow(clippy::cognitive_complexity)]
    pub fn execute(&mut self, mmu: &mut dyn Mmu<R>) -> Result<(), Exception> {
        let instruction = mmu.fetch(self.pc);
        let opcode = instruction[0] & 0x7F;
        let funct3 = (instruction[1] & 0x70) >> 4;
        let funct7 = (instruction[3] & 0xFE) >> 1;

        // Increment the cycle counter
        #[cfg(feature = "ext-csr")]
        {self.csr.mcycle = self.csr.mcycle.add_unsigned(Register64::zero_extended_byte(1))}

        #[allow(clippy::unreadable_literal)]
        match (opcode, funct3, funct7) {
            // ADD
            (0b0110011, 0b000, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).add_unsigned(self.get(source2)));
                Ok(self.step())
            },
            // ADDW
            (0b0111011, 0b000, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_word(Register32(self.get(source1).word()).add_unsigned(Register32(self.get(source2).word())).word()));
                Ok(self.step())
            },
            // SUB
            (0b0110011, 0b000, 0b0100000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).sub_unsigned(self.get(source2)));
                Ok(self.step())
            },
            // SUBW
            (0b0111011, 0b000, 0b0100000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_word(Register32(self.get(source1).word()).sub_unsigned(Register32(self.get(source2).word())).word()));
                Ok(self.step())
            },
            // SLT
            (0b0110011, 0b010, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, if self.get(source1).lt_signed(self.get(source2)) { R::zero_extended_byte(1) } else { R::zero_extended_byte(0) });
                Ok(self.step())
            },
            // SLTU
            (0b0110011, 0b011, 0b0000000) => {
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
            // ADDIW
            (0b0011011, 0b000, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_word(Register32(self.get(source).word()).add_signed(immediate).word()));
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
            // SLLW
            (0b0111011, 0b001, 0b0000000) if R::WIDTH != RegisterWidth::Bits32 => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_word(Register32(self.get(source1).word()).shl(Register32(self.get(source2).word())).word()));
                Ok(self.step())
            },
            // SRL
            (0b0110011, 0b101, 0b0000000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).shr(self.get(source2)));
                Ok(self.step())
            },
            // SRLW
            (0b0111011, 0b101, 0b0000000) if R::WIDTH != RegisterWidth::Bits32 => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_word(Register32(self.get(source1).word()).shr(Register32(self.get(source2).word())).word()));
                Ok(self.step())
            },
            // SRA
            (0b0110011, 0b101, 0b0100000) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, self.get(source1).sha(self.get(source2)));
                Ok(self.step())
            },
            // SRAW
            (0b0111011, 0b101, 0b0100000) if R::WIDTH != RegisterWidth::Bits32 => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                self.set(destination, R::sign_extended_word(Register32(self.get(source1).word()).sha(Register32(self.get(source2).word())).word()));
                Ok(self.step())
            },
            // SLLI
            (0b0010011, 0b001, _) => {
                let variant::I::<R> { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).shl(immediate.and(R::zero_extended_byte(0x0E))));
                Ok(self.step())
            },
            // SLLIW
            (0b0011011, 0b001, _) if R::WIDTH != RegisterWidth::Bits32 => {
                let variant::I::<R> { destination, source, immediate } = Variant::decode(instruction);
                if immediate.byte() & 0x20 != 0 {
                    Err(Exception::ShiftWordReservedBit)
                } else {
                    self.set(destination, R::sign_extended_word(Register32(self.get(source).word()).shl(Register32(immediate.word()).and(Register32::zero_extended_byte(0x0E))).word()));
                    Ok(self.step())
                }
            },
            // SRLI
            (0b0010011, 0b101, _) if instruction[3] & 0x40 == 0 => {
                let variant::I::<R> { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).shr(immediate.and(R::zero_extended_byte(0x0E))));
                Ok(self.step())
            },
            // SRLIW
            (0b0011011, 0b101, _) if instruction[3] & 0x40 == 0 && R::WIDTH != RegisterWidth::Bits32 => {
                let variant::I::<R> { destination, source, immediate } = Variant::decode(instruction);
                if immediate.byte() & 0x20 != 0 {
                    Err(Exception::ShiftWordReservedBit)
                } else {
                    self.set(destination, R::sign_extended_word(Register32(self.get(source).word()).shr(Register32(immediate.word()).and(Register32::zero_extended_byte(0x0E))).word()));
                    Ok(self.step())
                }
            },
            // SRAI
            (0b0010011, 0b101, _) if instruction[3] & 0x40 != 0 => {
                let variant::I::<R> { destination, source, immediate } = Variant::decode(instruction);
                self.set(destination, self.get(source).sha(immediate.and(R::zero_extended_byte(0x0E))));
                Ok(self.step())
            },
            // SRAIW
            (0b0011011, 0b101, _) if instruction[3] & 0x40 != 0 && R::WIDTH != RegisterWidth::Bits32 => {
                let variant::I::<R> { destination, source, immediate } = Variant::decode(instruction);
                if immediate.byte() & 0x20 != 0 {
                    Err(Exception::ShiftWordReservedBit)
                } else {
                    self.set(destination, R::sign_extended_word(Register32(self.get(source).word()).sha(Register32(immediate.word()).and(Register32::zero_extended_byte(0x0E))).word()));
                    Ok(self.step())
                }
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
            // LWU
            (0b0000011, 0b110, _) if R::WIDTH != RegisterWidth::Bits32 => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                let address = self.get(source).add_signed(immediate);
                self.set(destination, R::zero_extended_word([
                    mmu.get(address.unsigned()),
                    mmu.get(address.append(1)),
                    mmu.get(address.append(2)),
                    mmu.get(address.append(3))
                ]));
                Ok(self.step())
            },
            // LD
            (0b0000011, 0b011, _) => {
                let variant::I { destination, source, immediate } = Variant::decode(instruction);
                let address = self.get(source).add_signed(immediate);
                self.set(destination, R::sign_extended_double([
                    mmu.get(address.unsigned()),
                    mmu.get(address.append(1)),
                    mmu.get(address.append(2)),
                    mmu.get(address.append(3)),
                    mmu.get(address.append(4)),
                    mmu.get(address.append(5)),
                    mmu.get(address.append(6)),
                    mmu.get(address.append(7))
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

            // M Extension
            // MUL
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b000, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).mul(self.get(source2))))
            },
            // MULH
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b001, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).mulh(self.get(source2))))
            },
            // MULHSU
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b010, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).mulhsu(self.get(source2))))
            },
            // MULHU
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b011, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).mulhu(self.get(source2))))
            },
            // DIV
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b100, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).div(self.get(source2))))
            },
            // DIVU
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b101, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).divu(self.get(source2))))
            },
            // DIVU
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b110, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).rem(self.get(source2))))
            },
            // REMU
            #[cfg(feature = "ext-m")]
            (0b0110011, 0b111, 0b0000001) => {
                let variant::R { destination, source1, source2 } = Variant::decode(instruction);
                Ok(self.set(destination, self.get(source1).remu(self.get(source2))))
            },

            // Zicsr Extension
            // CSRRW
            #[cfg(feature = "ext-csr")]
            (0b1110011, 0b001, _) => {
                let variant::C { destination, source, csr } = Variant::decode(instruction);
                Ok(if destination != 0 {
                    let temporary = self.get_csr(csr).expect("TODO: Exception signaling");
                    self.set_csr(csr, self.get(source));
                    self.set(destination, temporary)
                } else {
                    self.set_csr(csr, self.get(source))
                })
            },
            // CSRRS
            #[cfg(feature = "ext-csr")]
            (0b1110011, 0b010, _) => {
                let variant::C { destination, source, csr } = Variant::decode(instruction);
                let temporary = self.get_csr(csr).expect("TODO: Exception signaling");
                Ok(if source != 0 {
                    // Source is a bitmask which sets bits in the csr
                    self.set_csr(csr, temporary.or(self.get(source)));
                    self.set(destination, temporary)
                } else {
                    self.set(destination, temporary)
                })
            },
            // CSRRC
            #[cfg(feature = "ext-csr")]
            (0b1110011, 0b011, _) => {
                let variant::C { destination, source, csr } = Variant::decode(instruction);
                let temporary = self.get_csr(csr).expect("TODO: Exception signaling");
                Ok(if source != 0 {
                    // Source is a bitmask which clears bits in the csr
                    self.set_csr(csr, temporary.and(self.get(source).not()));
                    self.set(destination, temporary)
                } else {
                    self.set(destination, temporary)
                })
            },
            // CSRRWI
            #[cfg(feature = "ext-csr")]
            (0b1110011, 0b101, _) => {
                let variant::C { destination, source, csr } = Variant::decode(instruction);
                let immediate = R::zero_extended_byte(source as u8);
                Ok(if destination != 0 {
                    let temporary = self.get_csr(csr).expect("TODO: Exception signaling");
                    self.set_csr(csr, immediate);
                    self.set(destination, temporary)
                } else {
                    self.set_csr(csr, immediate)
                })
            },
            // CSRRSI
            #[cfg(feature = "ext-csr")]
            (0b1110011, 0b110, _) => {
                let variant::C { destination, source, csr } = Variant::decode(instruction);
                let temporary = self.get_csr(csr).expect("TODO: Exception signaling");
                Ok(if source != 0 {
                    // Source is a bitmask which sets bits in the csr
                    self.set_csr(csr, temporary.or(R::zero_extended_byte(source as u8)));
                    self.set(destination, temporary)
                } else {
                    self.set(destination, temporary)
                })
            },
            // CSRRCI
            #[cfg(feature = "ext-csr")]
            (0b1110011, 0b111, _) => {
                let variant::C { destination, source, csr } = Variant::decode(instruction);
                let temporary = self.get_csr(csr).expect("TODO: Exception signaling");
                Ok(if source != 0 {
                    // Source is a bitmask which clears bits in the csr
                    self.set_csr(csr, temporary.and(R::zero_extended_byte(source as u8).not()));
                    self.set(destination, temporary)
                } else {
                    self.set(destination, temporary)
                })
            },
            (opcode, funct3, funct7) => Err(Exception::UnknownInstruction(opcode, funct3, funct7))
        }
    }
}

/// A Memory Management Unit (MMU) handles memory accesses on the system.
/// Devices and memory regions other than working memory (ie. RAM) may be mapped by way of the MMU.
pub trait Mmu<R: Register> {
    /// Get the byte at the given address
    fn get(&self, address: R::Unsigned) -> u8;
    /// Set the byte at the given address
    fn set(&mut self, address: R::Unsigned, value: u8);
    /// Fetch an instruction to execute
    fn fetch(&self, address: R) -> [u8; 4] {
        [
            self.get(address.unsigned()),
            self.get(address.append(1)),
            self.get(address.append(2)),
            self.get(address.append(3))
        ]
    }
}

/// An exception thrown by a hart during instruction decoding and execution
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Exception {
    IllegalInstruction,
    UnknownInstruction(u8, u8, u8),
    ShiftWordReservedBit
}
impl std::fmt::Debug for Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IllegalInstruction => write!(f, "Illegal value in instruction"),
            Self::UnknownInstruction(opcode, funct3, funct7) => write!(f, "UnknownInstruction(opcode: {:#b}, funct3: {:#b}, funct7: {:#b})", opcode, funct3, funct7),
            Self::ShiftWordReservedBit => write!(f, "Shift *W instruction was used with immediate[5] set. This bit is reserved.")
        }
    }
}