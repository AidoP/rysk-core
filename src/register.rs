/// An integer type which can apply operations as specified by the ISA
/// No panic shall occur from any method
pub trait Integer {
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn shl(self, other: Self) -> Self;
    fn shr(self, other: Self) -> Self;

    fn lt(self, other: Self) -> bool;
    fn gte(self, other: Self) -> bool;
    fn eq(self, other: Self) -> bool;
    fn neq(self, other: Self) -> bool;

    fn and(self, other: Self) -> Self;
    fn or(self, other: Self) -> Self;
    fn xor(self, other: Self) -> Self;
}
macro_rules! impl_integer {
    ($($name:ident),*) => {
        $(
            impl Integer for $name {
                #[inline(always)]
                fn add(self, other: Self) -> Self { $name::wrapping_add(self, other) }
                #[inline(always)]
                fn sub(self, other: Self) -> Self { $name::wrapping_sub(self, other) }
                #[inline(always)]
                fn shl(self, other: Self) -> Self { $name::wrapping_shl(self, other as _) }
                #[inline(always)]
                fn shr(self, other: Self) -> Self { $name::wrapping_shr(self, other as _) }
            
                #[inline(always)]
                fn lt(self, other: Self) -> bool { self < other }
                #[inline(always)]
                fn gte(self, other: Self) -> bool { self >= other }
                #[inline(always)]
                fn eq(self, other: Self) -> bool { self == other }
                #[inline(always)]
                fn neq(self, other: Self) -> bool { self != other }
            
                #[inline(always)]
                fn and(self, other: Self) -> Self { self & other }
                #[inline(always)]
                fn or(self, other: Self) -> Self { self | other }
                #[inline(always)]
                fn xor(self, other: Self) -> Self { self ^ other }
            }
        )*
    };
}
impl_integer! { u32, i32, u64, i64, u128, i128, usize, isize }

#[derive(Debug, PartialEq, Eq)]
pub enum RegisterWidth {
    Bits32,
    Bits64,
    Bits128
}

/// Byte order independent interpretations for a register
pub trait Xlen {
    type Signed: Integer;
    type Unsigned: Integer;
    type Bytes;
    const WIDTH: RegisterWidth;

    /// Interpret the register as a signed value
    fn signed(self) -> Self::Signed;
    /// Interpret the register as an unsigned value
    fn unsigned(self) -> Self::Unsigned;

    /// Create a register from a signed value
    fn from_signed(from: Self::Signed) -> Self;
    /// Create a register from an unsigned value
    fn from_unsigned(from: Self::Unsigned) -> Self;

    /// Return the unsigned value added to an unsigned system-native value
    fn append(self, offset: usize) -> Self::Unsigned;
}

