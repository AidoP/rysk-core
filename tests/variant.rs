use rysk_core::*;
use variant::Variant;
#[test]
fn variant_r() {
    const ALL_BITS: [u8; 4] = [0xFF; 4];
    assert_eq!(variant::R::decode(ALL_BITS), variant::R {
        destination: 0x1F,
        source1: 0x1F,
        source2: 0x1F
    });
}

#[test]
fn variant_i() {
    const ALL_BITS: [u8; 4] = [0xFF; 4];
    assert_eq!(variant::I::<Register32>::decode(ALL_BITS), variant::I {
        destination: 0x1F,
        source: 0x1F,
        immediate: 0xFFFFFFFFu32.into()
    });
}

#[test]
fn variant_c() {
    const ALL_BITS: [u8; 4] = [0xFF; 4];
    assert_eq!(variant::C::decode(ALL_BITS), variant::C {
        destination: 0x1F,
        source: 0x1F,
        csr: 0x0FFF,
    });
}

#[test]
fn variant_s() {
    const ALL_BITS: [u8; 4] = [0xFF; 4];
    assert_eq!(variant::S::<Register32>::decode(ALL_BITS), variant::S {
        source1: 0x1F,
        source2: 0x1F,
        immediate: 0xFFFFFFFFu32.into()
    });
}