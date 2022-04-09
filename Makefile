guest/guest.elf: guest/guest.S guest/guest.ld
	gcc -c -o guest/guest.o guest/guest.S
	ld -nostdlib -T guest/guest.ld -z max-page-size=0x1000 guest/guest.o -o guest/guest.elf

guest: guest/guest.elf
	
all: guest

.PHONY: guest
