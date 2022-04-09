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
        cguest.o(.text)
    }
    .data : {
        cguest.o(.data)
    }
    .bss : {
        cguest.o(.bss)
    }
}