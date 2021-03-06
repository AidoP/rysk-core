/// An integer which may be multiplied in the way specified by the ISA
/// A seperate trait is used as implementation in a macro is too difficult
pub trait Multiply<S, U>: Sized {
    /// Multiply two signed integers, returning both the high, overflowed, bits and the lower bits.
    fn muls(first: S, second: S) -> (S, S);
    /// Multiply two unsigned integers, returning both the high, overflowed, bits and the lower bits.
    fn mulu(first: U, second: U) -> (U, U);
    /// Multiply a signed integer by an unsigned integer, returning both the high, overflowed, bits and the lower bits.
    fn mulsu(first: S, second: U) -> (S, S);
}

macro_rules! impl_multiply {
    ($(($signed:ident, $unsigned:ident, * = $bytes:expr) -> ($signed_long:ident, $unsigned_long:ident)),*) => {
        $(
            impl Multiply<$signed, $unsigned> for $signed {
                fn muls(first: $signed, second: $signed) -> ($signed, $signed) {
                    let result = (first as $signed_long).saturating_mul(second as _);
                    (result as _, (result >> ($bytes * 8)) as _)
                }
                fn mulu(first: $unsigned, second: $unsigned) -> ($unsigned, $unsigned) {
                    let result = (first as $unsigned_long).saturating_mul(second as _);
                    (result as _, (result >> ($bytes * 8)) as _)
                }
                fn mulsu(first: $signed, second: $unsigned) -> ($signed, $signed) {
                    let result = (first as $signed_long).saturating_mul(second as _);
                    (result as _, (result >> ($bytes * 8)) as _)
                }
            }
        )*
    };
}

impl_multiply!{(i32, u32, * = 4) -> (i64, u64), (i64, u64, * = 8) -> (i128, u128)}

#[cfg(target_pointer_width = "32")]
impl_multiply!{(isize, usize, * = 4) -> (i64, u64)}
#[cfg(target_pointer_width = "64")]
impl_multiply!{(isize, usize, * = 8) -> (i128, u128)}

/// An integer type which can apply operations as specified by the ISA
/// No panic shall occur from any method
pub trait Integer: Default {
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
    fn shl(self, other: Self) -> Self;
    fn shr(self, other: Self) -> Self;

    fn div(self, other: Self) -> Self;
    fn rem(self, other: Self) -> Self;

    fn lt(self, other: Self) -> bool;
    fn gte(self, other: Self) -> bool;
    fn eq(self, other: Self) -> bool;
    fn neq(self, other: Self) -> bool;

    fn and(self, other: Self) -> Self;
    fn or(self, other: Self) -> Self;
    fn xor(self, other: Self) -> Self;
    fn not(self) -> Self;
}
macro_rules! impl_integer {
    ($($name:ident(* = $shift:expr, $larger_type:ident)),*) => {
        $(
            /// A trait needs to be implemented on something, so currently just using the unsigned type
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
                fn div(self, other: Self) -> Self {
                    if other == 0 {
                        -1 as _
                    } else if self == std::$name::MIN && other == -1 as _ {
                        self
                    } else {
                        self / other
                    }
                }
                #[inline(always)]
                fn rem(self, other: Self) -> Self {
                    if other == 0 {
                        self
                    } else if self == std::$name::MIN && other == -1 as _ {
                        0
                    } else {
                        self % other
                    }
                }

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
                #[inline(always)]
                fn not(self) -> Self { !self }
            }
        )*
    };
}
impl_integer! { u32(* = 4, u64), i32(* = 4, i64), u64(* = 8, u128), i64(* = 4, u128), usize(* = 8, usize), isize(* = 8, usize) }

#[derive(Debug, PartialEq, Eq)]
pub enum RegisterWidth {
    Bits32,
    Bits64
}

/// Byte order independent interpretations for a register
pub trait Xlen {
    /// The concrete signed type that the inner value represents
    type Signed: Integer + Multiply<Self::Signed, Self::Unsigned> + Copy;
    /// The concrete unsigned type that the inner value represents
    type Unsigned: Integer + Copy;
    /// The width of the register. Defines the available instruction set (ie. RV32I, RV64I or RV128I)
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
    /// Return the value as an unsigned system-native value
    fn usize(self) -> usize;

