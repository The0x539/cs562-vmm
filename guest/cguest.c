#include <stdint.h>
#include <stdbool.h>

// 🦀
typedef uint8_t u8;
typedef uint16_t u16;

u8 inb(u8 port) {
	u8 ret;
	asm ("inb %%dx,%%al":"=a" (ret):"d" (port));
	return ret;
}

void outb(u8 port, u8 val) {
	asm ("outb %%al,%%dx": :"d" (port), "a" (val));
}

void outs(u8 port, u16 val) {
	asm ("out %%ax,%%dx": :"d" (port), "a" (val));
}

void put_char(u8 ch) {
	outb(0x42, ch);
}

void timer_enable(u16 millis) {
	outs(0x46, millis);
	outb(0x47, 1);
}

bool wait_or_get_char(u8 *out) {
	while (true) {
		if (inb(0x45)) {
			u8 ch = inb(0x44);
			put_char(ch);
			outb(0x45, 0);
			*out = ch;
			return true;
		} else if (inb(0x47) & 2) {
			outb(0x47, 1);
			return false;
		}
	}
}

void print(u8 *str) {
	for (u8 i = 0; str[i] != '\0'; i++) {
		put_char(str[i]);
	}
}

void cmain() {
	timer_enable(750);

	u8 buf_a[64] = {0};
	u8 buf_b[64] = {0};
	u8 *out_buf = buf_a;
	u8 *in_buf = buf_b;
	u8 idx = 0;

	while (true) {
		u8 ch = 0xFF;
		bool got_char = wait_or_get_char(&ch);
		if (got_char) {
			put_char(ch);
			in_buf[idx++] = ch;
			if (ch == '\0') {
				u8 *tmp = in_buf;
				in_buf = out_buf;
				out_buf = tmp;
				idx = 0;
			}
		} else {
			print(out_buf);
		}
	}
}
