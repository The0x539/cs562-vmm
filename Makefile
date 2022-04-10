RUST_GUEST_PATH=guest/rust
RUST_GUEST_MANIFEST=$(RUST_GUEST_PATH)/Cargo.toml
RUST_GUEST_ARTIFACTS=$(RUST_GUEST_PATH)/target/i686-unknown-linux-gnu/release

.PHONY: run vmm rsguest guest clean

all: vmm guest

clean:
	@rm -f guest/guest.o guest/cguest.o guest/guest.elf
	@cd $(RUST_GUEST_PATH); cargo clean
	@cargo clean

rsguest:
	cargo build --release --target i686-unknown-linux-gnu --manifest-path $(RUST_GUEST_MANIFEST)

guest/guest.elf: guest/guest.S rsguest guest/guest.ld
	gcc -c -m32 -o guest/guest.o guest/guest.S
	ld -m elf_i386 -nostdlib -T guest/guest.ld -z max-page-size=0x1000 -L guest/ -L $(RUST_GUEST_ARTIFACTS) -o guest/guest.elf

guest: guest/guest.elf

vmm:
	cargo build --release

run: vmm guest
	target/release/vmm guest/guest.elf

