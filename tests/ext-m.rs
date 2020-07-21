#[cfg(feature = "ext-m")]
mod ext_tests {

    use rysk_core::*;
    #[test]
    fn test_mul() {
        use register::*;
        let all_bits: Register32 = 0xFFFF_FFFFu32.into();

        assert_eq!(all_bits.mul(2.into()), 0xFFFF_FFFEu32.into()); // 0xFFFF_FFFE == -2
        assert_eq!(all_bits.mulh(2.into()), (-1).into());
        assert_eq!(all_bits.mulhu(2.into()), 0x1u32.into());
        assert_eq!(all_bits.mulhsu(2.into()), (-1).into());
    }
}