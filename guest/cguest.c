#include <stdint.h>

// 🦀
typedef uint8_t u8;
typedef uint16_t u16;

u8 inb(u8 port) {
	u8 ret;
	asm volatile ("inb %%dx,%%al":"=a" (ret):"d" (port));
	return ret;
}

void outb(u8 port, u8 val) {
	asm volatile ("outb %%al,%%dx": :"d" (port), "a" (val));
}

u8 get_char() {
	while (inb(0x45) == 0);
	u8 ch = inb(0x44);
	outb(0x45, 0);
	return ch;
}

void put_char(u8 ch) {
	outb(0x42, ch);
}

void cmain() {
	while (1) {
		char ch = get_char();
		put_char(ch);
	}
	return;
}
