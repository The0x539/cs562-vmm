.section .text

.code16
.globl _start
_start:
    cli
    mov $gdtr32, %eax
    lgdt (%eax)
    mov %cr0, %eax
    or $1, %al
    mov %eax, %cr0
    ljmp $0x8, $thirtytwo

.code32
thirtytwo:
    mov $0x8FFF, %esp
    call cmain

.section .data

gdt32:
    .quad 0x0000000000000000
    .quad 0x00cf9a000000ffff
    .quad 0x00cf92000000ffff
gdtr32:
    .word 23
    .long gdt32