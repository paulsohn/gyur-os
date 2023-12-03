# GYUR OS

This is yet another rust implementation of [MikanOS](https://github.com/uchan-nos/mikanos) by @uchan-nos, intended to run on x86-64 systems.

Gyul(귤) is Korean translation for mikan(tangerine). The last letter is R with respect to **R**ust language.

## Build
This repository uses Linux shell scripts to build and execute, due to cargo issues on package-wise build target settings.
Other OS environments are currently not supported, but I will eventually switch shell scripts into `build.rs`-using processes once I get more familiar to them.

As the bootable device image for QEMU, `./disk.img` is intended to be used. This is initially not included in the repo, so create it manually with `./sh/create_disk.sh`.

To build, run `./sh/build.sh` on top directory.
If you just want a compile check, run `./sh/check.sh`.

After build, run `./sh/run_qemu.sh` for executing QEMU.

## Roadmap, implementation notes, and issues
- [x] **Day 01 (Hello World)** '23.07.07.
- [x] **Day 02 (Memory Map)** '23.08.09.
  * For EDK2 APIs, `uefi-rs` crate was used.
- [x] **Day 03 (Bootloader and Frame Buffer)** '23.08.16.
  * To match function call ABI between bootloader and kernel, use "sysv64" ABI(i.e.`extern "sysv64"`) "C" ABI seems to have different meaning between EFI and ELF targets.
  * There was a bug that the actual kernel entry address has a page-sized displacement between the entry point specified by kernel EFI file. The bug was resolved after I rewrite the kernel loader to load EFI section-wise, on the second half of Day 04.
  * For print-like debugging for kernels, inserting artificial values such as `0xCAFEBABE` or `0xDEADBEEF` into registers might help.
- [x] **Day 04 (Pixel Rendering)** '23.08.27.
  * Used [Noto Sans Mono](https://fonts.google.com/noto/specimen/Noto+Sans+Mono) as system font. Symbols for ASCII control characters are self-made.
  * It turns out that specifying kernel address to linker is somewhat redundant.
- [x] **Day 05 (Text Rendering and Console)** '23.09.04.
  * Our text formatting heavily depends on `core::fmt::Write` trait.
  * For implementing singletons in rust, one of the easiest implementations is unsafe `static mut`. Instead, I chose to implement mutual-exclusive singleton like `std::sync::OnceLock` by combining `core::cell::OnceCell` and `spin::Mutex`, following @phil-opp fashion. I will switch to `static mut` when more performance-intensive processes are added.
  * Spent a couple of days for debugging that rendering function pauses. I would call this 'self-deadlock' in the sense that a method requiring a mutex lock calls another method requiring the same lock. Of course, this is merely a design mistake, and can be avoided by classifying methods which are able to wait for the lock.
- [x] Day 06A (Mouse cursor implementation) '23.09.09.
  * Implemented PCI scan algorithms, both brute-force and PCI-to-PCI bridge DFS.
  * Modified `./sh/run_qemu.sh` to add `qemu-xhci` device.
- [x] Day 06B (USB driver implementation) '23.12.02.
  * Imported xHCI USB driver code into Rust. ~~WHICH TOOK 3 MONTHS~~
    * The temporary allocator is bump allocator.
    * Ongoing contribution to related open source crates ([accessor](https://github.com/toku-sa-n/accessor) and [xhci](https://github.com/rust-osdev/xhci))
    * Debugging threads: [#158](https://github.com/uchan-nos/os-from-zero/issues/158) and [#159](https://github.com/uchan-nos/os-from-zero/issues/159)
  * Other implementations
- [ ] Day 07
  * ???

...and so on.

## References
* [MikanOS](https://github.com/uchan-nos/mikanos) by @uchan-nos, and its implementations
  * [🍊 Mikan](https://github.com/siketyan/mikan) by @siketyan
* [Writing an OS with Rust](https://os.phil-opp.com) by @phil-opp