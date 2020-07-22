#![allow(clippy::unit_arg)]
//! Rysk Core assists in the creation of RISCV virtual machines, providing virtual harts.
//! 
//! Usage:
//! - Implement the `system::Mmu` trait
//! - Create an instance of `system::Core` with `register::Register*` as the generic type
//! - Execute instructions using `system::Core::execute()`

pub mod variant;
pub mod register;
pub mod system;

pub use system::{ Core, Mmu };
pub use register::{ Register, Register32, Register64, RegisterSize };

#[cfg(feature = "ext-csr")]
pub mod csr;

pub mod version {
    pub const PATCH: u8 = 3;
    pub const MINOR: u8 = 0;
    pub const MAJOR: u8 = 0;
}