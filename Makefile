RUST_GUEST_PATH=guest/rust
RUST_TARGET_PLATFORM=i686-unknown-linux-gnu
RUST_GUEST_MANIFEST=$(RUST_GUEST_PATH)/Cargo.toml
RUST_GUEST_ARTIFACTS=$(RUST_GUEST_PATH)/target/$(RUST_TARGET_PLATFORM)/release

.PHONY: run vmm rsguest guest clean

all: vmm guest

clean:
	@rm -f guest/guest.o guest/guest.elf
	@cd $(RUST_GUEST_PATH); cargo clean
	@cargo clean

rsguest:
	cargo build --release --target $(RUST_TARGET_PLATFORM) --manifest-path $(RUST_GUEST_MANIFEST)

guest/guest.elf: guest/guest.S rsguest guest/guest.ld
	gcc -c -m32 -o guest/guest.o guest/guest.S
	ld -m elf_i386 -nostdlib -T guest/guest.ld -z max-page-size=0x1000 -L guest/ -L $(RUST_GUEST_ARTIFACTS) -o guest/guest.elf

guest: guest/guest.elf

vmm:
	cargo build --release

run: vmm guest
	target/release/vmm guest/guest.elf