    /// Sets the high bit to interrupt and the least significant byte to cause, as specified for the *cause CSR
    #[cfg(feature = "ext-csr")]
    fn trap_cause(cause: u8, interrupt: bool) -> Self;
}

/// Operations on a register carried out by system instructions
pub trait Register: Xlen + Sized + Default + Copy {
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

    #[cfg(feature = "ext-m")]
    /// Multiplication returning the low bits
    fn mul(self, other: Self) -> Self {
        Self::from_signed(Self::Signed::muls(self.signed(), other.signed()).0)
    }
    #[cfg(feature = "ext-m")]
    /// Signed multiplication returning the high bits
    fn mulh(self, other: Self) -> Self {
        Self::from_signed(Self::Signed::muls(self.signed(), other.signed()).1)
    }
    #[cfg(feature = "ext-m")]
    /// Unsigned multiplication returning the high bits
    fn mulhu(self, other: Self) -> Self {
        Self::from_unsigned(Self::Signed::mulu(self.unsigned(), other.unsigned()).1)
    }
    #[cfg(feature = "ext-m")]
    /// Signed-Unsigned multiplication returning the high bits
    fn mulhsu(self, other: Self) -> Self {
        Self::from_signed(Self::Signed::mulsu(self.signed(), other.unsigned()).1)
    }
    #[cfg(feature = "ext-m")]
    /// Signed division rounding toward zero
    fn div(self, other: Self) -> Self {
        Self::from_signed(self.signed().div(other.signed()))
    }
    #[cfg(feature = "ext-m")]
    /// Unsigned division rounding toward zero
    fn divu(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().div(other.unsigned()))
    }
    #[cfg(feature = "ext-m")]
    /// Remainder of the equivalent signed division with the sign matching the dividend
    fn rem(self, other: Self) -> Self {
        Self::from_signed(self.signed().rem(other.signed()))
    }
    #[cfg(feature = "ext-m")]
    /// Remainder of the equivalent unsigned division 
    fn remu(self, other: Self) -> Self {
        Self::from_unsigned(self.unsigned().rem(other.unsigned()))
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
    /// Applies the bitwise NOT operation to self
    fn not(self) -> Self {
        Self::from_unsigned(self.unsigned().not())
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

/// A 32-bit value with byte-order and sign independent operations
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Register32(pub [u8; 4]);
impl Xlen for Register32 {
    type Signed = i32;
    type Unsigned = u32;
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
    fn usize(self) -> usize {
        self.unsigned() as usize
    }
    #[cfg(feature = "ext-csr")]
    fn trap_cause(cause: u8, interrupt: bool) -> Self {
        Self([if interrupt { 0x80 } else { 0 }, 0, 0, cause])
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
impl From<u32> for Register32 {
    fn from(value: u32) -> Self {
        Self::from_unsigned(value)
    }
}
impl From<i32> for Register32 {
    fn from(value: i32) -> Self {
        Self::from_signed(value)
    }
}

/// A 64-bit value with byte-order and sign independent operations
#[derive(Clone, Copy, Debug)]
pub struct Register64(pub [u8; 8]);
impl Register64 {
    /// Split the 64 bit register into 2 32 bit registers
    /// The lower word is returned as the first item in the tuple
    pub fn split(self) -> (Register32, Register32) {
        (Register32([self.0[0], self.0[1], self.0[2], self.0[3]]), Register32([self.0[4], self.0[5], self.0[6], self.0[7]]))
    }
}
impl Xlen for Register64 {
    type Signed = i64;
    type Unsigned = u64;
    const WIDTH: RegisterWidth = RegisterWidth::Bits32;
    fn signed(self) -> i64 {
        i64::from_le_bytes(self.0)
    }
    fn unsigned(self) -> u64 {
        u64::from_le_bytes(self.0)
    }
    fn from_signed(from: i64) -> Self {
        Self(from.to_le_bytes())
    }
    fn from_unsigned(from: u64) -> Self {
        Self(from.to_le_bytes())
    }
    fn append(self, value: usize) -> u64 {
        self.unsigned() + value as u64
    }
    fn usize(self) -> usize {
        self.unsigned() as usize
    }
    #[cfg(feature = "ext-csr")]
    fn trap_cause(cause: u8, interrupt: bool) -> Self {
        Self([if interrupt { 0x80 } else { 0 }, 0, 0, 0, 0, 0, 0, cause])
    }
}
impl Register for Register64 {
    #[inline]
    fn sign_extended_byte(byte: u8) -> Self {
        let extended = if byte & 0x80 != 0 { 0xFF } else { 0 };
        Self([byte, extended, extended, extended, extended, extended, extended, extended])
    }
    #[inline]
    fn zero_extended_byte(byte: u8) -> Self {
        Self([byte, 0, 0, 0, 0, 0, 0, 0])
    }
    #[inline]
    fn sign_extended_half(half: [u8; 2]) -> Self {
        let extended = if half[1] & 0x80 != 0 { 0xFF } else { 0 };
        Self([half[0], half[1], extended, extended, extended, extended, extended, extended])
    }
    #[inline]
    fn zero_extended_half(half: [u8; 2]) -> Self {
        Self([half[0], half[1], 0, 0, 0, 0, 0, 0])
    }
    #[inline(always)]
    fn sign_extended_word(word: [u8; 4]) -> Self {
        let extended = if word[3] & 0x80 != 0 { 0xFF } else { 0 };
        Self([word[0], word[1], word[2], word[3], extended, extended, extended, extended])
    }
    #[inline(always)]
    fn zero_extended_word(word: [u8; 4]) -> Self {
        Self([word[0], word[1], word[2], word[3], 0, 0, 0, 0])
    }
    #[inline(always)]
    fn sign_extended_double(double: [u8; 8]) -> Self {
        Self(double)
    }
    #[inline(always)]
    fn zero_extended_double(double: [u8; 8]) -> Self {
        Self(double)
    }

    #[inline(always)]
    fn byte(self) -> u8 { self.0[0] }
    #[inline(always)]
    fn half(self) -> [u8; 2] { [self.0[0], self.0[1]] }
    #[inline(always)]
    fn word(self) -> [u8; 4] { [self.0[0], self.0[1], self.0[2], self.0[3]] }
    #[inline(always)]
    fn double(self) -> [u8; 8] { self.0 }
}
impl Default for Register64 {
    fn default() -> Self {
        Self([0, 0, 0, 0, 0, 0, 0, 0])
    }
}

/// A native register-sized value with byte-order and sign independent actions
#[cfg(not(target_pointer_width = "16"))]
#[derive(Clone, Copy, Debug)]
pub struct RegisterSize(pub [u8; std::mem::size_of::<usize>()]);
#[cfg(not(target_pointer_width = "16"))]
impl Xlen for RegisterSize {
    type Signed = isize;
    type Unsigned = usize;

    #[cfg(target_pointer_width = "32")]
    const WIDTH: RegisterWidth = RegisterWidth::Bits32;
    #[cfg(target_pointer_width = "64")]
    const WIDTH: RegisterWidth = RegisterWidth::Bits64;

    fn signed(self) -> isize {
        isize::from_le_bytes(self.0)
    }
    fn unsigned(self) -> usize {
        usize::from_le_bytes(self.0)
    }
    fn from_signed(from: isize) -> Self {
        Self(from.to_le_bytes())
    }
    fn from_unsigned(from: usize) -> Self {
        Self(from.to_le_bytes())
    }
    fn append(self, value: usize) -> usize {
        self.unsigned() + value as usize
    }
    fn usize(self) -> usize {
        self.unsigned()
    }
    #[cfg(feature = "ext-csr")]
    fn trap_cause(cause: u8, interrupt: bool) -> Self {
        let msb = if interrupt { 0x80 } else { 0 };
        #[cfg(target_pointer_width = "32")]
        { Self([msb, 0, 0, cause]) }
        #[cfg(target_pointer_width = "64")]
        { Self([msb, 0, 0, 0, 0, 0, 0, cause]) }
    }
}
#[cfg(not(target_pointer_width = "16"))]
impl Register for RegisterSize {
    #[inline]
    fn sign_extended_byte(byte: u8) -> Self {
        let extended = if byte & 0x80 != 0 { 0xFF } else { 0 };
        #[cfg(target_pointer_width = "32")]
        {Self([byte, extended, extended, extended])}
        #[cfg(target_pointer_width = "64")]
        {Self([byte, extended, extended, extended, extended, extended, extended, extended])}
    }
    #[inline]
    fn zero_extended_byte(byte: u8) -> Self {
        #[cfg(target_pointer_width = "32")]
        {Self([byte, 0, 0, 0])}
        #[cfg(target_pointer_width = "64")]
        {Self([byte, 0, 0, 0, 0, 0, 0, 0])}
    }
    #[inline]
    fn sign_extended_half(half: [u8; 2]) -> Self {
        let extended = if half[1] & 0x80 != 0 { 0xFF } else { 0 };
        #[cfg(target_pointer_width = "32")]
        {Self([half[0], half[1], extended, extended])}
        #[cfg(target_pointer_width = "64")]
        {Self([half[0], half[1], extended, extended, extended, extended, extended, extended])}
    }
    #[inline]
    fn zero_extended_half(half: [u8; 2]) -> Self {
        #[cfg(target_pointer_width = "32")]
        {Self([half[0], half[1], 0, 0])}
        #[cfg(target_pointer_width = "64")]
        {Self([half[0], half[1], 0, 0, 0, 0, 0, 0])}
    }
    #[inline(always)]
    fn sign_extended_word(word: [u8; 4]) -> Self {
        let extended = if word[3] & 0x80 != 0 { 0xFF } else { 0 };
        #[cfg(target_pointer_width = "32")]
        {Self(word)}
        #[cfg(target_pointer_width = "64")]
        {Self([word[0], word[1], word[2], word[3], extended, extended, extended, extended])}
    }
    #[inline(always)]
    fn zero_extended_word(word: [u8; 4]) -> Self {
        #[cfg(target_pointer_width = "32")]
        {Self(word)}
        #[cfg(target_pointer_width = "64")]
        {Self([word[0], word[1], word[2], word[3], 0, 0, 0, 0])}
    }
    #[inline(always)]
    fn sign_extended_double(double: [u8; 8]) -> Self {
        #[cfg(target_pointer_width = "32")]
        {panic!("Cannot create a 32 bit register from a 64 bit value")}
        #[cfg(target_pointer_width = "64")]
        {Self(double)}
    }
    #[inline(always)]
    fn zero_extended_double(double: [u8; 8]) -> Self {
        #[cfg(target_pointer_width = "32")]
        {panic!("Cannot create a 32 bit register from a 64 bit value")}
        #[cfg(target_pointer_width = "64")]
        {Self(double)}
    }

    #[inline(always)]
    fn byte(self) -> u8 { self.0[0] }
    #[inline(always)]
    fn half(self) -> [u8; 2] { [self.0[0], self.0[1]] }
    #[inline(always)]
    fn word(self) -> [u8; 4] { [self.0[0], self.0[1], self.0[2], self.0[3]] }
    #[inline(always)]
    fn double(self) -> [u8; 8] {
        #[cfg(not(target_pointer_width = "32"))]
        {[self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7]]}
        #[cfg(target_pointer_width = "32")]
        {panic!("Cannot create a 64 bit value from a 32 bit register")}
    }
}
#[cfg(not(target_pointer_width = "16"))]
impl Default for RegisterSize {
    fn default() -> Self {
        #[cfg(target_pointer_width = "32")]
        {Self([0, 0, 0, 0])}
        #[cfg(target_pointer_width = "64")]
        {Self([0, 0, 0, 0, 0, 0, 0, 0])}
    }
}