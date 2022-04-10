OUTPUT_FORMAT(elf32-i386)

ENTRY(_start)

SECTIONS
{
    . = 0x1000;
    .text : {
        guest.o(.text)
    }
    .data : {
        guest.o(.data)
    }
    .bss : {
        guest.o(.bss)
    }

    . = 0x2000;
    .text : {
        librsguest.a(.text)
    }
    .data : {
        librsguest.a(.data)
    }
    .bss : {
        librsguest.a(.bss)
    }

    . = 0x3000;
    .text : {
        cguest.o(.text)
    }
    .data : {
        cguest.o(.data)
    }
    .bss : {
        cguest.o(.bss)
    }
}