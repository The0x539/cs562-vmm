.PHONY: run vmm guest clean

all: vmm guest

clean:
	@rm -f guest/guest.o guest/cguest.o guest/guest.elf
	@cargo clean

guest/guest.elf: guest/guest.S guest/cguest.c guest/guest.ld
	gcc -c -o guest/guest.o guest/guest.S
	gcc -c -o guest/cguest.o guest/cguest.c
	ld -nostdlib -T guest/guest.ld -z max-page-size=0x1000 -Lguest/ -o guest/guest.elf

guest: guest/guest.elf

vmm:
	cargo build --release

run: vmm guest
	target/release/vmm guest/guest.elf

