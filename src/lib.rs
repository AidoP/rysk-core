#![allow(clippy::unit_arg)]
//! Rysk Core assists in the creation of RISCV virtual machines, providing virtual harts.

pub mod variant;
pub mod register;
pub mod system;

pub use variant::Variant;
pub use register::{Register,Register32,Xlen};
pub use system::{Core, Mmu};

pub mod version {
    pub const PATCH: u8 = 2;
    pub const MINOR: u8 = 0;
    pub const MAJOR: u8 = 0;
}