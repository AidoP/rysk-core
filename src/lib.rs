#![allow(clippy::unit_arg)]
//! Rysk Core assists in the creation of RISCV virtual machines, providing virtual harts.

pub mod variant;
pub mod register;
pub mod system;

pub use variant::Variant;
pub use register::{Register,Register32,Xlen};
pub use system::{Core, Mmu};
