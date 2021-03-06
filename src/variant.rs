use crate::register::Register;

/// Decode an instruction encoding variant into its significant parts
/// ```rust
/// use rysk_core::variant::{ self, Variant };
/// let instruction = [0x13, 0, 0, 0];
/// let variant::R { destination, source1, source2 } = Variant::decode(instruction);
/// ```
pub trait Variant {
    fn decode(instruction: [u8; 4]) -> Self;
}

/// Extract the destination register index from an instruction
macro_rules! destination {
    ($instruction:expr) => {
        ((($instruction[0] & 0x80) >> 7) | (($instruction[1] & 0x0F) << 1)) as _
    };
}
/// Extract the first source register index from an instruction
macro_rules! source1 {
    ($instruction:expr) => {
        ((($instruction[1] & 0x80) >> 7) | (($instruction[2] & 0x0F) << 1)) as _
    };
}
/// Extract the second source register index from an instruction
macro_rules! source2 {
    ($instruction:expr) => {
        ((($instruction[2] & 0xF0) >> 4) | (($instruction[3] & 0x01) << 4)) as _
    };
}

/// The R instruction type, encoding a destination and 2 source registers.
#[derive(Debug, Eq, PartialEq)]
pub struct R {
    pub destination: usize,
    pub source1: usize,
    pub source2: usize
}
impl Variant for R {
    /// Decode the instruction to an R variant as specified in the ISA
    /// ```rust
    /// use rysk_core::variant::*;
    /// assert_eq!(R { destination: 0, source1: 0, source2: 0 }, Variant::decode([0, 0, 0, 0]));
    /// ```
    fn decode(instruction: [u8; 4]) -> Self {
        Self {
            destination: destination!(instruction),
            source1: source1!(instruction),
            source2: source2!(instruction),
        }
    }
}

/// The I instruction type, encoding a destination and source register as well as an immediate value.
/// The immediate value is a sign extended 12-bit integer.
#[derive(Debug, Eq, PartialEq)]
pub struct I<R: Register> {
    pub destination: usize,
    pub source: usize,
    pub immediate: R
}
impl<R: Register> Variant for I<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            destination: destination!(instruction),
            source: source1!(instruction),
            immediate: R::sign_extended_half([((instruction[2] & 0xF0) >> 4) | ((instruction[3] & 0x0F) << 4), ((instruction[3] & 0xF0) >> 4) | if signed { 0xF0 } else { 0 }])
        }
    }
}

/// A variation of the I type where the immediate encodes a 12-bit unsigned integer index.
#[derive(Debug, Eq, PartialEq)]
pub struct C {
    pub destination: usize,
    pub source: usize,
    pub csr: usize
}
impl Variant for C {
    fn decode(instruction: [u8; 4]) -> Self {
        Self {
            destination: destination!(instruction),
            source: source1!(instruction),
            csr: ((instruction[2] & 0xF0) >> 4) as usize | ((instruction[3] & 0xFF) as usize) << 4
        }
    }
}

/// The S instruction type, encoding 2 source registers and a 12-bit sign extended immediate value.
#[derive(Debug, Eq, PartialEq)]
pub struct S<R: Register> {
    pub source1: usize,
    pub source2: usize,
    pub immediate: R
}
impl<R: Register> Variant for S<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            source1: source1!(instruction),
            source2: source2!(instruction),
            immediate: R::sign_extended_half([((instruction[0] & 0x80) >> 7) | ((instruction[1] & 0x0F) << 1) | ((instruction[3] & 0x0E) << 4), ((instruction[3] & 0xF0) >> 4) | if signed { 0xF0 } else { 0 }])
        }
    }
}

/// A variation of the S type where the immediate is a 13-bit branch offset.
/// The branch offset's least significant bit is not set as it must always be aligned, thereby allowing for larger offsets.
#[derive(Debug, Eq, PartialEq)]
pub struct B<R: Register> {
    pub source1: usize,
    pub source2: usize,
    pub immediate: R
}
impl<R: Register> Variant for B<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            source1: source1!(instruction),
            source2: source2!(instruction),
            immediate: R::sign_extended_half([
                ((instruction[1] & 0xF) << 1) | ((instruction[3] & 0x0E) << 4),
                ((instruction[3] & 0x70) >> 4) | ((instruction[0] & 0x80) >> 4) | ((instruction[3] & 0x80) >> 3) | if signed { 0xE0 } else { 0 },
            ])
        }
    }
}

/// The U instruction variant, encoding a destination and a 32-bit immediate value with the lower 12 bits zeroed.
#[derive(Debug, Eq, PartialEq)]
pub struct U<R: Register> {
    pub destination: usize,
    pub immediate: R
}
impl<R: Register> Variant for U<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        Self {
            destination: destination!(instruction),
            immediate: R::sign_extended_word([0, instruction[1] & 0xF0, instruction[2], instruction[3]])
        }
    }
}

/// A variation of the U instruction type where the immediate encodes a 21-bit jump offset.
/// The least significant bit of the offset is zeroed as it must be aligned, thereby allowing a greater offset range.
#[derive(Debug, Eq, PartialEq)]
pub struct J<R: Register> {
    pub destination: usize,
    pub immediate: R
}
impl<R: Register> Variant for J<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            destination: destination!(instruction),
            immediate: R::sign_extended_word([
                ((instruction[2] & 0xE0) >> 4) // 1-3
                    | ((instruction[3] & 0x0F) << 4), // 4-7
                ((instruction[3] & 0x70) >> 4) // 8-10
                    | ((instruction[2] & 0x10) >> 1) // 11
                    | (instruction[1] & 0xF0), // 12-15
                (instruction[2] & 0x0F) // 16-19
                    | ((instruction[3] & 0x80) >> 3) // 20
                    | if signed {0xE0} else {0},
                    if signed {0xFF} else {0}
            ])
        }
    }
}