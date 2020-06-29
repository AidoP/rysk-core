use crate::Register;

pub trait Variant {
    fn decode(instruction: [u8; 4]) -> Self;
}

#[inline(always)]
fn destination(instruction: [u8; 4]) -> usize {
    (((instruction[0] & 0x80) >> 7) | ((instruction[1] & 0x0F) << 1)) as _
}
#[inline(always)]
fn source1(instruction: [u8; 4]) -> usize {
    (((instruction[1] & 0x80) >> 7) | ((instruction[2] & 0x0F) << 1)) as _
}
#[inline(always)]
fn source2(instruction: [u8; 4]) -> usize {
    (((instruction[2] & 0xF0) >> 4) | ((instruction[3] & 0x01) << 4)) as _
}

pub struct R {
    pub destination: usize,
    pub source1: usize,
    pub source2: usize
}
impl Variant for R {
    fn decode(instruction: [u8; 4]) -> Self {
        Self {
            destination: destination(instruction),
            source1: source1(instruction),
            source2: source2(instruction),
        }
    }
}

pub struct I<R: Register> {
    pub destination: usize,
    pub source: usize,
    pub immediate: R
}
impl<R: Register> Variant for I<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            destination: destination(instruction),
            source: source1(instruction),
            immediate: R::sign_extended_half([((instruction[2] & 0xF0) >> 4) | ((instruction[3] & 0x0F) << 4), ((instruction[3] & 0xF0) >> 4) | if signed { 0xF0 } else { 0 }])
        }
    }
}

pub struct S<R: Register> {
    pub source1: usize,
    pub source2: usize,
    pub immediate: R
}
impl<R: Register> Variant for S<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            source1: source1(instruction),
            source2: source2(instruction),
            immediate: R::sign_extended_half([((instruction[0] & 0x80) >> 7) | ((instruction[1] & 0x0F) << 1) | ((instruction[3] & 0x0E) << 4), ((instruction[3] & 0xF0) >> 4) | if signed { 0xF0 } else { 0 }])
        }
    }
}

pub struct B<R: Register> {
    pub source1: usize,
    pub source2: usize,
    pub immediate: R
}
impl<R: Register> Variant for B<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            source1: source1(instruction),
            source2: source2(instruction),
            immediate: R::sign_extended_half([
                ((instruction[1] & 0xF) << 1) | ((instruction[3] & 0x0E) << 4),
                ((instruction[3] & 0x70) >> 4) | ((instruction[0] & 0x80) >> 4) | ((instruction[3] & 0x80) >> 3) | if signed { 0xE0 } else { 0 },
            ])
        }
    }
}

pub struct U<R: Register> {
    pub destination: usize,
    pub immediate: R
}
impl<R: Register> Variant for U<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        Self {
            destination: destination(instruction),
            immediate: R::sign_extended_word([0, instruction[1] & 0xF0, instruction[2], instruction[3]])
        }
    }
}

pub struct J<R: Register> {
    pub destination: usize,
    pub immediate: R
}
impl<R: Register> Variant for J<R> {
    fn decode(instruction: [u8; 4]) -> Self {
        let signed = instruction[3] & 0x80 != 0;
        Self {
            destination: destination(instruction),
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