; boot.s
global start
extern kmain

; Setting up the Multiboot header - see GRUB Multiboot Specification
MODULEALIGN equ  1<<0             ; align loaded modules on page boundaries
MEMINFO     equ  1<<1             ; provide memory map
FLAGS       equ  MODULEALIGN | MEMINFO
MAGIC       equ  0x1BADB002       ; 'magic number' lets bootloader find the header
CHECKSUM    equ -(MAGIC + FLAGS)  ; checksum of above, to prove we are multiboot

section .multiboot_header
align 4
    dd MAGIC
    dd FLAGS
    dd CHECKSUM

section .bss
align 16
stack_bottom:
    resb 16384 ; 16 KiB
stack_top:

section .text
start:
    mov esp, stack_top
    push ebx ; Push a pointer to the multiboot information structure
    push eax ; Push the magic value
    call kmain
    cli
.hang:
    hlt
    jmp .hang
