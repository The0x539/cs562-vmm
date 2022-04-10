#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// 🦀
typedef uint8_t u8;
typedef uint16_t u16;

// ain't got no SIMD or something

void *memset(void *s, int c, size_t n) {
	char *ss = s;
	char cc = c;
	for (size_t i = 0; i < n; i++) {
		ss[i] = cc;
	}
	return s;
}

void *memcpy(void *restrict dest, const void *restrict src, size_t n) {
	char *restrict d = dest;
	const char *restrict s = src;
	for (size_t i = 0; i < n; i++) {
		d[i] = s[i];
	}
	return dest;
}

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
			*out = inb(0x44);
			outb(0x45, 0);
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
		u8 ch;
		bool got_char = wait_or_get_char(&ch);
		if (got_char) {
			in_buf[idx++] = ch;
			if (ch == '\n') {
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
