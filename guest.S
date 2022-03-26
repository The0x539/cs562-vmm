.section .text

.code16

.globl _start
_start:
mov $1000, %ax
out %ax, $0x46
mov $1, %al
out %al, $0x47

check:
in $0x47, %al
and $2, %al
jz check

print:
mov 'y', %al
out %al, $0x42
mov '\n', %al
out %al, $0x42
jmp check
