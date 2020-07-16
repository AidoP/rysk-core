# Rysk Core
RISCV decoding and execution primitives. All you need to implement your own virtual RISCV machine.

## [Virtual Machine](https://gitlab.com/AidoP1/rysk)
If you are looking for a working virtual machine you want [Rysk](https://gitlab.com/AidoP1/rysk). This is a library for building RISCV virtual machines.

# Usage
First, add a dependency to your `Cargo.toml`:
```toml
    [dependencies]
    rysk-core = "0.0.2"
```

Then in your project,
```rust
    // Implement a memory management unit
    impl rysk_core::Mmu</* Register Size */> for YourMmu { /* ... */ }

    // Run your system
    fn main() {
        let mmu = YourMmu::new();
        let core = rysk_core::Core::</* Register Size */>::new(/* PC initial address */);

        loop {
            // fetch, then decode & execute
            core.execute(mmu.fetch_instruction(core.pc), &mut mmu).expect("Unable to decode instruction");
        }
    }
```

# Goals
Current Goals
- [ ] Compatability with all platforms `std` supports
- [ ] Support for RV32IMA and RV64IMA
- [x] Dependency-free
- [ ] Support for the Privileged ISA

Future Goals
- [ ] Performance
- [ ] Support for all base extensions
- [ ] Debugging tools

# Compliance

> Note: This list does not account for errors and/or bugs in the implementation. Perfect compliance is not guaranteed
> 
> If an instruction does not behave as specified please leave an issue

|   Extension   | Support |
| :-----------: | :-----: |
| RV32I         | Partial |
| RV32E         | None*   |
| RV64I         | Full    |
| RV128I        | None    |
| *Zifencei*    | None    |
| *Zicsr*       | Partial |
| N             | None    |
| M             | None    |
| A             | None    |
| F             | None    |
| D             | None    |
| Q             | None    |
| C             | None    |
| G             | Partial |
| LBJTPV        | N/A     |
| *Zam*         | N/A     |
| *Ztso*        | Always  |

> *Support for the embedded extension is low priority. Const-generics will make implementing this feature much cleaner and as such supporting RV32E is not planned until const-generics land in stable Rust.

### Privilege Levels
|    Level   | Support |
| :--------: | :-----: |
| Machine    | Partial |
| Supervisor | None    |
| User       | None    |

# Extensions
Most extensions are enabled through cargo features.

| Extension |   Feature   |
| :-------: | :---------: |
| *Zicsr*   | **default** |
| *Zicsr*   | ext-csr     |

The base extension (RV32I, RV64I or RV128I) is set through the generic register type used. `MXLEN` is a set at compile time and therefore cannot be changed by RISCV programs (ie. `misa[MXLEN]` is read-only).