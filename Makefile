guest: guest.S guest.ld
	gcc -c -o guest.o guest.S
	ld -nostdlib -T guest.ld -z max-page-size=0x1000 guest.o -o guest
	
all: guest
