use crate::register::{Register,Register32,Register64};

/// The Control Status Registers (CSR) a single HART must provide storage for to comply with the privileged ISA
/// Other CSR's may not need storage and as such are not a part of this struct
pub struct Csr<R: Register> {
    /// The ID of this hart
    pub mhartid: R,
    /// The address of a potentially vectorised interupt handler
    pub mtvec: R,
    /// Delegation of exceptions to lower modes
    pub medeleg: R,
    /// Delegation of interrupts to lower modes
    pub mideleg: R,
    /// Sets if interrupts are enabled
    pub mie: R,
    /// States if an interrupt is pending
    pub mip: R,
    /// Counts the number of cycles the hart has executed. As there is no speculative execution or other operations minstret is the same as this value
    pub mcycle: Register64,
    /// Determine if counters are accessible in lower privilege modes
    pub mcounteren: Register32,
    /// Scratch register dedicated to machine-mode usage
    pub mscratch: R,
    /// The virtual address of an interrupted or excepted instruction in machine-mode
    pub mepc: R,
    /// The cause of an interrupt or exception
    pub mcause: R,
    /// An implementation-defined value set during a trap
    pub mtval: R,
}

impl<R: Register> Csr<R> {
    pub fn new(hart: R::Unsigned, trap_address: R::Unsigned) -> Self {
        Self {
            mhartid: R::from_unsigned(hart),
            mtvec: R::from_unsigned(trap_address),
            medeleg: Default::default(),
            mideleg: Default::default(),
            mie: Default::default(),
            mip: Default::default(),
            mcycle: Default::default(),
            mcounteren: Default::default(),
            mscratch: Default::default(),
            mepc: Default::default(),
            mcause: Default::default(),
            mtval: Default::default()
        }
    }
}