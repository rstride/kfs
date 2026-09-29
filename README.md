# KFS

KFS is a learning-focused Unix-like x86 kernel built from scratch in Rust and assembly as part of the École 42 Kernel From Scratch curriculum.

The repository keeps each curriculum milestone isolated. The implemented stages currently cover bootstrapping a freestanding 32-bit kernel, linking and loading it with GRUB, writing directly to the VGA text buffer, defining a Global Descriptor Table, and exposing a minimal interactive shell.

## Implemented capabilities

- Multiboot-compatible assembly entry point
- Freestanding Rust kernel with a custom linker script and i386 target
- VGA text output, formatting macros, scrolling, and backspace handling
- Kernel and user code/data/stack segments through a GDT
- Minimal command shell with `halt`, `reboot`, and kernel-stack inspection
- Bootable ISO generation and QEMU-based local execution

## Repository structure

- `kfs_1/`: boot flow and VGA output foundation
- `kfs_2/`: segmentation, GDT loading, and interactive shell
- `subjects/`: the original curriculum specifications

Each milestone contains its own Rust configuration, linker script, GRUB configuration, and Makefile so it can be built independently.

## Prerequisites

The project targets 32-bit x86 and requires a Rust toolchain with the required source components, an assembler/linker toolchain, GRUB ISO utilities, and QEMU for execution. Refer to the Makefile in the milestone you want to run for the exact commands and local package names.

## Scope

This is an educational kernel, not a production operating system. It intentionally prioritizes transparent low-level implementation and incremental learning over portability or production hardening.