/// Operations on a register carried out by system instructions
pub trait Register: Xlen + Sized {
    /// Add 2 registers with signed arithmetic
    fn add_signed(self, other: Self) -> Self {
        Self::from_signed(self.signed().add(other.signed()))
    }
    /// Add 2 registers with unsigned arithmetic
    fn add_unsigned(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().add(other.unsigned()))
    }
    /// Subtract other from self where both are unsigned
    fn sub_unsigned(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().sub(other.unsigned()))
    }

    /// Shift left by a certain number of bits
    fn shl(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().shl(other.unsigned()))
    }
    /// Shift right by a certain number of bits
    fn shr(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().shr(other.unsigned()))
    }
    /// Arithmetic shift right by a certain number of bits; shift right, preserving the sign
    fn sha(self, other: Self) -> Self {
        Self::from_signed(self.signed().shr(other.signed()))
    }

    /// Applies the bitwise AND operation to self and other
    fn and(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().and(other.unsigned()))
    }
    /// Applies the bitwise OR operation to self and other
    fn or(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().or(other.unsigned()))
    }
    /// Applies the bitwise XOR operation to self and other
    fn xor(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().xor(other.unsigned()))
    }

    /// Tests if self is equal to other
    fn eq(self, other: Self) -> bool {
        self.unsigned().eq(other.unsigned())
    }
    /// Tests if self is not equal to other
    fn neq(self, other: Self) -> bool {
        self.unsigned().neq(other.unsigned())
    }
    /// Tests if self is less than other where both are interpreted as signed values
    fn lt_signed(self, other: Self) -> bool {
        self.signed().lt(other.signed())
    }
    /// Tests if self is less than other where both are interpreted as unsigned values
    fn lt_unsigned(self, other: Self) -> bool {
        self.unsigned().lt(other.unsigned())
    }
    /// Tests if self is greater than or equal to other where both are interpreted as signed values
    fn gte_signed(self, other: Self) -> bool {
        self.signed().gte(other.signed())
    }
    /// Tests if self is greater than or equal to other where both are interpreted as unsigned values
    fn gte_unsigned(self, other: Self) -> bool {
        self.unsigned().gte(other.unsigned())
    }

    /// Create a register with the lower portion set to the byte and the rest set to the msb of the byte
    fn sign_extended_byte(byte: u8) -> Self;
    /// Create a register with the lower portion set to the byte and the rest set to zeroes
    fn zero_extended_byte(byte: u8) -> Self;
    /// Create a register with the lower portion set to the half and the rest set to the msb of the half
    fn sign_extended_half(half: [u8; 2]) -> Self;
    /// Create a register with the lower portion set to the half and the rest set to zeroes
    fn zero_extended_half(half: [u8; 2]) -> Self;
    /// Create a register with the lower portion set to the word and the rest set to the msb of the word
    fn sign_extended_word(word: [u8; 4]) -> Self;
    /// Create a register with the lower portion set to the word and the rest set to zeroes
    fn zero_extended_word(word: [u8; 4]) -> Self;
    /// Create a register with the lower portion set to the double and the rest set to the msb of the double
    fn sign_extended_double(double: [u8; 8]) -> Self;
    /// Create a register with the lower portion set to the double and the rest set to zeroes
    fn zero_extended_double(double: [u8; 8]) -> Self;

    /// Get the lowest byte
    fn byte(self) -> u8;
    /// Get the lowest half
    fn half(self) -> [u8; 2];
    /// Get the lowest word
    fn word(self) -> [u8; 4];
    /// Get the lowest double
    fn double(self) -> [u8; 8];
}

/// A 32-bit value with byte-order and sign independent actions
#[derive(Clone, Copy, Debug)]
pub struct Register32(pub [u8; 4]);
impl Xlen for Register32 {
    type Signed = i32;
    type Unsigned = u32;
    type Bytes = [u8; 4];
    const WIDTH: RegisterWidth = RegisterWidth::Bits32;
    fn signed(self) -> i32 {
        i32::from_le_bytes(self.0)
    }
    fn unsigned(self) -> u32 {
        u32::from_le_bytes(self.0)
    }
    fn from_signed(from: i32) -> Self {
        Self(from.to_le_bytes())
    }
    fn from_unsigned(from: u32) -> Self {
        Self(from.to_le_bytes())
    }
    fn append(self, value: usize) -> u32 {
        self.unsigned() + value as u32
    }
}
impl Register for Register32 {
    #[inline]
    fn sign_extended_byte(byte: u8) -> Self {
        let extended = if byte & 0x80 != 0 { 0xFF } else { 0 };
        Self([byte, extended, extended, extended])
    }
    #[inline]
    fn zero_extended_byte(byte: u8) -> Self {
        Self([byte, 0, 0, 0])
    }
    #[inline]
    fn sign_extended_half(half: [u8; 2]) -> Self {
        let extended = if half[1] & 0x80 != 0 { 0xFF } else { 0 };
        Self([half[0], half[1], extended, extended])
    }
    #[inline]
    fn zero_extended_half(half: [u8; 2]) -> Self {
        Self([half[0], half[1], 0, 0])
    }
    #[inline(always)]
    fn sign_extended_word(word: [u8; 4]) -> Self {
        Self(word)
    }
    #[inline(always)]
    fn zero_extended_word(word: [u8; 4]) -> Self {
        Self(word)
    }
    #[inline(always)]
    fn sign_extended_double(_: [u8; 8]) -> Self {
        panic!("Cannot create a 32 bit register from a 64 bit value")
    }
    #[inline(always)]
    fn zero_extended_double(_: [u8; 8]) -> Self {
        panic!("Cannot create a 32 bit register from a 64 bit value")
    }

    #[inline(always)]
    fn byte(self) -> u8 { self.0[0] }
    #[inline(always)]
    fn half(self) -> [u8; 2] { [self.0[0], self.0[1]] }
    #[inline(always)]
    fn word(self) -> [u8; 4] { self.0 }
    #[inline(always)]
    fn double(self) -> [u8; 8] { panic!("Cannot get a 64 bit value from a 32 bit register") }
}
impl Default for Register32 {
    fn default() -> Self {
        Self([0, 0, 0, 0])
    }
}