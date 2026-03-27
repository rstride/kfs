KERNEL = build/kfs_2.bin
ISO = kfs_2.iso

LD = /Users/romainstride/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/rust-lld
NASM = nasm
CARGO = ~/.cargo/bin/cargo

LDFLAGS = -flavor gnu -n -T linker.ld
NASMFLAGS = -f elf32

RUST_TARGET = i386-target.json

.PHONY: all clean run iso ensure_dirs

all: iso

ensure_dirs:
	mkdir -p build
	mkdir -p root/boot/grub

build/boot.o: src/boot.s | ensure_dirs
	$(NASM) $(NASMFLAGS) $< -o $@

build/libkfs_2.a: src/lib.rs src/vga_buffer.rs | ensure_dirs
	$(CARGO) build --release -Z build-std=core,compiler_builtins --target $(RUST_TARGET)
	cp target/i386-target/release/libkfs_2.a build/

$(KERNEL): build/boot.o  build/libkfs_2.a | ensure_dirs
	$(LD) $(LDFLAGS) -o $@ build/boot.o build/libkfs_2.a

iso: $(KERNEL) | ensure_dirs
	cp $(KERNEL) root/boot/
	cp grub.cfg root/boot/grub/
	i686-elf-grub-mkrescue -o $(ISO) root

run: iso
	qemu-system-i386 -cdrom $(ISO) -m 32M -serial stdio

clean:
	$(CARGO) clean
	rm -rf build root $(ISO)
