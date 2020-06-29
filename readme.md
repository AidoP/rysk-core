# Rysk Core
RISCV decoding and execution primitives. All you need to implement your own virtual RISCV machine.

## [Virtual Machine](https://gitlab.com//AidoP1/rysk)
If you are looking for a working virtual machine you want [Rysk](https://gitlab.com//AidoP1/rysk). This is a library for building RISCV virtual machines.

# Usage
First, add a dependency to your `Cargo.toml`:
```toml
    [dependencies]
    # Note, this will not work until the crate is published to crates.io
    rysk-core = "0.0.1"
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
- [ ] Support for all base extensions
- [x] Dependency-free

Future Goals
- [ ] Performance

# Compliance

| Extension | Support |
| :-------- | :-----: |
| RV32I     | Partial |
| RV32E     | None    |
| RV64I     | None    |
| *Zifencei* | None    |
| *Zicsr*   | None    |
| M         | None    |
| A         | None    |
| F         | None    |
| D         | None    |
| Q         | None    |
| C         | None    |
| G         | Partial |
| LBJTPV    | N/A     |
| *Zam*, *Ztso* | N/A |