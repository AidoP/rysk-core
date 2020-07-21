#[cfg(feature = "ext-m")]
mod ext_m_tests {

    use rysk_core::*;
    #[test]
    fn test_mul() {
        let all_bits: Register32 = 0xFFFF_FFFFu32.into();

        assert_eq!(all_bits.mul(2.into()), 0xFFFF_FFFEu32.into()); // 0xFFFF_FFFE == -2
        assert_eq!(all_bits.mulh(2.into()), (-1).into());
        assert_eq!(all_bits.mulhu(2.into()), 0x1u32.into());
        assert_eq!(all_bits.mulhsu(2.into()), (-1).into());
    }

    #[test]
    fn test_div() { 
        let neg_sixteen: Register32 = (-16).into();
        let thirty_five: Register32 = 35.into();
        assert_eq!(neg_sixteen.div(4.into()), (-4).into());

        assert_eq!(thirty_five.div((-9).into()), (-3).into());
        assert_eq!(thirty_five.div(9.into()), 3.into());
        assert_eq!(thirty_five.divu(9.into()), 3.into());

        // Division by zero is defined in RISCV to avoid the complexity of adding a trap handler in simple implementations
        assert_eq!(thirty_five.div(0.into()), (-1).into());
        assert_eq!(thirty_five.divu(0.into()), 0xFFFF_FFFFu32.into());

        // Overflow
        let max_neg: Register32 = std::i32::MIN.into();
        assert_eq!(max_neg.div((-1).into()), max_neg);
    }

    #[test]
    fn test_rem() { 
        let neg_nine: Register32 = (-9).into();
        let thirty_five: Register32 = 35.into();

        // Sign is of the dividend
        assert_eq!(neg_nine.rem(4.into()), (-1).into());
        assert_eq!(thirty_five.rem((-6).into()), 5.into());

        assert_eq!(thirty_five.rem(6.into()), 5.into());
        assert_eq!(thirty_five.remu(6.into()), 5.into());

        // Remainder from division by 0 is the dividend
        assert_eq!(thirty_five.rem(0.into()), thirty_five);

        // Overflow
        let max_neg: Register32 = (-2i32).pow(31).into();
        assert_eq!(max_neg.rem((-1).into()), 0.into());
    }
}