#![allow(clippy::unit_arg)]
//! Rysk Core assists in the creation of RISCV virtual machines, providing virtual harts.

pub mod variant;
pub mod register;
pub mod system;

pub mod prelude {
    pub use crate::variant::Variant;
    pub use crate::register::{Register,Register32,Xlen};
    pub use crate::system::{Core, Mmu};
}

#[cfg(feature = "ext-csr")]
pub mod csr;

pub mod version {
    pub const PATCH: u8 = 2;
    pub const MINOR: u8 = 0;
    pub const MAJOR: u8 = 0;
}