.PHONY: run vmm guest

guest/guest.elf: guest/guest.S guest/guest.ld
	gcc -c -o guest/guest.o guest/guest.S
	gcc -c -o guest/cguest.o guest/cguest.c
	ld -nostdlib -T guest/guest.ld -z max-page-size=0x1000 -Lguest/ -o guest/guest.elf

guest: guest/guest.elf

vmm:
	cargo build --release

all: vmm guest

run: vmm guest
	target/release/vmm guest/guest.elf

