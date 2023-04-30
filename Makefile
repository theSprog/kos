OS_DIR = target/riscv64gc-unknown-none-elf/release
QEMU_FLAGS = -machine virt -nographic -smp 4
QEMU_BIOS = -bios ./boot/rustsbi-qemu.bin -device loader,file=./kos.bin,addr=0x80200000
QEMU = qemu-system-riscv64
TOOLS = ~/riscv64-elf-tools/bin

.PHONY: build run debugs debugc clean

build:
	make clean
	cargo b --release
	ln -sf $(OS_DIR)/kos ./kos
	rust-objcopy --strip-all ./kos -O binary ./kos.bin

run: build
	$(QEMU) $(QEMU_FLAGS) $(QEMU_BIOS)

# server
debugs: build
	$(QEMU) $(QEMU_FLAGS) $(QEMU_BIOS) -s -S

# client
debugc:
	$(TOOLS)/riscv64-unknown-elf-gdb -ex 'file ./kos' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234' -ex 'b *0x80200000' -ex 'c'

clean:
	@if test -e kos; then rm kos; fi
	@if test -e kos.bin; then rm kos.bin; fi
	@if test -n "$(wildcard target/*)"; then rm -r target/*; fi
